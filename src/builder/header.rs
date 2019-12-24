use std::fmt;
use std::ops::Deref;
use std::slice::Iter as SliceIter;
use std::sync::Arc;

const MAX_ENCODED_WORD_LEN: usize = 75;

/// Trait for converting from a Rust type into a Header value.
pub trait ToHeader {
    /// Turn the `value` into a String suitable for being used in
    /// a message header.
    ///
    /// Returns None if the value cannot be stringified.
    fn to_header(value: Self) -> String;
}

/// Trait for converting from a Rust time into a Header value
/// that handles its own folding.
///
/// Be mindful that this trait does not mean that the value will
/// not be folded later, rather that the type returns a value that
/// should not be folded, given that the header value starts so far
/// in to a line.
pub trait ToFoldedHeader {
    fn to_folded_header(start_pos: usize, value: Self) -> String;
}
/// Creates a new Header for the given `name` and `value`,
/// as converted through the `ToHeader` or `ToFoldedHeader` trait.
///
/// Returns None if the value failed to be converted.
pub fn new_with_value<T: ToFoldedHeader>(name: String, value: T) -> Header {
    let header_len = name.len() + 2;

    Header::new(
        name.clone(),
        ToFoldedHeader::to_folded_header(header_len, value),
    )
}
impl<T: ToHeader> ToFoldedHeader for T {
    fn to_folded_header(_: usize, value: T) -> String {
        // We ignore the start_position because the thing will fold anyway.
        ToHeader::to_header(value)
    }
}

impl ToHeader for String {
    fn to_header(value: String) -> String {
        value
    }
}

impl<'a> ToHeader for &'a str {
    fn to_header(value: &'a str) -> String {
        value.to_string()
    }
}

/// Represents an RFC 822 Header
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Header {
    /// The name of this header
    pub name: String,
    value: String,
}

impl<S: Into<String>, T: Into<String>> From<(S, T)> for Header {
    fn from(header: (S, T)) -> Self {
        let (name, value) = header;
        Header::new(name.into(), value.into())
    }
}

impl Header {
    /// Creates a new Header for the given `name` and `value`
    pub fn new(name: String, value: String) -> Header {
        Header { name, value }
    }

    /// Creates a new Header for the given `name` and `value`,
    /// as converted through the `ToHeader` or `ToFoldedHeader` trait.
    ///
    /// Returns None if the value failed to be converted.
    pub fn new_with_value<T: ToFoldedHeader>(name: String, value: T) -> Header {
        let header_len = name.len() + 2;

        Header::new(
            name.clone(),
            ToFoldedHeader::to_folded_header(header_len, value),
        )
    }

    /// Encode a UTF-8 string according to RFC 2047
    ///
    /// Currently, this only uses "B" encoding.
    ///
    /// Can be used on header content.
    pub fn encode_rfc2047(text: &str) -> String {
        let mut first = true;
        let mut res = String::new();
        let mut tmp_res = String::new();
        for source_char in text.chars() {
            let mut b = [0; 4];
            let enc_char = source_char.encode_utf8(&mut b);
            dbg!(&enc_char);
            let dest_char = base64::encode_config(enc_char.as_bytes(), base64::STANDARD);
            dbg!(&dest_char);
            if tmp_res.len() + dest_char.len() < MAX_ENCODED_WORD_LEN - 12 {
                tmp_res.push_str(&dest_char)
            } else {
                if !first {
                    res.push_str("\r\n ");
                }
                res.push_str(&format!("=?utf-8?B?{}?=", tmp_res));
                tmp_res.clear();
                first = false;
            }
        }

        if tmp_res.len() > 0 {
            if !first {
                res.push_str("\r\n ");
            }
            res.push_str(&format!("=?utf-8?B?{}?=", tmp_res));
        }

        res
    }
}

impl fmt::Display for Header {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}: {}", self.name, self.value)
    }
}

pub struct HeaderIter<'s> {
    iter: SliceIter<'s, Arc<Header>>,
}

impl<'s> HeaderIter<'s> {
    fn new(iter: SliceIter<'s, Arc<Header>>) -> HeaderIter<'s> {
        HeaderIter { iter: iter }
    }
}

impl<'s> Iterator for HeaderIter<'s> {
    type Item = &'s Header;

    fn next(&mut self) -> Option<&'s Header> {
        match self.iter.next() {
            Some(s) => Some(s.deref()),
            None => None,
        }
    }
}

/// A collection of Headers
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct HeaderMap {
    // An ordered list of headers,
    // which is used to iterate over.
    ordered_headers: Vec<Arc<Header>>,
}

impl HeaderMap {
    pub fn new() -> HeaderMap {
        HeaderMap {
            ordered_headers: Vec::new(),
        }
    }

    /// Adds a header to the collection
    pub fn insert(&mut self, header: Header) {
        self.ordered_headers.push(Arc::new(header));
    }

    /// Get an Iterator over the collection of headers.
    pub fn iter(&self) -> HeaderIter {
        HeaderIter::new(self.ordered_headers.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_encode_rfc2047() {
        assert_eq!(Header::encode_rfc2047("testà"), "=?utf-8?B?dGVzdMOg?=");
        assert_eq!(
            Header::encode_rfc2047(
                "testàtesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttest"
            ),
            "=?utf-8?B?dGVzdMOgdGVzdHRlc3R0ZXN0dGVzdHRlc3R0ZXN0dGVzdHRlc3R0ZXN0dGVzdHR?=\r\n=?utf-8?B?lc3R0ZXN0dGVzdHRlc3R0ZXN0dGVzdHRlc3R0ZXN0?="
        );
    }

    static SAMPLE_HEADERS: [(&'static str, &'static str); 4] = [
        ("Test", "Value"),
        ("Test", "Value 2"),
        ("Test-2", "Value 3"),
        ("Test-Multiline", "Foo\nBar"),
    ];

    fn make_sample_headers() -> Vec<Header> {
        SAMPLE_HEADERS
            .iter()
            .map(|&(name, value)| Header::new(name.to_string(), value.to_string()))
            .collect()
    }

    #[test]
    fn test_header_map_iter() {
        let mut headers = HeaderMap::new();
        let mut expected_headers = HashSet::new();
        for header in make_sample_headers().into_iter() {
            headers.insert(header.clone());
            expected_headers.insert(header);
        }

        let mut count = 0;
        // Ensure all the headers returned are expected
        for header in headers.iter() {
            assert!(expected_headers.contains(header));
            count += 1;
        }
        // And that there is the right number of them
        assert_eq!(count, expected_headers.len());
    }
}
