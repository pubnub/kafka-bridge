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
    authkey: String,
    user: String,
    password: String,
    stream: TcpStream,
    //reader: BufRead
}

impl NATS {
    pub fn new(host: String, authkey: String, user: String, password: String)
    -> Result<NATS, std::io::Error> {
        //let mut stream = TcpStream::connect(host).unwrap();
//        let mut reader = BufRead::new(stream);

        Ok(NATS {
            authkey: authkey.clone(),
            user: user.clone(),
            password: password.clone(),
            stream: TcpStream::connect(host).unwrap(),
            //reader: reader
        })
    }

    pub fn subscribe(&mut self, channel: String)
    -> Result<String, std::io::Error> {
        let subscription = format!("SUB {}", channel);
        //let mut buffer = vec![];

        let _ = self.stream.write(subscription.as_bytes());
        let mut reader = BufReader::new(self.stream.try_clone().unwrap());
        let mut line = String::new();
        let _len = reader.read_line(&mut line);

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
        let mut reader = BufReader::new(self.stream.try_clone().unwrap());
        let mut line = String::new();

        reader.read_line(&mut line);
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
