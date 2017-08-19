//! Lettre is a mailer written in Rust. It provides a simple email builder and several transports.
//!
//! This mailer contains the available transports for your emails. To be sendable, the
//! emails have to implement `SendableEmail`.
//!

#![deny(unsafe_code, unstable_features)]

#[macro_use]
extern crate log;
extern crate base64;
#[cfg(feature = "crammd5-auth")]
extern crate hex;
#[cfg(feature = "crammd5-auth")]
extern crate crypto;
extern crate bufstream;
extern crate native_tls;
#[cfg(feature = "file-transport")]
extern crate serde_json;
#[cfg(feature = "serde-impls")]
extern crate serde;
#[cfg(feature = "serde-impls")]
#[macro_use]
extern crate serde_derive;

pub mod smtp;
pub mod sendmail;
pub mod stub;
#[cfg(feature = "file-transport")]
pub mod file;

#[cfg(feature = "file-transport")]
pub use file::FileEmailTransport;
pub use sendmail::SendmailTransport;
pub use smtp::ClientSecurity;
pub use smtp::SmtpTransport;
pub use smtp::client::net::ClientTlsParameters;

use std::str;
use std::fmt::{self, Display, Formatter};
use std::io::{self, BufReader, Read};

/// Email address
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
pub struct EmailAddress(pub String);

impl Display for EmailAddress {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl EmailAddress {
    /// Creates a new email address
    pub fn new(address: String) -> EmailAddress {
        EmailAddress(address)
    }
}

/// Email sendable by an SMTP client
pub trait SendableEmail<'a, T: Read + 'a> {
    /// To
    fn to(&self) -> Vec<EmailAddress>;
    /// From
    fn from(&self) -> EmailAddress;
    /// Message ID, used for logging
    fn message_id(&self) -> String;
    /// Message content
    fn message(&'a self) -> Box<T>;
}

/// Transport method for emails
pub trait EmailTransport<'a, U: Read + 'a, V> {
    /// Sends the email
    fn send<T: SendableEmail<'a, U> + 'a>(&mut self, email: &'a T) -> V;
    /// Close the transport explicitly
    fn close(&mut self);
    /// Reset the transport state
    fn reset(&mut self);
}

/// Minimal email structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
pub struct SimpleSendableEmail {
    /// To
    to: Vec<EmailAddress>,
    /// From
    from: EmailAddress,
    /// Message ID
    message_id: String,
    /// Message content
    message: Vec<u8>,
}

impl SimpleSendableEmail {
    /// Returns a new email
    pub fn new(
        from_address: EmailAddress,
        to_addresses: Vec<EmailAddress>,
        message_id: String,
        message: String,
    ) -> SimpleSendableEmail {
        SimpleSendableEmail {
            from: from_address,
            to: to_addresses,
            message_id: message_id,
            message: message.into_bytes(),
        }
    }
}

impl<'a> SendableEmail<'a, &'a [u8]> for SimpleSendableEmail {
    fn to(&self) -> Vec<EmailAddress> {
        self.to.clone()
    }

    fn from(&self) -> EmailAddress {
        self.from.clone()
    }

    fn message_id(&self) -> String {
        self.message_id.clone()
    }

    fn message(&'a self) -> Box<&[u8]> {
        Box::new(self.message.as_slice())
    }
}
