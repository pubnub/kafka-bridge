// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::{thread, time};

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// NATS
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub struct NATS {
    host: String,
    channel: String,
    _authkey: String,
    _user: String,
    _password: String,
    stream: TcpStream,
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
    ) -> Result<Self, std::io::Error> {
        let stream = Self::connect(host);
        Self::subscribe(stream.try_clone().unwrap(), &channel);

        Ok(Self {
            host: host.into(),
            channel: channel.into(),
            _authkey: authkey.into(),
            _user: user.into(),
            _password: password.into(),
            stream: stream.try_clone().unwrap(),
            reader: BufReader::new(stream),
        })
    }

    fn subscribe(mut stream: TcpStream, channel: &str) {
        let subscription = format!("SUB {} 1\r\n", channel);
        let _ = stream.write(subscription.as_bytes());
        // TODO reconnect on write error.
    }

    fn reconnect(&mut self) {
        thread::sleep(time::Duration::new(1, 0));
        eprintln!("RECONNECTING");
        self.stream = Self::connect(&self.host);
        Self::subscribe(self.stream.try_clone().unwrap(), &self.channel);
        self.reader = BufReader::new(self.stream.try_clone().unwrap());
    }

    fn connect(host: &str) -> TcpStream {
        loop {
            let connection = TcpStream::connect(&host);
            let error = match connection {
                Ok(stream) => return stream,
                Err(error) => error,
            };

            eprintln!(
                "{}",
                json::stringify(object! {
                    "message" => "NATS Host unreachable.",
                    "host" => host,
                    "error" => format!("{}", error),
                })
            );
            thread::sleep(time::Duration::new(5, 0));
        }
    }

    pub fn next_message(&mut self) -> Result<NATSMessage, std::io::Error> {
        Ok(loop {
            let mut line = String::new();
            let status = self.reader.read_line(&mut line);

            if status.is_err() || status.unwrap() == 0 {
                Self::reconnect(self);
                //TODO move to reconnect
                continue;
            }

            let mut detail = line.split_whitespace();
            if Some("MSG") != detail.next() {
                continue;
            }

            let mut data = String::new();
            let status = self.reader.read_line(&mut data);
            if status.is_err() {
                Self::reconnect(self);
                continue;
            }

            // TODO Check length of detail iterator
            // TODO vecotr collection
            break NATSMessage {
                channel: detail.next().unwrap().into(),
                my_id: detail.next().unwrap().into(),
                sender_id: detail.next().unwrap().into(),
                data: data.trim().into(),
            };
        })
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
