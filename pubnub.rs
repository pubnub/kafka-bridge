// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Libs
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use std::io::{Read, Write};
use std::net::TcpStream;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// PubNub
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub struct PubNub {
    //host:   String,
    pubkey: String,
    subkey: String,
    stream: TcpStream,
}

impl PubNub {
    pub fn new(host: String, pubkey: String, subkey: String) -> PubNub {
        PubNub {
     //       host:     host.clone(),
            pubkey: pubkey.clone(),
            subkey: subkey.clone(),
            stream: TcpStream::connect(host).unwrap(),
        }
    }

    pub fn publish(&mut self, channel: String, message: String)
    -> Result<(), std::io::Error> {
        let uri = format!(
            "/publish/{}/{}/0/{}/0/{}",
            self.pubkey,
            self.subkey,
            channel,
            message
        );

        let request = format!("GET {} HTTP/1.1\r\nHost: pubnub\r\n\r\n", uri);
        let _ = self.stream.write(request.as_bytes());
        let _ = self.stream.read(&mut [0; 128]);

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
    fn test_connect_ok() {
        let mut pubnub = PubNub::new(
            "psdsn.pubnub.com:80".to_string(),
            "demo".to_string(),
            "demo".to_string()
        );
        let result = pubnub.publish("demo".to_string(), "1234".to_string());
        assert!(result.is_ok());
    }
}
