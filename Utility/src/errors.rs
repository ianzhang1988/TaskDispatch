// #![deny(unused_mut)]
extern crate zookeeper;

use std::error::Error;
use std::fmt;
use self::zookeeper::ZkError;

#[derive(Debug)]
pub enum RcpError {
    ZkErr(ZkError),
    InternalError,
}

impl fmt::Display for RcpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RcpError::ZkErr(ref err) => write!(f, "Zookeeper Error: {}", err),
            //_ => write!(f, "Eecipe Error: {}", self.description()),
            _ => write!(f, "Eecipe Error: {:?}", self),
        }
    }
}

//impl Error for RcpError {
//    fn cause(&self) -> Option<&Error> {
//        match *self {
//            RcpError::ZkErr(ref err) => Some(err),
//            _ => None,
//        }
//    }
//}

impl From<ZkError> for RcpError {
    fn from(err: ZkError) -> RcpError {
        RcpError::ZkErr(err)
    }
}