use crate::socket::Socket;
use json;
use json::JsonValue;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
//use percent_encoding::percent_decode;

pub struct Client {
    publish_socket: Socket,
    subscribe_socket: Socket,
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
    pub timetoken: String,
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
/// //let result = pubnub.next_message();
/// //assert!(result.is_ok());
/// //let message = result.expect("Received Message");
/// //println!("{} -> {}", message.channel, message.data);
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
        let publish_socket = Socket::new(host);
        let subscribe_socket = Socket::new(host);

        let mut pubnub = Self {
            publish_socket: publish_socket,
            subscribe_socket: subscribe_socket,
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
        let json_message = json::stringify(message);
        let encoded_message =
            utf8_percent_encode(&json_message, DEFAULT_ENCODE_SET)
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
            &format!("GET {} HTTP/1.1\r\nHost: pubnub\r\n\r\n", uri,);
        let _size = match self.publish_socket.write(request) {
            Ok(size) => size,
            Err(_error) => {
                self.publish_socket.reconnect();
                return Err(Error::PublishWrite);
            }
        };

        // Capture and return TimeToken
        let response: JsonValue = match self.http_response("publish") {
            Ok(data) => data,
            // TODO _error ( RECONNECT!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!)yypppp
            // TODO _error ( RECONNECT!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!)yypppp
            // TODO _error ( RECONNECT!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!)yypppp
            // TODO _error ( RECONNECT!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!)yypppp
            // TODO _error ( RECONNECT!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!)yypppp
            // TODO _error ( RECONNECT!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!)yypppp
            // TODO _error ( RECONNECT!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!)yypppp
            // TODO _error ( RECONNECT!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!)yypppp
            // TODO _error ( RECONNECT!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!)yypppp
            // TODO _error ( RECONNECT!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!)yypppp
            // EVERWHERE>...... _error
            // EVERWHERE>...... _error
            // EVERWHERE>...... _error
            // EVERWHERE>...... _error
            // EVERWHERE>...... _error
            // EVERWHERE>...... _error
            // EVERWHERE>...... _error
            // EVERWHERE>...... _error
            // EVERWHERE>...... _error
            // EVERWHERE>...... _error
            // EVERWHERE>...... _error
            Err(_error) => return Err(Error::PublishResponse),
        };
        Ok(response[2].to_string())

    }

    fn http_response(&mut self, which_socket: &str) -> Result<JsonValue, Error> {
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
                let result = match data.split_whitespace().skip(1).next() {
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
                let response = match json::parse(&paylaod) {
                    Ok(response) => response,
                    Err(_error) => return Err(Error::HTTPResponse),
                };
                return Ok(response);
            }
        }
    }

    pub fn next_message() -> Result<Message, Error> {
        // TODO if EOF, self.subscribe() again
        Ok(Message {
            channel: "TODO channel".to_string(),
            data: "TODO data".to_string(),
            metadata: "TODO metadata".to_string(),
            timetoken: "TODO timetoken".to_string(),
        })
    }

    fn subscribe(&mut self) -> Result<(), Error> {
        if self.channel.len() <= 0 {
            return Ok(());
        }
        let uri = format!(
            "/v2/subscribe/{subscribe_key}/{channel}/0/{timetoken}",
            subscribe_key = self.subscribe_key,
            channel = self.channel,
            timetoken = self.timetoken,
        );
        let request =
            &format!("GET {} HTTP/1.1\r\nHost: pubnub\r\n\r\n", uri,);
        let _size = match self.subscribe_socket.write(request) {
            Ok(_size) => return Ok(()),
            Err(_error) => {
                self.subscribe_socket.reconnect();
                return Err(Error::SubscribeWrite);
            }
        };
    }
}
