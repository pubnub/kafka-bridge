use json;
use std::sync::mpsc::Sender;
use kafka::consumer::Consumer;
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
    Initialize,
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
    ) -> Result<Self, Error> {
        let mut client = kafka::client::KafkaClient::new(brokers);
        let _resources = client.load_metadata_all();
        let consumer   = match Consumer::from_client(client)
            .with_topic(topic.into()).with_group(group.into()).create() {
                Ok(result) => result,
                Err(err)   => panic!(err),
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
