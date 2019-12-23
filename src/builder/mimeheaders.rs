use super::header::ToHeader;
use super::results::ParsingResult;
use std::collections::HashMap;

/// Content-Type string, major/minor as the first and second elements
/// respectively.
pub type MimeContentType = (String, String);

/// Special header type for the Content-Type header.
pub struct MimeContentTypeHeader {
    /// The content type presented by this header
    pub content_type: MimeContentType,
    /// Parameters of this header
    pub params: HashMap<String, String>,
}

impl ToHeader for MimeContentTypeHeader {
    fn to_header(value: MimeContentTypeHeader) -> ParsingResult<String> {
        let (mime_major, mime_minor) = value.content_type;
        let mut result = format!("{}/{}", mime_major, mime_minor);
        for (key, val) in value.params.iter() {
            result = format!("{}; {}={}", result, key, val);
        }
        Ok(result)
    }
}
