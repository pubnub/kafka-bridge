use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use kafka::error::Error as KafkaError;

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
    client_id: String,
    root: String,
    topic: String,
}


fn consume_messages(
    group: String,
    topic: String,
    brokers: Vec<String>
) -> Result<(), KafkaError> {
    let mut con = try!(
        Consumer::from_hosts(brokers)
            .with_topic(topic)
            .with_group(group)
            .with_fallback_offset(FetchOffset::Earliest)
            .with_offset_storage(GroupOffsetStorage::Kafka)
            .create()
    );

    loop {
        let mss = try!(con.poll());
        if mss.is_empty() {
            println!("No messages available right now.");
            return Ok(());
        }

        for ms in mss.iter() {
            for m in ms.messages() {
                println!("{}:{}@{}: {:?}", ms.topic(), ms.partition(), m.offset, m.value);
            }
            let _ = con.consume_messageset(ms);
        }
        try!(con.commit_consumed());
    }
}
