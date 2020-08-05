use rdkafka::consumer::{BaseConsumer, Consumer as _};
use rdkafka::error::KafkaResult;
use rdkafka::producer::{
    BaseRecord, DefaultProducerContext, ThreadedProducer,
};
use rdkafka::{ClientConfig, Message as _, Offset, TopicPartitionList};
use std::sync::mpsc::Sender;

pub struct Message {
    pub topic: String,
    pub group: String,
    pub data: String,
}

pub struct PublishClient {
    producer: ThreadedProducer<DefaultProducerContext>,
    topic: String,
}

pub struct SubscribeClient {
    consumer: BaseConsumer,
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
pub struct SaslPlainConfig {
    pub username: String,
    pub password: String,
}

#[cfg(feature = "sasl-ssl")]
pub struct SaslSslConfig {
    pub username: String,
    pub password: String,
    pub ca_location: String,
    pub certificate_location: String,
    pub key_location: String,
    pub key_password: String,
}

#[cfg(feature = "sasl-gssapi")]
pub struct SaslGssapiConfig {
    pub kerberos_service_name: String,
    pub kerberos_keytab: String,
    pub kerberos_principal: String,
    pub ca_location: String,
    pub certificate_location: String,
    pub key_location: String,
    pub key_password: String,
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
/// let kafka_topic = "topic";
/// let kafka_partition = 0;
/// let kafka_group = "";
///
/// let mut kafka = match kafka::SubscribeClient::new(
///     brokers,
///     kafka_message_tx.clone(),
///     &kafka_topic,
///     &kafka_group,
///     kafka_partition,
/// ) {
///     Ok(kafka) => kafka,
///     Err(error) => {
///         println!("{:?}", error);
///         return;
///     }
/// };
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
    #[allow(clippy::needless_pass_by_value)]
    // TODO apply the lint on the next breaking release
    /// Creates a new [`SubscribeClient`].
    ///
    /// # Errors
    ///
    /// This function can return [`Error::KafkaInitialize`] on Kafka client
    /// initialization failure.
    pub fn new(
        brokers: Vec<String>,
        sender: Sender<Message>,
        topic: &str,
        group: &str,
        partition: i32,
    ) -> Result<Self, Error> {
        let consumer: BaseConsumer = ClientConfig::new()
            .set("group.id", group)
            .set("metadata.broker.list", &brokers.join(","))
            .create()
            .map_err(|_| Error::KafkaInitialize)?;
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition_offset(topic, partition, Offset::Beginning);
        consumer.assign(&tpl).map_err(|_| Error::KafkaInitialize)?;

        Ok(Self {
            consumer,
            sender,
            topic: topic.into(),
            group: group.into(),
        })
    }

    #[cfg(feature = "sasl-plain")]
    /// Creates a new [`SubscribeClient`] using SASL with user/pass.
    ///
    /// # Errors
    ///
    /// This function can return [`Error::KafkaInitialize`] on Kafka client
    /// initialization failure.
    pub fn new_sasl_plain(
        brokers: &[String],
        sender: Sender<Message>,
        topic: &str,
        group: &str,
        partition: i32,
        sasl_plain_config: &SaslPlainConfig,
    ) -> Result<Self, Error> {
        let consumer = ClientConfig::from(sasl_plain_config)
            .set("group.id", group)
            .set("metadata.broker.list", &brokers.join(","))
            .create::<BaseConsumer>()
            .map_err(|_| Error::KafkaInitialize)?;
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition_offset(topic, partition, Offset::Beginning);
        consumer.assign(&tpl).map_err(|_| Error::KafkaInitialize)?;

        Ok(Self {
            consumer,
            sender,
            topic: topic.into(),
            group: group.into(),
        })
    }

    #[cfg(feature = "sasl-ssl")]
    /// Creates a new [`SubscribeClient`] using SASL with user/pass over SSL.
    ///
    /// # Errors
    ///
    /// This function can return [`Error::KafkaInitialize`] on Kafka client
    /// initialization failure.
    pub fn new_sasl_ssl(
        brokers: &[String],
        sender: Sender<Message>,
        topic: &str,
        group: &str,
        partition: i32,
        sasl_ssl_config: &SaslSslConfig,
    ) -> Result<Self, Error> {
        let consumer = ClientConfig::from(sasl_ssl_config)
            .set("group.id", group)
            .set("metadata.broker.list", &brokers.join(","))
            .create::<BaseConsumer>()
            .map_err(|_| Error::KafkaInitialize)?;
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition_offset(topic, partition, Offset::Beginning);
        consumer.assign(&tpl).map_err(|_| Error::KafkaInitialize)?;

        Ok(Self {
            consumer,
            sender,
            topic: topic.into(),
            group: group.into(),
        })
    }

    #[cfg(feature = "sasl-gssapi")]
    /// Creates a new [`SubscribeClient`] using SASL with GSSAPI.
    ///
    /// # Errors
    ///
    /// This function can return [`Error::KafkaInitialize`] on Kafka client
    /// initialization failure.
    pub fn new_sasl_gssapi(
        brokers: &[String],
        sender: Sender<Message>,
        topic: &str,
        group: &str,
        partition: i32,
        sasl_gssapi_config: &SaslGssapiConfig,
    ) -> Result<Self, Error> {
        let consumer = ClientConfig::from(sasl_gssapi_config)
            .set("group.id", group)
            .set("metadata.broker.list", &brokers.join(","))
            .create::<BaseConsumer>()
            .map_err(|_| Error::KafkaInitialize)?;
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition_offset(topic, partition, Offset::Beginning);
        consumer.assign(&tpl).map_err(|_| Error::KafkaInitialize)?;

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
        for message in &self.consumer {
            let message = message?;
            println!(
                "Message {{ offset: {:?}, key: {:?}, payload: {:?} }}",
                message.offset(),
                message.key(),
                message.payload()
            );
            let mut data = match message.payload_view::<str>() {
                None => String::new(),
                Some(Ok(s)) => s.into(),
                Some(Err(e)) => {
                    panic!(
                        "Error while deserializing message payload: {:?}",
                        e
                    );
                }
            };

            let parsetest = json::parse(&data);
            if parsetest.is_err() {
                data = json::stringify(data);
            }

            self.sender
                .send(Message {
                    topic: self.topic.clone(),
                    group: self.group.clone(),
                    data,
                })
                .expect("Error writing to mpsc Sender");
        }
        Ok(())
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
/// let (kafka_publish_tx, kafka_publish_rx) = mpsc::channel();
/// let brokers = "0.0.0.0:9094".split(",").map(|s| s.to_string()).collect();
/// let mut kafka = match kafka::PublishClient::new(brokers, "topic") {
///     Ok(kafka) => kafka,
///     Err(error) => {
///         println!("{:?}", error);
///         return;
///     }
/// };
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
    #[allow(clippy::needless_pass_by_value)]
    // TODO apply the lint on the next breaking release
    /// Creates a new [`PublishClient`].
    ///
    /// # Errors
    ///
    /// This function can return [`Error::KafkaInitialize`] on Kafka client
    /// initialization failure.
    pub fn new(brokers: Vec<String>, topic: &str) -> Result<Self, Error> {
        let producer = ClientConfig::new()
            .set("metadata.broker.list", &brokers.join(","))
            .set("request.required.acks", "1")
            .set("request.timeout.ms", "1000")
            .create()
            .map_err(|_| Error::KafkaInitialize)?;

        Ok(Self {
            producer,
            topic: topic.into(),
        })
    }

    #[cfg(feature = "sasl-plain")]
    /// Creates a new [`PublishClient`] using SASL with user/pass.
    ///
    /// # Errors
    ///
    /// This function can return [`Error::KafkaInitialize`] on Kafka client
    /// initialization failure.
    pub fn new_sasl_plain(
        brokers: &[String],
        topic: &str,
        sasl_plain_config: &SaslPlainConfig,
    ) -> Result<Self, Error> {
        let producer = ClientConfig::from(sasl_plain_config)
            .set("metadata.broker.list", &brokers.join(","))
            .set("request.required.acks", "1")
            .set("request.timeout.ms", "1000")
            .create()
            .map_err(|_| Error::KafkaInitialize)?;

        Ok(Self {
            producer,
            topic: topic.into(),
        })
    }

    #[cfg(feature = "sasl-ssl")]
    /// Creates a new [`PublishClient`] using SASL with user/pass over SSL.
    ///
    /// # Errors
    ///
    /// This function can return [`Error::KafkaInitialize`] on Kafka client
    /// initialization failure.
    pub fn new_sasl_ssl(
        brokers: &[String],
        topic: &str,
        sasl_ssl_config: &SaslSslConfig,
    ) -> Result<Self, Error> {
        let producer = ClientConfig::from(sasl_ssl_config)
            .set("metadata.broker.list", &brokers.join(","))
            .set("request.required.acks", "1")
            .set("request.timeout.ms", "1000")
            .create()
            .map_err(|_| Error::KafkaInitialize)?;

        Ok(Self {
            producer,
            topic: topic.into(),
        })
    }

    #[cfg(feature = "sasl-gssapi")]
    /// Creates a new [`PublishClient`] using SASL with GSSAPI.
    ///
    /// # Errors
    ///
    /// This function can return [`Error::KafkaInitialize`] on Kafka client
    /// initialization failure.
    pub fn new_sasl_gssapi(
        brokers: &[String],
        topic: &str,
        sasl_gssapi_config: &SaslGssapiConfig,
    ) -> Result<Self, Error> {
        let producer = ClientConfig::from(sasl_gssapi_config)
            .set("metadata.broker.list", &brokers.join(","))
            .set("request.required.acks", "1")
            .set("request.timeout.ms", "1000")
            .create()
            .map_err(|_| Error::KafkaInitialize)?;

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
            .send(BaseRecord::<'_, (), _>::to(&self.topic).payload(message))
            .map_err(|(err, _)| err)
    }
}

#[cfg(feature = "sasl-plain")]
impl From<&SaslPlainConfig> for ClientConfig {
    fn from(config: &SaslPlainConfig) -> Self {
        let SaslPlainConfig { username, password } = config;
        let mut config = ClientConfig::new();
        config
            .set("security.protocol", "SASL_PLAINTEXT")
            .set("sasl.mechanism", "PLAIN")
            .set("sasl.username", username)
            .set("sasl.password", password);
        config
    }
}

#[cfg(feature = "sasl-ssl")]
impl From<&SaslSslConfig> for ClientConfig {
    fn from(config: &SaslSslConfig) -> Self {
        let SaslSslConfig {
            username,
            password,
            ca_location,
            certificate_location,
            key_location,
            key_password,
        } = config;
        let mut config = ClientConfig::new();
        config
            .set("security.protocol", "SASL_SSL")
            .set("sasl.mechanism", "PLAIN")
            .set("sasl.username", username)
            .set("sasl.password", password)
            .set("ssl.ca.location", ca_location)
            .set("ssl.certificate.location", certificate_location)
            .set("ssl.key.location", key_location)
            .set("ssl.key.password", key_password);
        config
    }
}

#[cfg(feature = "sasl-gssapi")]
impl From<&SaslGssapiConfig> for ClientConfig {
    fn from(config: &SaslGssapiConfig) -> Self {
        let SaslGssapiConfig {
            kerberos_service_name,
            kerberos_keytab,
            kerberos_principal,
            ca_location,
            certificate_location,
            key_location,
            key_password,
        } = config;
        let mut config = ClientConfig::new();
        config
            .set("security.protocol", "SASL_SSL")
            .set("sasl.kerberos.service.name", kerberos_service_name)
            .set("sasl.kerberos.keytab", kerberos_keytab)
            .set("sasl.kerberos.principal", kerberos_principal)
            .set("ssl.ca.location", ca_location)
            .set("ssl.certificate.location", certificate_location)
            .set("ssl.key.location", key_location)
            .set("ssl.key.password", key_password);
        config
    }
}
