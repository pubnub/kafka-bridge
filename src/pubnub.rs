use crate::socket::Socket;
use json::JsonValue;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

pub struct SubscribeClient {
    socket: Socket,
    root: String,
    channel: String,
    messages: Vec<Message>,
    timetoken: String,
    subscribe_key: String,
    _secret_key: String,
    agent: String,
}

pub struct PublishClient {
    socket: Socket,
    root: String,
    publish_key: String,
    subscribe_key: String,
    _secret_key: String,
    agent: String,
}

pub struct Message {
    pub root: String,
    pub channel: String,
    pub data: String,
    pub metadata: String,
    pub id: String,
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
    MissingChannel,
    HTTPResponse,
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// HTTP Response Reader/Parser
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
fn http_response(socket: &mut Socket) -> Result<JsonValue, Error> {
    let mut body_length: usize = 0;
    loop {
        let data = match socket.readln() {
            Ok(data) => data,
            Err(_error) => return Err(Error::HTTPResponse),
        };

        // Capture Content Length of Payload
        if body_length == 0 && data.contains("Content-Length") {
            let result = match data.split_whitespace().nth(1) {
                Some(length) => length.parse(),
                None => return Err(Error::HTTPResponse),
            };
            body_length = match result {
                Ok(length) => length,
                Err(_error) => return Err(Error::HTTPResponse),
            };
        }

        // End of Headers
        if data.len() == 2 {
            let paylaod = match socket.read(body_length) {
                Ok(data) => data,
                Err(_error) => return Err(Error::HTTPResponse),
            };
            match json::parse(&paylaod) {
                Ok(response) => return Ok(response),
                Err(_error) => return Err(Error::HTTPResponse),
            };
        }
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # PubNub Subscriber Client
///
/// This client lib offers subscribe support to PubNub.
///
/// ```no_run
/// use edge_messaging_platform::pubnub::SubscribeClient;
///
/// let host = "psdsn.pubnub.com:80";
/// let channel = "demo";
/// let root = "";
/// let publish_key = "demo";
/// let subscribe_key = "demo";
/// let _secret_key = "secret";
/// let agent = "nats-edge_messaging_platform";
/// let mut pubnub = SubscribeClient::new(
///     host,
///     root,
///     channel,
///     subscribe_key,
///     _secret_key,
///     agent,
///  ).expect("NATS Subscribe Client");
///
/// let result = pubnub.next_message();
/// assert!(result.is_ok());
/// let message = result.expect("Received Message");
/// println!("{} -> {}", message.channel, message.data);
/// ```
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl SubscribeClient {
    pub fn new(
        host: &str,
        root: &str,
        channel: &str,
        subscribe_key: &str,
        _secret_key: &str,
        agent: &str,
    ) -> Result<Self, Error> {
        let socket = Socket::new(host, agent, 30);

        let mut pubnub = Self {
            socket,
            root: root.into(),
            channel: channel.into(),
            messages: Vec::new(),
            timetoken: "0".into(),
            subscribe_key: subscribe_key.into(),
            _secret_key: _secret_key.into(),
            agent: agent.into(),
        };

        match pubnub.subscribe() {
            Ok(()) => Ok(pubnub),
            Err(_error) => Err(Error::Subscribe),
        }
    }

    pub fn next_message(&mut self) -> Result<Message, Error> {
        // Return next saved mesasge
        if let Some(message) = self.messages.pop() {
            return Ok(message);
        }

        // Capture
        let response: JsonValue = match http_response(&mut self.socket) {
            Ok(data) => data,
            Err(_error) => {
                // Already returning an error, would you like another?
                self.subscribe().is_err();

                // Return first error
                return Err(Error::SubscribeRead);
            }
        };

        // Save Last Received Netwrok Queue ID
        self.timetoken = response["t"]["t"].to_string();

        // Capture Messages in Vec Buffer
        for message in response["m"].members() {
            // Carefully deal with ROOT.CHANNEL
            let source = message["c"].to_string();
            let meta = message["u"].to_string();

            let channel = if self.root.is_empty() {
                source
            } else {
                source[self.root.len() + 1..].to_string()
            };

            self.messages.push(Message {
                root: self.root.to_string(),
                channel,
                data: message["d"].to_string(),
                metadata: meta,
                id: message["p"]["t"].to_string(),
            });
        }

        // Ask for more messages from network
        match self.subscribe() {
            Ok(()) => self.next_message(),
            Err(_) => Err(Error::SubscribeRead),
        }
    }

    fn subscribe(&mut self) -> Result<(), Error> {
        // Don't subscribe if without a channel
        if self.channel.is_empty() {
            return Err(Error::MissingChannel);
        }
        let channel = if self.root.is_empty() {
            self.channel.to_string()
        } else {
            format!(
                "{root}.{channel}",
                channel = self.channel,
                root = self.root
            )
        };
        let uri = format!(
            "/v2/subscribe/{subscribe_key}/{channel}/0/{timetoken}?pnsdk={agent}&filter-expr={filter}",
            //"/v2/subscribe/{subscribe_key}/{channel}/0/{timetoken}?pnsdk={agent}",
            subscribe_key = self.subscribe_key,
            channel = channel,
            timetoken = self.timetoken,
            agent = self.agent,
            filter = "source%22%21%3D%22%27NATS%27".to_string(),
        );
        let request =
            format!("GET {} HTTP/1.1\r\nHost: pubnub\r\n\r\n", uri,);
        match self.socket.write(request) {
            Ok(_size) => Ok(()),
            Err(_error) => Err(Error::SubscribeWrite),
        }
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # PubNub Publisher Client
///
/// This client lib offers publish support to PubNub.
///
/// ```no_run
/// use edge_messaging_platform::pubnub::PublishClient;
///
/// let host = "psdsn.pubnub.com:80";
/// let root = "";
/// let channel = "demo";
/// let publish_key = "demo";
/// let subscribe_key = "demo";
/// let _secret_key = "secret";
/// let agent = "nats-edge_messaging_platform";
/// let mut pubnub = PublishClient::new(
///     host,
///     root,
///     publish_key,
///     subscribe_key,
///     _secret_key,
///     agent,
///  ).expect("NATS Subscribe Client");
///
/// let result = pubnub.publish(channel, "data");
/// assert!(result.is_ok());
/// ```
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl PublishClient {
    pub fn new(
        host: &str,
        root: &str,
        publish_key: &str,
        subscribe_key: &str,
        secret_key: &str,
        agent: &str,
    ) -> Result<Self, Error> {
        let socket = Socket::new(host, agent, 5);

        Ok(Self {
            socket,
            root: root.into(),
            publish_key: publish_key.into(),
            subscribe_key: subscribe_key.into(),
            _secret_key: secret_key.into(),
            agent: agent.into(),
        })
    }

    pub fn publish(
        &mut self,
        channel: &str,
        message: &str,
    ) -> Result<String, Error> {
        let encoded_message =
            utf8_percent_encode(message, DEFAULT_ENCODE_SET).to_string();
        let channel = if self.root.is_empty() {
            channel.to_string()
        } else {
            format!("{root}.{channel}", channel = channel, root = self.root)
        };
        let uri = format!(
            "/publish/{}/{}/0/{}/0/{}?pnsdk={pnsdk}&meta={meta}",
            self.publish_key,
            self.subscribe_key,
            channel,
            encoded_message,
            pnsdk=self.agent,
            meta="{\"source\":\"NATS\"}"
        );

        let request =
            format!("GET {} HTTP/1.1\r\nHost: pubnub\r\n\r\n", uri,);
        let _size = match self.socket.write(request) {
            Ok(size) => size,
            Err(_error) => return Err(Error::PublishWrite),
        };

        // Capture and return TimeToken
        let response: JsonValue = match http_response(&mut self.socket) {
            Ok(data) => data,
            Err(_error) => return Err(Error::PublishResponse),
        };
        Ok(response[2].to_string())
    }
}
