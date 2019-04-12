// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Libs
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// NATS
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub struct NATS {
    _channel: String,
    _authkey: String,
    _user: String,
    _password: String,
    _stream: TcpStream,
    reader: BufReader<TcpStream>,
}

pub struct NATSMessage {
    pub channel: String,
    pub my_id: String,
    pub sender_id: String,
    pub data: String,
}

impl NATS {
    pub fn new(
        host: &str,
        channel: &str,
        authkey: &str,
        user: &str,
        password: &str,
    ) -> Result<NATS, std::io::Error> {
        let mut stream = TcpStream::connect(host).unwrap();
        let subscription = format!("SUB {} 1\r\n", channel);
        let _ = stream.write(subscription.as_bytes());

        Ok(NATS {
            _channel: channel.into(),
            _authkey: authkey.into(),
            _user: user.into(),
            _password: password.into(),
            _stream: stream.try_clone().unwrap(),
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
                channel: detail.next().unwrap().into(),
                my_id: detail.next().unwrap().into(),
                sender_id: detail.next().unwrap().into(),
                data: data.trim().into(),
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

    #[cfg(test)]
    pub fn ping(&mut self) -> Result<String, std::io::Error> {
        let ping = format!("PING");
        let _ = self._stream.write(ping.as_bytes());
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
        let result = NATS::new("0.0.0.0:4222", "demo", "", "", "");
        assert!(result.is_ok());
    }

    #[test]
    fn ping_ok() {
        let result = NATS::new("0.0.0.0:4222", "demo", "", "", "");
        assert!(result.is_ok());

        let mut nats = result.unwrap();
        let result = nats.ping();
        assert!(result.is_ok());
    }
}
