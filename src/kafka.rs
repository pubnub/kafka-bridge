use kafka::client::KafkaClient;
use kafka::consumer::Consumer;
//use kafka::error::Error as KafkaError;

pub struct Message {
    pub root: String,
    pub topic: String,
    pub my_id: String,
    pub sender_id: String,
    pub data: String,
}

pub struct PublishClient {
    root: String,
}

pub struct SubscribeClient {
    consumer: Consumer,
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
        root: &str,
        topic: &str,
        group: &str,
    ) -> Result<Self, Error> {
        let mut client   = kafka::client::KafkaClient::new(brokers);
        let _resources   = client.load_metadata_all();
        let consumer = Consumer::from_client(client)
            .with_topic(topic.into())
            .with_group(group.into())
            .create().unwrap();

        Ok(Self {
            consumer:   consumer,
            root:       root.into(),
            topic:      topic.into(),
            group:      group.into(),
        })
    }

    /*
    pub fn next_message(&mut self) -> Result<Message, KafkaError> {
        for msg in self.consumer {
            println!("{:?}", msg);
        }
    }
    */
}
