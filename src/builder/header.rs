use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::slice::Iter as SliceIter;
use std::sync::Arc;

use super::results::ParsingResult;

/// Encode a UTF-8 string according to RFC 2047, if need be.
///
/// Currently, this only uses "B" encoding, when pure ASCII cannot represent the
/// string accurately.
///
/// Can be used on header content.
pub fn encode_rfc2047(text: &str) -> Cow<str> {
    if text.is_ascii() {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(
            base64::encode_config(text.as_bytes(), base64::STANDARD)
                // base64 so ascii
                .as_bytes()
                // Max length - wrapping chars
                .chunks(75 - 12)
                .map(|d| format!("=?utf-8?B?{}?=", std::str::from_utf8(d).unwrap()))
                .collect::<Vec<String>>()
                .join("\r\n"),
        )
    }
}

/// Trait for converting from a Rust type into a Header value.
pub trait ToHeader {
    /// Turn the `value` into a String suitable for being used in
    /// a message header.
    ///
    /// Returns None if the value cannot be stringified.
    fn to_header(value: Self) -> ParsingResult<String>;
}

/// Trait for converting from a Rust time into a Header value
/// that handles its own folding.
///
/// Be mindful that this trait does not mean that the value will
/// not be folded later, rather that the type returns a value that
/// should not be folded, given that the header value starts so far
/// in to a line.
/// [unstable]
pub trait ToFoldedHeader {
    fn to_folded_header(start_pos: usize, value: Self) -> ParsingResult<String>;
}

impl<T: ToHeader> ToFoldedHeader for T {
    fn to_folded_header(_: usize, value: T) -> ParsingResult<String> {
        // We ignore the start_position because the thing will fold anyway.
        ToHeader::to_header(value)
    }
}

impl ToHeader for String {
    fn to_header(value: String) -> ParsingResult<String> {
        Ok(value)
    }
}

impl<'a> ToHeader for &'a str {
    fn to_header(value: &'a str) -> ParsingResult<String> {
        Ok(value.to_string())
    }
}

/// Represents an RFC 822 Header
/// [unstable]
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
    /// [unstable]
    pub fn new(name: String, value: String) -> Header {
        Header {
            name: name,
            value: value,
        }
    }

    /// Creates a new Header for the given `name` and `value`,
    /// as converted through the `ToHeader` or `ToFoldedHeader` trait.
    ///
    /// Returns None if the value failed to be converted.
    /// [unstable]
    pub fn new_with_value<T: ToFoldedHeader>(name: String, value: T) -> ParsingResult<Header> {
        let header_len = name.len() + 2;
        ToFoldedHeader::to_folded_header(header_len, value)
            .map(|val| Header::new(name.clone(), val))
    }
}

impl fmt::Display for Header {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}: {}", self.name, self.value)
    }
}

/// [unstable]
pub struct HeaderIter<'s> {
    iter: SliceIter<'s, Arc<Header>>,
}

impl<'s> HeaderIter<'s> {
    /// [unstable]
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
/// [unstable]
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct HeaderMap {
    // We store headers "twice" inside the HeaderMap.
    //
    // The first is as an ordered list of headers,
    // which is used to iterate over.
    ordered_headers: Vec<Arc<Header>>,
    // The second is as a mapping between header names
    // and all of the headers with that name.
    //
    // This allows quick retrieval of a header by name.
    headers: HashMap<String, Vec<Arc<Header>>>,
}

impl HeaderMap {
    /// [unstable]
    pub fn new() -> HeaderMap {
        HeaderMap {
            ordered_headers: Vec::new(),
            headers: HashMap::new(),
        }
    }

    /// Adds a header to the collection
    /// [unstable]
    pub fn insert(&mut self, header: Header) {
        let header_name = header.name.clone();
        let rc = Arc::new(header);
        // Add to the ordered list of headers
        self.ordered_headers.push(rc.clone());

        // and to the mapping between header names and values.
        match self.headers.entry(header_name) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(rc.clone());
            }
            Entry::Vacant(entry) => {
                // There haven't been any headers with this name
                // as of yet, so make a new list and push it in.
                let mut header_list = Vec::new();
                header_list.push(rc.clone());
                entry.insert(header_list);
            }
        };
    }

    /// Get an Iterator over the collection of headers.
    /// [unstable]
    pub fn iter(&self) -> HeaderIter {
        HeaderIter::new(self.ordered_headers.iter())
    }

    /// Get the last value of the header with `name`
    /// [unstable]
    pub fn get(&self, name: String) -> Option<&Header> {
        self.headers
            .get(&name)
            .map(|headers| headers.last().unwrap())
            .map(|rc| rc.deref())
    }

    /// [unstable]
    /// Get the number of headers within this map.
    pub fn len(&self) -> usize {
        self.ordered_headers.len()
    }

    /// [unstable]
    /// Find a list of headers of `name`, `None` if there
    /// are no headers with that name.
    pub fn find(&self, name: &String) -> Option<Vec<&Header>> {
        self.headers
            .get(name)
            .map(|rcs| rcs.iter().map(|rc| rc.deref()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_encode_rfc2047() {
        assert_eq!(encode_rfc2047("test"), "test");
        assert_eq!(encode_rfc2047("testà"), "=?utf-8?B?dGVzdMOg?=");
        assert_eq!(
            encode_rfc2047(
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
    fn test_header_map_len() {
        let mut headers = HeaderMap::new();
        for (i, header) in make_sample_headers().into_iter().enumerate() {
            headers.insert(header);
            assert_eq!(headers.len(), i + 1);
        }
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
