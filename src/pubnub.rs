use crate::socket::Socket;
use json::JsonValue;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
//use percent_encoding::percent_decode;

pub struct Client {
    publish_socket: Socket,
    subscribe_socket: Socket,
    messages: Vec<Message>,
    timetoken: String,
    channel: String,
    publish_key: String,
    subscribe_key: String,
    _secret_key: String,
    agent: String,
}

pub struct Message {
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
/// # PubNub Client
///
/// This client lib offers publish/subscribe support to PubNub.
///
/// ```no_run
/// use nats_bridge::pubnub::Client;
///
/// let host = "psdsn.pubnub.com:80";
/// let channel = "demo";
/// let publish_key = "demo";
/// let subscribe_key = "demo";
/// let _secret_key = "secret";
/// let mut pubnub = Client::new(
///     host,
///     channel,
///     publish_key,
///     subscribe_key,
///     _secret_key,
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
        _secret_key: &str,
    ) -> Result<Self, Error> {
        let publish_socket = Socket::new(host, "PubNub Publisher");
        let subscribe_socket = Socket::new(host, "PubNub Subscriber");

        let mut pubnub = Self {
            publish_socket,
            subscribe_socket,
            messages: Vec::new(),
            channel: channel.into(),
            timetoken: "0".into(),
            publish_key: publish_key.into(),
            subscribe_key: subscribe_key.into(),
            _secret_key: _secret_key.into(),
            agent: "PubNub".into(),
        };

        match pubnub.subscribe() {
            Ok(()) => Ok(pubnub),
            Err(_error) => Err(Error::Subscribe),
        }
    }

    pub fn publish(
        &mut self,
        channel: &str,
        message: &str,
    ) -> Result<String, Error> {
        //let json_message = json::parse(message);
        let encoded_message = utf8_percent_encode(
            /*&json_message*/ message,
            DEFAULT_ENCODE_SET,
        )
        .to_string();
        let uri = format!(
            "/publish/{}/{}/0/{}/0/{}?pnsdk={}",
            self.publish_key,
            self.subscribe_key,
            channel,
            encoded_message,
            self.agent
        );

        let request =
            format!("GET {} HTTP/1.1\r\nHost: pubnub\r\n\r\n", uri,);
        let _size = match self.publish_socket.write(request) {
            Ok(size) => size,
            Err(_error) => return Err(Error::PublishWrite),
        };

        // Capture and return TimeToken
        let response: JsonValue = match self.http_response("publish") {
            Ok(data) => data,
            Err(_error) => return Err(Error::PublishResponse),
        };
        Ok(response[2].to_string())
    }

    fn http_response(
        &mut self,
        which_socket: &str,
    ) -> Result<JsonValue, Error> {
        let mut body_length: usize = 0;
        let socket = match which_socket {
            "publish" => &mut self.publish_socket,
            "subscribe" => &mut self.subscribe_socket,
            _ => return Err(Error::HTTPResponse),
        };
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

    pub fn next_message(&mut self) -> Result<Message, Error> {
        // Return next saved mesasge
        if !self.messages.is_empty() {
            match self.messages.pop() {
                Some(message) => return Ok(message),
                None => {}
            };
        }

        // Capture
        let response: JsonValue = match self.http_response("subscribe") {
            Ok(data) => data,
            Err(_error) => return Err(Error::SubscribeRead),
        };

        // Save Last Received Netwrok Queue ID
        self.timetoken = response["t"]["t"].to_string();

        // Ask for more messages from network
        self.subscribe()?;

        // Capture Messages in Vec Buffer
        for message in response["m"].members() {
            self.messages.push(Message {
                channel: message["c"].to_string(),
                data: message["d"].to_string(),
                metadata: "TODO metadata".to_string(),
                id: message["p"]["t"].to_string(),
            });
        }

        // Loop
        self.next_message()
    }

    fn subscribe(&mut self) -> Result<(), Error> {
        // Don't subscribe if without a channel
        if self.channel.is_empty() {
            return Ok(());
        }
        let uri = format!(
            "/v2/subscribe/{subscribe_key}/{channel}/0/{timetoken}",
            subscribe_key = self.subscribe_key,
            channel = self.channel,
            timetoken = self.timetoken,
        );
        let request =
            format!("GET {} HTTP/1.1\r\nHost: pubnub\r\n\r\n", uri,);
        match self.subscribe_socket.write(request) {
            Ok(_size) => return Ok(()),
            Err(_error) => return Err(Error::SubscribeWrite),
        };
    }
}
