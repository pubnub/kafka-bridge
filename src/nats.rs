// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Libs
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// NATS
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub struct NATS {
    channel: String,
    authkey: String,
    user: String,
    password: String,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

pub struct NATSMessage {
    pub channel: String,
    pub myId: String,
    pub senderId: String,
    pub data: String,
}

impl NATS {
    pub fn new(
        host: String,
        channel: String,
        authkey: String,
        user: String,
        password: String,
    ) -> Result<NATS, std::io::Error> {
        let mut stream = TcpStream::connect(host).unwrap();
        let subscription = format!("SUB {} 1\r\n", channel);
        let _ = stream.write(subscription.as_bytes());

        Ok(NATS {
            channel: channel.clone(),
            authkey: authkey.clone(),
            user: user.clone(),
            password: password.clone(),
            stream: stream.try_clone().unwrap(),
            reader: BufReader::new(stream),
        })
    }

    pub fn next_message(&mut self) -> Result<NATSMessage, std::io::Error> {
        Ok(loop {
            let mut line = String::new();
            let _len = self.reader.read_line(&mut line);
            let mut detail = line.split_whitespace();
            if Some("MSG") != detail.next() {
                continue;
            }

            let mut data = String::new();
            let _len = self.reader.read_line(&mut data);

            break NATSMessage {
                channel: detail.next().unwrap().to_string(),
                myId: detail.next().unwrap().to_string(),
                senderId: detail.next().unwrap().to_string(),
                data: data.trim().to_string(),
            };
        })

        // if line == MSG
        // get channel
        // get message payload
        // convert to JSON String?

        //reader.read_line()

        //Ok(line.unwrap())

        /*
        match result {
            Ok(data) => {
                println!("{}", data.as_slice().trim());
            }
            Err(e) => {
                println!("error reading: {}", e);
                break;
            }
        }
        */

        //Ok(result)
    }

    pub fn ping(&mut self) -> Result<String, std::io::Error> {
        let ping = format!("PING");
        let _ = self.stream.write(ping.as_bytes());
        let mut line = String::new();

        let _ = self.reader.read_line(&mut line);
        Ok(line)
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
        let result = NATS::new(
            "0.0.0.0:4222".to_string(),
            "demo".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn ping_ok() {
        let result = NATS::new(
            "0.0.0.0:4222".to_string(),
            "demo".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        );
        assert!(result.is_ok());

        let mut nats = result.unwrap();
        let result = nats.ping();
        assert!(result.is_ok());
    }
}
