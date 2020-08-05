#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{
    BaseConsumer, CommitMode, Consumer, DefaultConsumerContext,
};
use rdkafka::error::KafkaResult;
use rdkafka::message::Message as RDKafkaMessage;
use rdkafka::producer::{
    BaseRecord, DefaultProducerContext, ThreadedProducer,
};
use std::sync::mpsc::Sender;

pub struct Message {
    pub topic: String,
    pub group: String,
    pub data: String,
}

pub struct PublishClient {
    producer: CustomProducer,
    topic: String,
}

type CustomConsumer = BaseConsumer<DefaultConsumerContext>;
type CustomProducer = ThreadedProducer<DefaultProducerContext>;

pub struct SubscribeClient {
    consumer: CustomConsumer,
    sender: Sender<Message>,
    topic: String,
    group: String,
}

#[derive(Debug)]
pub enum Error {
    KafkaInitialize,
    Publish,
    PublishWrite,
    PublishResponse,
    Subscribe,
    SubscribeWrite,
    SubscribeRead,
    MissingTopic,
    HTTPResponse,
}

#[cfg(feature = "sasl-plain")]
pub struct SASLConfig {
    pub username: String,
    pub password: String,
}

#[cfg(feature = "sasl-ssl")]
pub struct SASLConfig {
    pub username: String,
    pub password: String,
    pub ca_location: String,
    pub certificate_location: String,
    pub key_location: String,
    pub key_password: String,
}

#[cfg(feature = "sasl-plain")]
impl From<&SASLConfig> for ClientConfig {
    fn from(src: &SASLConfig) -> ClientConfig {
        let mut cfg = ClientConfig::new();
        cfg.set("security.protocol", "sasl_plaintext")
            .set("sasl.mechanism", "PLAIN")
            .set("sasl.username", &src.username)
            .set("sasl.password", &src.password);
        cfg
    }
}

#[cfg(feature = "sasl-ssl")]
impl From<&SASLConfig> for ClientConfig {
    fn from(src: &SASLConfig) -> ClientConfig {
        let mut cfg = ClientConfig::new();
        cfg.set("security.protocol", "sasl_ssl")
            .set("sasl.mechanism", "PLAIN")
            .set("ssl.ca.location", &src.ca_location)
            .set("ssl.certificate.location", &src.certificate_location)
            .set("ssl.key.location", &src.key_location)
            .set("ssl.key.password", &src.key_password)
            .set("sasl.username", &src.username)
            .set("sasl.password", &src.password);
        cfg
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # Kafka Subscribe Client ( Consumer )
///
/// This client lib will consumer messages and place them into an
/// MPSC Sender<crate::kafka::Message>.
///
/// ```no_run
/// use kafka_bridge::kafka;
/// use std::sync::mpsc;
///
/// let brokers = "0.0.0.0:9094".split(",").map(|s| s.to_string()).collect();
/// let (kafka_message_tx, kafka_message_rx) = mpsc::channel();
/// let kafka_topic                          = "topic";
/// let kafka_group                          = "";
///
/// let mut kafka = match kafka::SubscribeClient::new(
///     &brokers,
///     kafka_message_tx.clone(),
///     &kafka_topic,
///     &kafka_group,
/// ) {
///     Ok(kafka) => kafka,
///     Err(error) => {
///         println!("{:?}", error);
///         return;
///     }
///
/// // Consume messages from broker and make them available
/// // to `kafka_message_rx`.
/// kafka.consume().expect("Error consuming Kafka messages");
///
/// let message: kafka::Message =
///     kafka_message_rx.recv().expect("MPSC Channel Receiver");
/// ```
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl SubscribeClient {
    /// # Errors
    ///
    /// This function can return [`Error::KafkaInitialize`] on Kafka client
    /// initialization failure.
    pub fn new(
        brokers: &[String],
        sender: Sender<Message>,
        topic: &str,
        group: &str,
    ) -> Result<Self, Error> {
        let context = DefaultConsumerContext;
        let config = SubscribeClient::fill_client_config(
            ClientConfig::new(),
            brokers,
            group,
        );
        let consumer: KafkaResult<CustomConsumer> =
            config.create_with_context(context);

        let consumer = consumer.map_err(|err| {
            println!("Failed to intialize consumer: {}", err);
            Error::KafkaInitialize
        })?;

        consumer.subscribe(&[topic]).map_err(|err| {
            println!("Failed to initialize: {}", err);
            Error::KafkaInitialize
        })?;

        Ok(Self {
            consumer,
            sender,
            topic: topic.into(),
            group: group.into(),
        })
    }

    #[cfg(any(feature = "sasl-plain", feature = "sasl-ssl"))]
    /// Creates a new [`SubscribeClient`] using SASL with SASL_PLAINTEXT or SASL_SSL depending on config.
    ///
    /// # Errors
    ///
    /// This function can return [`Error::KafkaInitialize`] on Kafka client
    /// initialization failure.
    pub fn new_with_sasl(
        brokers: &[String],
        sender: Sender<Message>,
        topic: &str,
        group: &str,
        sasl_cfg: &SASLConfig,
    ) -> Result<Self, Error> {
        let context = CustomConsumerContext;
        let config = SubscribeClient::fill_client_config(
            ClientConfig::from(sasl_cfg),
            brokers,
            group,
        );

        let consumer: KafkaResult<CustomConsumer> =
            config.create_with_context(context);

        let consumer = consumer.map_err(|err| {
            println!("Failed to intialize consumer: {}", err);
            Error::KafkaInitialize
        })?;

        consumer.subscribe(&[topic]).map_err(|err| {
            println!("Failed to initialize: {}", err);
            Error::KafkaInitialize
        })?;

        Ok(Self {
            consumer,
            sender,
            topic: topic.into(),
            group: group.into(),
        })
    }
    /// Consumes messages and sends them through the channel.
    ///
    /// # Errors
    ///
    /// This function can return [`KafkaError`](rdkafka::error::KafkaError) on
    /// unsuccessful poll.
    pub fn consume(&mut self) -> KafkaResult<()> {
        while let Some(r) = self.consumer.poll(None) {
            let m = r?;

            let mut data = match m.payload_view::<str>() {
                None => String::new(),
                Some(Ok(s)) => s.into(),
                Some(Err(_e)) => String::new(),
            };
            println!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                  m.key(), data, m.topic(), m.partition(), m.offset(), m.timestamp());

            let parsetest = json::parse(&data);
            if parsetest.is_err() {
                data = json::stringify(data);
            }

            self.sender
                .send(Message {
                    topic: self.topic.clone(),
                    group: self.group.clone(),
                    data: data.to_string(),
                })
                .expect("Error writing to mpsc Sender");

            self.consumer.commit_message(&m, CommitMode::Async)?;
        }

        Ok(())
    }

    fn fill_client_config(
        mut cfg: ClientConfig,
        brokers: &[String],
        group: &str,
    ) -> ClientConfig {
        cfg.set("group.id", group)
            .set("bootstrap.servers", &brokers[0])
            .set("enable.partition.eof", "false")
            .set("auto.offset.reset", "earliest")
            .set("enable.auto.commit", "true")
            .set_log_level(RDKafkaLogLevel::Debug);
        cfg
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # Kafka Publish Client ( Producer )
///
/// This client lib will produce messages into Kafka.
///
/// ```no_run
/// use kafka_bridge::kafka;
/// use std::sync::mpsc;
/// 
/// let brokers = "0.0.0.0:9094".split(",").map(|s| s.to_string()).collect();
/// let mut kafka = match kafka::PublishClient::new(brokers, "topic") {
///     Ok(kafka) => kafka,
///     Err(error) => {
///         println!("{:?}", error);
///         return;
///     }
///
/// loop {
///     let message: kafka_bridge::pubnub::Message =
///         kafka_publish_rx.recv().expect("MPSC Channel Receiver");
///     match kafka.produce(&message.data) {
///         Ok(()) => {}
///         Err(error) => {
///             println!("{:?}", error);
///         }
///     };
/// }
/// ```
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl PublishClient {
    /// Creates a new [`PublishClient`].
    ///
    /// # Errors
    ///
    /// This function can return [`Error::KafkaInitialize`] on Kafka client
    /// initialization failure.
    pub fn new(brokers: &[String], topic: &str) -> Result<Self, Error> {
        let producer: KafkaResult<CustomProducer> = ClientConfig::new()
            .set("bootstrap.servers", &brokers[0])
            .set("request.timeout.ms", "1000")
            .set("acks", "1")
            .create();

        let producer = producer.map_err(|err| {
            println!("Failed to init kafka producer: {}", err);
            Error::KafkaInitialize
        })?;

        Ok(Self {
            producer,
            topic: topic.into(),
        })
    }

    #[cfg(any(feature = "sasl-plain", feature = "sasl-ssl"))]
    /// Creates a new [`PublishClient`] using SASL with SASL_PLAINTEXT or SASL_SSL depending on config.
    ///
    /// # Errors
    ///
    /// This function can return [`Error::KafkaInitialize`] on Kafka client
    /// initialization failure.
    pub fn new_with_sasl(
        brokers: &[String],
        topic: &str,
        sasl_cfg: &SASLConfig,
    ) -> Result<Self, Error> {
        let producer: KafkaResult<CustomProducer> =
            ClientConfig::from(sasl_cfg)
                .set("bootstrap.servers", &brokers[0])
                .set("request.timeout.ms", "1000")
                .set("acks", "1")
                .create();

        let producer = producer.map_err(|err| {
            println!("Failed to init kafka producer: {}", err);
            Error::KafkaInitialize
        })?;

        Ok(Self {
            producer,
            topic: topic.into(),
        })
    }

    /// Sends `message` into Kafka.
    ///
    /// # Errors
    ///
    /// This function can return [`KafkaError`](rdkafka::error::KafkaError) on
    /// unsuccessful send.
    pub fn produce(&mut self, message: &str) -> KafkaResult<()> {
        self.producer
            .send(BaseRecord::to(&self.topic).payload(message).key(""))
            .map_err(|(err, _)| err)
    }
}
