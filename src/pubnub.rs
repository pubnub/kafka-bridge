use crate::socket::{self, Socket};
use json::object;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

pub struct Client {
    socket: Socket<Policy>,
    timetoken: String,
    channel: String,
    publish_key: String,
    subscribe_key: String,
    secret_key: String,
    agent: String,
}

pub struct Message {
    pub channel: String,
    pub data: String,
    pub metadata: String,
    pub timetoken: String,
}

#[derive(Debug)]
pub enum Error {
    Initialize,
    Publish,
    PublishWrite,
    PublishResponse,
    Subscribe,
    MissingChannel,
}

struct Policy {
    host: String,
}

impl socket::Policy for Policy {
    fn host(&self) -> &str {
        &self.host
    }
    fn connected(&self) {
        self.log("PubNub Connected Successfully");
    }
    fn disconnected(&self, error: &str) {
        self.log(error);
    }
    fn unreachable(&self, error: &str) {
        self.log(error);
    }
    fn unwritable(&self, error: &str) {
        self.log(error);
    }
}

impl Policy {
    fn new(host: &str) -> Self {
        Self { host: host.into() }
    }

    fn log(&self, message: &str) {
        println!("{}", json::stringify(object!{
            "message" => message,
            "client" => "NATS",
            "host" => self.host.clone(),
        }));
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # PubNub Client
///
/// This client lib offers publish/subscribe support to PubNub.
///
/// ```no_run
/// use nats_bridge::nats::PubNub;
///
/// let host = "psdsn.pubnub.com:80";
/// let channel = "demo";
/// let publish_key = "demo";
/// let subscribe_key = "demo";
/// let secret_key = "secret";
/// let mut pubnub = PubNub::new(
///     host,
///     channel,
///     publish_key,
///     subscribe_key,
///     secret_key,
///  ).expect("NATS Subscribe Client");
///
/// let result = pubnub.next_message();
/// assert!(result.is_ok());
/// let message = result.expect("Received Message");
/// println!("{} -> {}", message.channel, message.data);
/// ```
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl Client {
    pub fn new(
        host: &str,
        channel: &str,
        publish_key: &str,
        subscribe_key: &str,
        secret_key: &str,
    ) -> Result<Self, Error> {
        let policy = Policy::new(host.into());
        let socket = Socket::new(policy);

        let pubnub = Self {
            socket: socket,
            channel: channel.into(),
            timetoken: "0".into(),
            publish_key: publish_key.into(),
            subscribe_key: subscribe_key.into(),
            secret_key: secret_key.into(),
            agent: "PubNub".into(),
        };

        //pubnub.subscribe();
        Ok(pubnub)
    }

    pub fn publish(
        &mut self,
        channel: &str,
        message: &str,
    ) -> Result<String, Error>{
        let json_message = json::stringify(message);
        let encoded_message = utf8_percent_encode(
            &json_message,
            DEFAULT_ENCODE_SET,
        ).to_string();
        let uri = format!(
            "/publish/{}/{}/0/{}/0/{}?pnsdk={}",
            self.publish_key,
            self.subscribe_key,
            channel,
            encoded_message,
            self.agent
        );

        let request = &format!(
            "GET {} HTTP/1.1\r\nHost: pubnub\r\n\r\n",
            uri,
        );
        let _size = match self.socket.write(request) {
            Ok(size) => size,
            Err(_error) => return Err(Error::PublishWrite),
        };

        // TODO Capture Response Code
        loop {
            let data = match self.socket.readln() {
                Ok(data) => data,
                Err(_error) => return Err(Error::PublishResponse),
            };

            // End of Request
            if data.len() == 2 {
                let timetoken = "TODO TimeToken Here";
                return Ok(timetoken.to_string());
            }
        }
    }

    pub fn next_message() -> Result<Message, Error> {
        Ok(Message{
            channel: "TODO channel".to_string(),
            data: "TODO data".to_string(),
            metadata: "TODO metadata".to_string(),
            timetoken: "TODO timetoken".to_string(),
        })
    }

    /*
    fn subscribe(&mut self) {
        if self.channel.len() <= 0 { return }
        let subscribe_requet = &"";
        self.socket.write(subscribe_requet);
    }

    fn subscribe_string(&mut self) -> String {
        if self.channel.len() <= 0 { "".into() }
        else {
            format!(
                "SUB {channel}{timetoken}\r\n",
                channel=self.channel,
                timetoken=self.timetoken,
            )
        }
    }
    */
}
