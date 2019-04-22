// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
//use std::io::{BufRead, BufReader, Write};
//use std::net::{Shutdown, TcpStream};
//use std::{thread, time};
//use json::object;
//use std::any::Any;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Socket Connection Policy
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub(crate) trait SocketConnectivityPolicy {
    fn connected(&self);
    //fn disconnected(&self);
}
pub(crate) struct SocketPolicy {
    auto_reconnect: bool,
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Socket
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub(crate) struct Socket {
    pub(crate) name: &'static str,
    pub(crate) host: &'static str,
    pub(crate) policy: SocketPolicy,
    //pub(crate) policy: Any,
    //configuration: SocketConfiguration,
    //client: SocketClient,
    //stream: Option<TcpStream>,
    //reader: Option<BufReader<TcpStream>>,
}

/*
pub enum SocketError {
    ConnectionError
}
*/

impl Socket {
    pub fn new(
        name: &'static str,
        host: &'static str,
        policy: SocketPolicy,
    ) -> Self {
        Self {
            name: name,
            host: host,
            policy: policy,
            //client: client,
            //stream: None,
            //reader: None,
        }
    }

    pub fn connect(&mut self) {
    //socket::ConnectivityPolicy::connected(self.policy);
        //self.policy.connected();
        //self.connected();
        //self.stream = Some(TcpStream::connect(self.host));
    }
    /*
    pub fn connect(&mut self) -> Result<(), SocketError> {
        let error = match self.stream {
            Ok(stream) => return stream,
            Err(error) => error,
        };
        self.reader = BufReader::new(stream.try_clone().unwrap());
    }
    */
}
