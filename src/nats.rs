// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use json::object;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::{thread, time};


// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// NATS
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub struct NATS {
    socket: Socket,
    channel: String,
    _authkey: String,
    _user: String,
    _password: String,
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
        let socket = Socket::new("NATS", host);

        let nats = {
            socket: socket,
            channel: channel.into(),
            _authkey: authkey.into(),
            _user: user.into(),
            _password: password.into(),
        };


        Self::subscribe(
        Ok(nats)

    }

    fn subscribe(mut nats: NATS) {
        let subscription = format!("SUB {} 1\r\n", channel);
        let _ = nats.socket.write(subscription);
    }

    pub fn next_message(&mut self) -> Result<NATSMessage, std::io::Error> {
        Ok(loop {

            // create socket lib that is durable and implemetns the common
            // read/write and reconnect on errors.
            let line = self.socket.read_line();

            let mut detail = line.split_whitespace();
            if Some("MSG") != detail.next() {
                continue;
            }

            let data = self.socket.read_line();
            if !data.ok {
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
        let _ = self.socket.write(ping);

        let line = self.socket.read_line();
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
