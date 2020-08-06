use super::{ClientConfig, Error};
use rdkafka::error::KafkaResult;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use tokio::time::Duration;

#[allow(clippy::needless_doctest_main)] // needed for async main
/// Kafka Publish Client (Producer)
///
/// This client lib will produce messages into Kafka.
///
/// ```no_run
/// # #[tokio::main]
/// # async fn main() {
/// use kafka_bridge::kafka;
/// use std::sync::mpsc;
///
/// let (kafka_publish_tx, kafka_publish_rx) = mpsc::channel();
/// let brokers = "0.0.0.0:9094"
///     .split(",")
///     .map(ToString::to_string)
///     .collect::<Vec<_>>();
/// let mut kafka = match kafka::PublishClient::new(
///     kafka::ClientConfig::Plain,
///     &brokers,
///     "topic",
/// ) {
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
///     match kafka.produce(&message.data).await {
///         Ok(()) => {}
///         Err(error) => {
///             println!("{:?}", error);
///         }
///     };
/// }
/// # }
/// ```
pub struct Client {
    producer: FutureProducer,
    topic: String,
}

impl Client {
    /// Creates a new [`PublishClient`](Client).
    ///
    /// # Errors
    ///
    /// This function can return [`Error::KafkaInitialize`] on Kafka client
    /// initialization failure.
    pub fn new(
        config: ClientConfig,
        brokers: &[String],
        topic: &str,
    ) -> Result<Self, Error> {
        let producer = rdkafka::ClientConfig::from(config)
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
    pub async fn produce(&mut self, message: &str) -> KafkaResult<()> {
        self.producer
            .send(
                FutureRecord::<'_, (), _>::to(&self.topic).payload(message),
                Timeout::After(Duration::from_millis(5 * 1000)),
            )
            .await
            .map(|_| ())
            .map_err(|(err, _)| err)
    }
}
