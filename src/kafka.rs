use json;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use kafka::error::Error as KafkaError;
use kafka::producer::{Producer, Record, RequiredAcks};
use std::sync::mpsc::Sender;
use std::time::Duration;

pub struct Message {
    pub topic: String,
    pub group: String,
    pub data: String,
}

pub struct PublishClient {
    producer: Producer,
    topic: String,
}

pub struct SubscribeClient {
    consumer: Consumer,
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
        let consumer = match Consumer::from_hosts(brokers)
            .with_topic_partitions(topic.to_owned(), &[partition])
            .with_fallback_offset(FetchOffset::Earliest)
            .with_group(group.to_owned())
            .with_offset_storage(GroupOffsetStorage::Kafka)
            .create()
        {
            Ok(result) => result,
            Err(_err) => return Err(Error::KafkaInitialize),
        };

        Ok(Self {
            consumer: consumer,
            sender: sender,
            topic: topic.into(),
            group: group.into(),
        })
    }

    pub fn consume(&mut self) -> Result<(), KafkaError> {
        loop {
            for ms in self.consumer.poll().unwrap().iter() {
                for m in ms.messages() {
                    println!("{:?}", m);
                    let mut data = match String::from_utf8(m.value.to_vec()) {
                        Ok(v) => v,
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    };

                    let parsetest = json::parse(&data);
                    if parsetest.is_err() {
                        data = json::stringify(data);
                    }

                    self.sender
                        .send(Message {
                            topic: self.topic.clone(),
                            group: self.group.clone(),
                            data: data,
                        })
                        .expect("Error writing to mpsc Sender");
                }
                self.consumer
                    .consume_messageset(ms)
                    .expect("Error marking MessageSet as consumed.");
            }
            self.consumer.commit_consumed().unwrap();
        }
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # Kafka Publish Client ( Producer )
///
/// This client lib will produce messages into Kafka.
///
/// ```no_run
/// let brokers = "0.0.0.0:9094".split(",").map(|s| s.to_string()).collect();
/// let mut kafka = match kafka::PublishClient::new(brokers, topic) {
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
    pub fn new(brokers: Vec<String>, topic: &str) -> Result<Self, Error> {
        let producer = match Producer::from_hosts(brokers)
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
            .create()
        {
            Ok(result) => result,
            Err(_err) => return Err(Error::KafkaInitialize),
        };

        Ok(Self {
            producer: producer,
            topic: topic.into(),
        })
    }

    pub fn produce(&mut self, message: &str) -> Result<(), KafkaError> {
        return self.producer.send(&Record::from_value(
            &self.topic.to_string(),
            message.as_bytes(),
        ));
    }
}
