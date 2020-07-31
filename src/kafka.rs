use json;
use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{
    BaseConsumer, CommitMode, Consumer, ConsumerContext, Rebalance,
};
use rdkafka::error::KafkaResult;
use rdkafka::message::Message as RDKafkaMessage;
use rdkafka::producer::BaseProducer;
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::util::get_rdkafka_version;
use std::sync::mpsc::Sender;

pub struct Message {
    pub topic: String,
    pub group: String,
    pub data: String,
}

pub struct PublishClient {
    producer: BaseProducer,
    topic: String,
}

// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
struct CustomContext;

type CustomConsumer = BaseConsumer<CustomContext>;

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

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        // info!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        // info!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(
        &self,
        result: KafkaResult<()>,
        _offsets: &TopicPartitionList,
    ) {
        // info!("Committing offsets: {:?}", result);
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
///     brokers,
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
    pub fn new(
        brokers: Vec<String>,
        sender: Sender<Message>,
        topic: &str,
        group: &str,
        partition: i32,
    ) -> Result<Self, Error> {
        let context = CustomContext;
        let consumer: CustomConsumer = ClientConfig::new()
            .set("group.id", group)
            .set("bootstrap.servers", &brokers[0])
            .set_log_level(RDKafkaLogLevel::Debug)
            .set("enable.partition.eof", "false")
            .create_with_context(context)
            .expect("Consumer creating failed");
        // let consumer = match Consumer::from_hosts(brokers)
        //       .with_topic_partitions(topic.to_owned(), &[partition])
        //       .with_fallback_offset(FetchOffset::Earliest)
        //       .with_group(group.to_owned())
        //       .with_offset_storage(GroupOffsetStorage::Kafka)
        //       .create() {
        //         Ok(result) => result,
        //         Err(_err)  => return Err(Error::KafkaInitialize),
        //     };

        match consumer.subscribe(&[topic]) {
            Err(err) => {
                println!("Failed to initialize: {}", err);
                return Err(Error::KafkaInitialize);
            }
            _ => {}
        }

        Ok(Self {
            consumer: consumer,
            sender: sender,
            topic: topic.into(),
            group: group.into(),
        })
    }

    pub fn consume(&mut self) -> KafkaResult<()> {
        while let Some(r) = self.consumer.poll(None) {
            let m = match r {
                Err(e) => return Err(e),
                Ok(m) => m,
            };

            let payload = match m.payload_view::<str>() {
                None => "",
                Some(Ok(s)) => s,
                Some(Err(_e)) => "",
            };
            println!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                  m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());

            let parsetest = json::parse(&payload);
            if parsetest.is_err() {
                let data = json::stringify(payload);

                self.sender
                    .send(Message {
                        topic: self.topic.clone(),
                        group: self.group.clone(),
                        data: data,
                    })
                    .expect("Error writing to mpsc Sender");
            }

            self.consumer.commit_message(&m, CommitMode::Async)?;
        }

        Ok(())
    }
}

// impl PublishClient {
//     pub fn new(brokers: Vec<String>, topic: &str) -> Result<Self, Error> {
//         let producer = match Producer::from_hosts(brokers)
//             .with_ack_timeout(Duration::from_secs(1))
//             .with_required_acks(RequiredAcks::One)
//             .create()
//         {
//             Ok(result) => result,
//             Err(_err) => return Err(Error::KafkaInitialize),
//         };

//         Ok(Self {
//             producer: producer,
//             topic: topic.into(),
//         })
//     }

//     pub fn produce(&mut self, message: &str) -> Result<(), KafkaError> {
//         return self.producer.send(&Record::from_value(
//             &self.topic.to_string(),
//             message.as_bytes(),
//         ));
//     }
// }
