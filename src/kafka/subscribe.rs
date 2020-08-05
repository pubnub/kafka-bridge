use super::{ClientConfig, Error, Message};
use rdkafka::consumer::{BaseConsumer, Consumer as _};
use rdkafka::error::KafkaResult;
use rdkafka::{Message as _, Offset, TopicPartitionList};
use std::sync::mpsc::Sender;

/// Kafka Subscribe Client (Consumer)
///
/// This client lib will consumer messages and place them into an
/// MPSC Sender<crate::kafka::Message>.
///
/// ```no_run
/// use kafka_bridge::kafka;
/// use std::sync::mpsc;
///
/// let brokers = "0.0.0.0:9094"
///     .split(",")
///     .map(ToString::to_string)
///     .collect::<Vec<_>>();
/// let (kafka_message_tx, kafka_message_rx) = mpsc::channel();
/// let kafka_topic = "topic";
/// let kafka_partition = 0;
/// let kafka_group = "";
///
/// let mut kafka = match kafka::SubscribeClient::new(
///     kafka::ClientConfig::Plain,
///     &brokers,
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
pub struct Client {
    consumer: BaseConsumer,
    sender: Sender<Message>,
    topic: String,
    group: String,
}

impl Client {
    /// Creates a new [`SubscribeClient`](Client).
    ///
    /// # Errors
    ///
    /// This function can return [`Error::KafkaInitialize`] on Kafka client
    /// initialization failure.
    pub fn new(
        config: ClientConfig,
        brokers: &[String],
        sender: Sender<Message>,
        topic: &str,
        group: &str,
        partition: i32,
    ) -> Result<Self, Error> {
        let consumer = rdkafka::ClientConfig::from(config)
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
