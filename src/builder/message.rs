use super::header::{Header, HeaderMap};
use super::mimeheaders::{MimeContentType, MimeContentTypeHeader};
use super::rfc5322::Rfc5322Builder;

use std::collections::HashMap;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

const BOUNDARY_LENGTH: usize = 30;

/// Marks the type of a multipart message
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum MimeMultipartType {
    /// Entries which are independent.
    ///
    /// This value is the default.
    ///
    /// As defined by Section 5.1.3 of RFC 2046
    Mixed,
    /// Entries which are interchangeable, such that the system can choose
    /// whichever is "best" for its use.
    ///
    /// As defined by Section 5.1.4 of RFC 2046
    Alternative,
    /// Entries are (typically) a collection of messages.
    ///
    /// As defined by Section 5.1.5 of RFC 2046
    Digest,
    /// Entry order does not matter, and could be displayed simultaneously.
    ///
    /// As defined by Section 5.1.6 of RFC 2046
    Parallel,
}

impl MimeMultipartType {
    /// Returns a MimeContentType that represents this multipart type.
    pub fn to_content_type(&self) -> MimeContentType {
        let multipart = "multipart".to_string();
        match *self {
            MimeMultipartType::Mixed => (multipart, "mixed".to_string()),
            MimeMultipartType::Alternative => (multipart, "alternative".to_string()),
            MimeMultipartType::Digest => (multipart, "digest".to_string()),
            MimeMultipartType::Parallel => (multipart, "parallel".to_string()),
        }
    }
}

/// Represents a MIME message
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct MimeMessage {
    /// The headers for this message
    pub headers: HeaderMap,

    /// The content of this message
    ///
    /// Keep in mind that this is the undecoded form, so may be quoted-printable
    /// or base64 encoded.
    pub body: String,

    /// The MIME multipart message type of this message, or `None` if the message
    /// is not a multipart message.
    pub message_type: Option<MimeMultipartType>,

    /// The sub-messages of this message
    pub children: Vec<MimeMessage>,

    /// The boundary used for MIME multipart messages
    ///
    /// This will always be set, even if the message only has a single part
    pub boundary: String,
}

impl MimeMessage {
    pub fn new_blank_message() -> MimeMessage {
        let mut rng = thread_rng();

        MimeMessage {
            headers: HeaderMap::new(),
            body: "".to_string(),
            message_type: None,
            children: Vec::new(),
            boundary: std::iter::repeat(())
                .map(|()| rng.sample(Alphanumeric))
                .take(BOUNDARY_LENGTH)
                .collect(),
        }
    }

    /// Update the headers on this message based on the internal state.
    ///
    /// When certain properties of the message are modified, the headers
    /// used to represent them are not automatically updated.
    /// Call this if these are changed.
    pub fn update_headers(&mut self) {
        if self.children.len() > 0 && self.message_type.is_none() {
            // This should be a multipart message, so make it so!
            self.message_type = Some(MimeMultipartType::Mixed);
        }

        if let Some(message_type) = self.message_type {
            // We are some form of multi-part message, so update our
            // Content-Type header.
            let mut params = HashMap::new();
            params.insert("boundary".to_string(), self.boundary.clone());
            let ct_header = MimeContentTypeHeader {
                content_type: message_type.to_content_type(),
                params: params,
            };
            self.headers
                .insert(Header::new_with_value("Content-Type".to_string(), ct_header).unwrap());
        }
    }

    pub fn as_string(&self) -> String {
        let mut builder = Rfc5322Builder::new();

        for header in self.headers.iter() {
            builder.emit_folded(&header.to_string()[..]);
            builder.emit_raw("\r\n");
        }
        builder.emit_raw("\r\n");

        self.as_string_without_headers_internal(builder)
    }

    fn as_string_without_headers_internal(&self, mut builder: Rfc5322Builder) -> String {
        builder.emit_raw(&format!("{}\r\n", self.body)[..]);

        if self.children.len() > 0 {
            for part in self.children.iter() {
                builder.emit_raw(&format!("--{}\r\n{}\r\n", self.boundary, part.as_string())[..]);
            }

            builder.emit_raw(&format!("--{}--\r\n", self.boundary)[..]);
        }

        builder.result().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multipart_type_to_content_type() {
        let multipart = "multipart".to_string();

        assert_eq!(
            MimeMultipartType::Mixed.to_content_type(),
            (multipart.clone(), "mixed".to_string())
        );
        assert_eq!(
            MimeMultipartType::Alternative.to_content_type(),
            (multipart.clone(), "alternative".to_string())
        );
        assert_eq!(
            MimeMultipartType::Digest.to_content_type(),
            (multipart.clone(), "digest".to_string())
        );
        assert_eq!(
            MimeMultipartType::Parallel.to_content_type(),
            (multipart.clone(), "parallel".to_string())
        );
    }

    #[test]
    fn test_boundary_generation() {
        let message = MimeMessage::new_blank_message();
        // This is random, so we can only really check that it's the expected length
        assert_eq!(message.boundary.len(), super::BOUNDARY_LENGTH);
    }
}
