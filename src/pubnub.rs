// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use json;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// PubNub
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub struct PubNub {
    pubkey: String,
    subkey: String,
    _seckey: String,
    agent: String,
    _host: String,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

/*
pub struct PubNubMessage {
    pub channel: String,
    pub data: json::JsonValue,
    pub meta: json::JsonValue,
    pub store: String,
    pub replicate: String,
}
*/

impl PubNub {
    pub fn new(
        host: &str,
        pubkey: &str,
        subkey: &str,
        seckey: &str,
    ) -> Result<PubNub, std::io::Error> {
        let stream;

        match TcpStream::connect(&host) {
            Ok(data) => {
                stream = data;
            }
            Err(error) => {
                panic!("Error connecting PubNub: {error}", error = error);
            }
        }

        Ok(PubNub {
            pubkey: pubkey.into(),
            subkey: subkey.into(),
            _seckey: seckey.into(),
            agent: "nats-bridge".to_string(),
            _host: host.into(),
            stream: stream.try_clone().unwrap(),
            reader: BufReader::new(stream),
        })
    }

    /*
    fn connect(host: &str) -> TcpStream {
        loop {
            let connection = TcpStream::connect(&host);
            if connection.is_ok() { return connection.unwrap() }

            let error = connection.unwrap_err();
            println!("{}", json::stringify(object!{
                "message" => "PubNub API Reconnecting.",
                "host" => host,
                "error" => format!("{}", error),
            }));
            thread::sleep(time::Duration::new(5, 0));
        }
    }
    */

    pub fn publish(&mut self, channel: &str, message: &str) -> Result<(), std::io::Error> {
        let json_message = json::stringify(message);
        let uri = format!(
            "/publish/{}/{}/0/{}/0/{}?pnsdk={}",
            self.pubkey, self.subkey, channel, json_message, self.agent
        );

        let request = format!("GET {} HTTP/1.1\r\nHost: pubnub\r\n\r\n", uri);
        let _ = self.stream.write(request.as_bytes());

        loop {
            let mut buf = String::new();
            let count = self.reader.read_line(&mut buf).unwrap();
            if count == 2 {
                break;
            }
        }

        Ok(())
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Tests
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_ok() {
        let host = "psdsn.pubnub.com:80";
        let result = PubNub::new(host, "demo", "demo", "secret");
        assert!(result.is_ok());
    }

    #[test]
    fn publish_ok() {
        let host = "psdsn.pubnub.com:80";
        let result = PubNub::new(host, "demo", "demo", "secret");
        assert!(result.is_ok());

        let mut pubnub = result.unwrap();
        let result = pubnub.publish("demo", "123");
        assert!(result.is_ok());
    }
}
