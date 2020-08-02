#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{
    BaseConsumer, CommitMode, Consumer, ConsumerContext, Rebalance,
};
use rdkafka::error::KafkaResult;
use rdkafka::message::Message as RDKafkaMessage;
use rdkafka::producer::{
    BaseRecord, DefaultProducerContext, ThreadedProducer,
};
use rdkafka::topic_partition_list::TopicPartitionList;
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

// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
struct CustomConsumerContext;

type CustomConsumer = BaseConsumer<CustomConsumerContext>;
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

#[cfg(feature = "sasl")]
pub struct SASLConfig {
    pub username: String,
    pub password: String,
}

impl ClientContext for CustomConsumerContext {}

impl ConsumerContext for CustomConsumerContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        println!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        println!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(
        &self,
        result: KafkaResult<()>,
        _offsets: &TopicPartitionList,
    ) {
        println!("Committing offsets: {:?}", result);
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # Kafka Subscribe Client ( Consumer )
///
/// This client lib will consumer messages and place them into an
/// MPSC Sender<crate::kafka::Message>.
///
/// ```no_run
/// use kafka_bridge::kafka::SubscribeClient;
/// use kafka_bridge::pubnub::Message;
///
/// let brokers = "0.0.0.0:9094".split(",").map(|s| s.to_string()).collect();
/// let (kafka_message_tx, kafka_message_rx) = mpsc::channel();
/// let kafka_topic                          = "topic";
/// let kafka_partition                      = 0;
/// let kafka_group                          = "";
///
/// let mut kafka = match kafka::SubscribeClient::new(
///     &brokers,
///     kafka_message_tx.clone(),
///     &kafka_topic,
///     &kafka_group,
///     kafka_partition,
/// ) {
///     Ok(kafka)  => kafka,
///     Err(error) => { println!("{}", error); }
/// };
///
/// // Consume messages from broker and make them available
/// // to `kafka_message_rx`.
/// kafka.consume().expect("Error consuming Kafka messages");
///
/// let message: Message =
///     kafka_message_rx.recv().expect("MPSC Channel Receiver");
/// ```
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl SubscribeClient {
    /// # Errors
    ///
    /// Will return `Err` if failed to initialize kafka consumer.
    pub fn new(
        brokers: &[String],
        sender: Sender<Message>,
        topic: &str,
        group: &str,
        partition: i32,
    ) -> Result<Self, Error> {
        let context = CustomConsumerContext;
        let config =
            SubscribeClient::create_client_config(brokers, group, partition);
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

    #[cfg(feature = "sasl")]
    /// # Errors
    ///
    /// Will return `Err` if failed to initialize kafka consumer.
    pub fn new_with_sasl(
        brokers: &[String],
        sender: Sender<Message>,
        topic: &str,
        group: &str,
        partition: i32,
        sasl_cfg: &SASLConfig,
    ) -> Result<Self, Error> {
        let context = CustomConsumerContext;
        let mut config =
            SubscribeClient::create_client_config(brokers, group, partition);
        config
            .set("security.protocol", "sasl_plaintext")
            .set("sasl.mechanism", "PLAIN")
            .set("sasl.username", &sasl_cfg.username)
            .set("sasl.password", &sasl_cfg.password);

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
    /// # Errors
    ///
    /// Will return `Err` if failed to consume item from kafka feed.
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

    fn create_client_config(
        brokers: &[String],
        group: &str,
        partition: i32,
    ) -> ClientConfig {
        let mut cfg = ClientConfig::new();
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
/// let brokers = "0.0.0.0:9094".split(",").map(|s| s.to_string()).collect();
/// let mut kafka = match kafka::PublishClient::new(&brokers, topic) {
///     Ok(kafka)  => kafka,
///     Err(error) => { println!("{}", error); }
/// };
///
/// loop {
///     let message: kafka_bridge::pubnub::Message =
///         kafka_publish_rx.recv().expect("MPSC Channel Receiver");
///     match kafka.produce(&message.data) {
///         Ok(())     => {}
///         Err(error) => { println!("{}", error); }
///     };
/// }
/// ```
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl PublishClient {
    /// # Errors
    ///
    /// Will return `Err` if failed to initialize kafka consumer.
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

    #[cfg(feature = "sasl")]
    /// # Errors
    ///
    /// Will return `Err` if failed to initialize kafka consumer.
    pub fn new_with_sasl(
        brokers: &[String],
        topic: &str,
        sasl_cfg: &SASLConfig,
    ) -> Result<Self, Error> {
        let producer: KafkaResult<CustomProducer> = ClientConfig::new()
            .set("bootstrap.servers", &brokers[0])
            .set("request.timeout.ms", "1000")
            .set("acks", "1")
            .set("security.protocol", "sasl_plaintext")
            .set("sasl.mechanism", "PLAIN")
            .set("sasl.username", &sasl_cfg.username)
            .set("sasl.password", &sasl_cfg.password)
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

    /// # Errors
    ///
    /// Will return `Err` if failed to send message to kafka producer.
    pub fn produce(&mut self, message: &str) -> KafkaResult<()> {
        self.producer
            .send(BaseRecord::to(&self.topic).payload(message).key(""))
            .map_err(|(err, _)| err)
    }
}
