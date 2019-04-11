// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Libs
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use std::io::{BufReader, Write};
use std::io::BufRead;
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
    reader: BufReader<TcpStream>
}

impl NATS {
    pub fn new(
        host: String,
        channel: String,
        authkey: String,
        user: String,
        password: String
    ) -> Result<NATS, std::io::Error> {
        let mut stream = TcpStream::connect(host).unwrap();
        let subscription = format!("SUB {}", channel);
        let _ = stream.write(subscription.as_bytes());

        Ok(NATS {
            channel: channel.clone(),
            authkey: authkey.clone(),
            user: user.clone(),
            password: password.clone(),
            stream: stream.try_clone().unwrap(),
            reader: BufReader::new(stream)
        })
    }

    pub fn listen(&mut self)
    -> Result<String, std::io::Error> {
        let mut line = String::new();
        let _len = self.reader.read_line(&mut line);
        Ok(line)

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

    pub fn ping(&mut self)
    -> Result<String, std::io::Error> {
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
            "".to_string()
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
            "".to_string()
        );
        assert!(result.is_ok());

        let mut nats = result.unwrap();
        let result = nats.ping();
        assert!(result.is_ok());
    }
}
