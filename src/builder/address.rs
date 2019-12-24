use std::convert::TryFrom;
use std::fmt;

use crate::builder::{
    header::{Header, ToFoldedHeader},
    rfc5322::MIME_LINE_LENGTH,
};
use crate::{error::Error, EmailAddress};

/// Represents an RFC 5322 mailbox
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Mailbox {
    /// The given name for this address
    pub name: Option<String>,
    /// The mailbox address
    pub address: EmailAddress,
}

impl Mailbox {
    /// Create a new Mailbox without a display name
    pub fn new(address: EmailAddress) -> Mailbox {
        Mailbox {
            name: None,
            address,
        }
    }

    /// Create a new Mailbox with a display name
    pub fn new_with_name(name: String, address: EmailAddress) -> Mailbox {
        Mailbox {
            name: Some(name),
            address,
        }
    }
}

impl From<EmailAddress> for Mailbox {
    fn from(addr: EmailAddress) -> Self {
        Mailbox::new(addr)
    }
}

impl fmt::Display for Mailbox {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.name {
            Some(ref name) => {
                // FIXME do not always quote
                if name.is_ascii() {
                    write!(fmt, "\"{}\" <{}>", name, self.address)
                } else {
                    write!(fmt, "{} <{}>", Header::encode_rfc2047(name), self.address)
                }
            }
            None => write!(fmt, "<{}>", self.address),
        }
    }
}

impl<'a> TryFrom<&'a str> for Mailbox {
    type Error = Error;

    fn try_from(mailbox: &'a str) -> Result<Self, Error> {
        let email = EmailAddress::new(mailbox.into())?;
        Ok(Mailbox::new(email))
    }
}

impl TryFrom<String> for Mailbox {
    type Error = Error;

    fn try_from(mailbox: String) -> Result<Self, Error> {
        let email = EmailAddress::new(mailbox)?;
        Ok(Mailbox::new(email))
    }
}

impl<S: Into<String>, T: Into<String>> TryFrom<(S, T)> for Mailbox {
    type Error = Error;

    fn try_from(header: (S, T)) -> Result<Self, Error> {
        let (address, name) = header;
        let email = EmailAddress::new(address.into())?;
        Ok(Mailbox::new_with_name(name.into(), email))
    }
}

impl ToFoldedHeader for Vec<Mailbox> {
    fn to_folded_header(start_pos: usize, value: Vec<Mailbox>) -> String {
        let mut header = String::new();

        let mut line_len = start_pos;

        for addr in value.iter() {
            let addr_str = format!("{}, ", addr);

            if line_len + addr_str.len() > MIME_LINE_LENGTH {
                // Adding this would cause a wrap, so wrap before!
                header.push_str("\r\n\t");
                line_len = 0;
            }
            line_len += addr_str.len();
            header.push_str(&addr_str[..]);
        }

        // Clear up the final ", "
        let real_len = header.len() - 2;
        header.truncate(real_len);

        header
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::header::Header;

    #[test]
    fn test_address_to_string() {
        let addr = Mailbox::try_from("foo@example.org".to_string()).unwrap();
        assert_eq!(addr.to_string(), "<foo@example.org>".to_string());

        let name_addr =
            Mailbox::try_from(("foo@example.org".to_string(), "Joe Blogs".to_string())).unwrap();
        assert_eq!(
            name_addr.to_string(),
            "\"Joe Blogs\" <foo@example.org>".to_string()
        );
    }

    #[test]
    fn test_to_header_line_wrap() {
        let addresses = vec![
            Mailbox::try_from(("joe@example.org".to_string(), "Joe Blogs".to_string())).unwrap(),
            Mailbox::try_from(("john@example.org".to_string(), "John Doe".to_string())).unwrap(),
            Mailbox::try_from((
                "mafia_black@example.org".to_string(),
                "Mr Black".to_string(),
            ))
            .unwrap(),
        ];

        let header = Header::new_with_value("To".to_string(), addresses);
        assert_eq!(
            &header.to_string()[..],
            "To: \"Joe Blogs\" <joe@example.org>, \"John Doe\" <john@example.org>, \r\n\
             \t\"Mr Black\" <mafia_black@example.org>"
        );
    }
}
