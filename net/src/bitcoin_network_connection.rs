use std::cell::RefCell;
use std::fmt::{self, Debug};
use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::time::{Duration};

use circular::Buffer;
use nom::{ErrorKind, Err, IResult, Needed};

use message::Message;
use parser::message;

pub struct BitcoinNetworkConnection {
    host: String,
    buffer: RefCell<Buffer>,
    socket: RefCell<TcpStream>,
    /// Bytes that we need to parse the next message
    needed: RefCell<usize>,
    bad_messages: RefCell<usize>,
}

pub enum BitcoinNetworkError {
    BadBytes,
    Closed,
    ReadTimeout,
}

impl BitcoinNetworkConnection {
    pub fn new(host: String) -> Result<BitcoinNetworkConnection, Error> {
        info!("Trying to initialize connection to {}", host);
        let socket = TcpStream::connect(&host)?;
        socket.set_read_timeout(Some(Duration::from_secs(2)))?;
        // .expect("set_read_timeout call failed");
        socket.set_write_timeout(Some(Duration::from_secs(2)))?;
        
        Ok(BitcoinNetworkConnection {
            host: host,
            // Allocate a buffer with 4MB of capacity
            buffer: RefCell::new(Buffer::with_capacity(1024 * 1024 * 4)),
            socket: RefCell::new(socket),
            needed: RefCell::new(0),
            bad_messages: RefCell::new(0),
        })
    }

    pub fn with_stream(host: String, socket: TcpStream) -> Result<BitcoinNetworkConnection, Error> {
        socket.set_read_timeout(Some(Duration::from_secs(2)))?;
        // .expect("set_read_timeout call failed");
        socket.set_write_timeout(Some(Duration::from_secs(2)))?;
        
        Ok(BitcoinNetworkConnection {
            host: host,
            // Allocate a buffer with 4MB of capacity
            buffer: RefCell::new(Buffer::with_capacity(1024 * 1024 * 4)),
            socket: RefCell::new(socket),
            needed: RefCell::new(0),
            bad_messages: RefCell::new(0),
        })
    }

        // fn send(&mut self, message: Message) -> Result<(), Error> {
    //       trace!("{} About to write: {:?}", self.host, message);
    //       let written = self.socket.write(&message.encode())?;
    //       trace!("{} Written: {:}", self.host, written);
    //       Ok(())
    //   }

    pub fn try_send(&self, message: Message) -> Result<(), Error> {
          trace!("{} About to write: {:?}", self.host, message);
          let written = self.socket.borrow_mut().write(&message.encode(false))?;
          trace!("{} Written: {:}", self.host, written);
          Ok(())
    }

    pub fn recv(&self) -> Message {
        unimplemented!()
    }

    pub fn try_recv(&self) -> Option<Result<Message, BitcoinNetworkError>> {
        let len = self.buffer.borrow().available_data();
        trace!("[{}] Buffer len: {}", self.host, len);
        if let Some(message) = self.try_parse() {
            return Some(message);
        }

        match self.read() {
            Ok(_) => {},
            Err(e) => {
                return Some(Err(e))
            }
        }
        // If we haven't received any more data
        let read = self.buffer.borrow().available_data();
        if read < *self.needed.borrow() || read == 0 || read == len {
            return None;
        }

        // let _ = self.sender.try_send(ClientMessage::Alive(self.id));

        if let Some(message) = self.try_parse() {
            return Some(message);
        }
        None
    }

    fn try_parse(&self) -> Option<Result<Message, BitcoinNetworkError>> {
        let available_data = self.buffer.borrow().available_data();
        if available_data == 0 {
            return None;
        }
        let mut trim = false;
        let mut consume = 0;
        let parsed = match message(&self.buffer.borrow().data(), &self.host) {
            IResult::Done(remaining, msg) => Some((msg, remaining.len())),
            IResult::Incomplete(len) => {
                if let Needed::Size(s) = len {
                    *self.needed.borrow_mut() = s;
                }
                None
            }
            IResult::Error(e) => {
                match e {
                    Err::Code(ErrorKind::Custom(i)) => {
                        warn!("{} Gave us bad data!", self.host);
                        consume = i;
                        trim = true;
                    }
                    _ => {
                        consume = 1;
                        trim = true;
                    }
                }
                None
            }
        };
        if let Some((message, remaining_len)) = parsed {
            (self.buffer.borrow_mut()).consume(available_data - remaining_len);
            *self.needed.borrow_mut() = 0;
            return Some(Ok(message));
        }

        self.buffer.borrow_mut().consume(consume as usize);
        if trim {
            *self.bad_messages.borrow_mut() += 1;
            return Some(Err(BitcoinNetworkError::BadBytes))
        }
        None
    }

    fn read(&self) -> Result<(),BitcoinNetworkError> {
        let mut buff = [0; 8192];
        let read = match self.socket.borrow_mut().read(&mut buff) {
            Ok(r) => {
                if r == 0 {
                    // return Err(())?
                    return Err(BitcoinNetworkError::Closed)
                }
                r
            },
            Err(_e) => {
                return Err(BitcoinNetworkError::ReadTimeout);
            }
        };
        // if read == 0 {
        //     return;
        // }
        trace!("[{} / {}] Read: {}, Need: {}",
               self.buffer.borrow().available_data(),
               self.buffer.borrow().capacity(),
               read,
               *self.needed.borrow());
        self.buffer.borrow_mut().grow(read);
        let _ = self.buffer.borrow_mut().write(&buff[0..read]);

        if *self.needed.borrow() >= read {
            *self.needed.borrow_mut() -= read;
        } else {
            *self.needed.borrow_mut() = 0;
            return Ok(());
        }

     Ok(())
    }
}

impl Debug for BitcoinNetworkConnection {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f,
               r"BitcoinNetworkConnection {{
    ,
    }}",)
    }
}
