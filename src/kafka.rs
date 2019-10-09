use json;
use std::sync::mpsc::Sender;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use kafka::error::Error as KafkaError;


pub struct Message {
    pub root: String,
    pub topic: String,
    pub group: String,
    pub data: String,
}

/*
pub struct PublishClient {
    root: String,
}
*/

pub struct SubscribeClient {
    consumer: Consumer,
    sender: Sender<Message>,
    root: String,
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
/// ```
/// ```
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl SubscribeClient {
    pub fn new(
        brokers: Vec<String>,
        sender: Sender<Message>,
        root: &str,
        topic: &str,
        group: &str,
        partition: i32,
    ) -> Result<Self, Error> {
        let consumer = match Consumer::from_hosts(brokers)
              .with_topic_partitions(topic.to_owned(), &[partition])
              .with_fallback_offset(FetchOffset::Earliest)
              .with_group(group.to_owned())
              .with_offset_storage(GroupOffsetStorage::Kafka)
              .create() {
                Ok(result) => result,
                Err(err)   => {
                    println!("PANIC {:?}", err);
                    return Err(Error::KafkaInitialize);
                },
            };

        Ok(Self {
            consumer: consumer,
            sender:   sender,
            root:     root.into(),
            topic:    topic.into(),
            group:    group.into(),
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

                    self.sender.send(Message {
                        root: self.root.clone(),
                        topic: self.topic.clone(),
                        group: self.group.clone(),
                        data: data,
                    }).expect("Error writing to mpsc Sender");
                }
                self.consumer.consume_messageset(ms)
                .expect("Error marking MessageSet as consumed.");
            }
            self.consumer.commit_consumed().unwrap();
        }
    }
}
