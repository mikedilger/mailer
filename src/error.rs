// Copyright © 2014 - 2015 by Optimal Computing Limited (of New Zealand)
// This code is licensed under the MIT license (see LICENSE-MIT for details)

use std::error::Error as StdError;
use std::io;
use std::fmt;
use lettre::email::error::Error as EmailError;
use lettre::transport::error::Error as TransportError;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    BodyRequired,
    LettreEmail(EmailError),
    LettreTransport(TransportError),
    SendFailed(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) =>
                format!("{} {}", self.description(), e).fmt(f),
            Error::LettreEmail(ref e) =>
                format!("{} {}", self.description(), e).fmt(f),
            Error::LettreTransport(ref e) =>
                format!("{} {}", self.description(), e).fmt(f),
            Error::SendFailed(ref s) =>
                format!("{} {}", self.description(), s).fmt(f),
            _ =>
                format!("{}", self.description()).fmt(f),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(_) => "I/O Error",
            Error::BodyRequired => "Body Required",
            Error::LettreEmail(_) => "Email Builder Error",
            Error::LettreTransport(_) => "Transport Error",
            Error::SendFailed(_) => "Send Failed",
        }
    }
    fn cause(&self) -> Option<&StdError>
    {
        match *self {
            Error::Io(ref e) => Some(e),
            Error::LettreEmail(ref e) => Some(e),
            Error::LettreTransport(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<EmailError> for Error {
    fn from(err: EmailError) -> Error {
        Error::LettreEmail(err)
    }
}

impl From<TransportError> for Error {
    fn from(err: TransportError) -> Error {
        Error::LettreTransport(err)
    }
}
