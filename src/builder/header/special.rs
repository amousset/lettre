use hyperx::{
    header::{Formatter as HeaderFormatter, Header, Raw},
    Error as HyperError, Result as HyperResult,
};
use std::fmt::Result as FmtResult;
use std::str::from_utf8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MimeVersion {
    pub major: u8,
    pub minor: u8,
}

pub const MIME_VERSION_1_0: MimeVersion = MimeVersion { major: 1, minor: 0 };

impl MimeVersion {
    pub fn new(major: u8, minor: u8) -> Self {
        MimeVersion { major, minor }
    }
}

impl Default for MimeVersion {
    fn default() -> Self {
        MIME_VERSION_1_0
    }
}

impl Header for MimeVersion {
    fn header_name() -> &'static str {
        "MIME-Version"
    }

    fn parse_header(raw: &Raw) -> HyperResult<Self> {
        raw.one().ok_or(HyperError::Header).and_then(|r| {
            let s: Vec<&str> = from_utf8(r)
                .map_err(|_| HyperError::Header)?
                .split('.')
                .collect();
            if s.len() != 2 {
                return Err(HyperError::Header);
            }
            let major = s[0].parse().map_err(|_| HyperError::Header)?;
            let minor = s[1].parse().map_err(|_| HyperError::Header)?;
            Ok(MimeVersion::new(major, minor))
        })
    }

    fn fmt_header(&self, f: &mut HeaderFormatter) -> FmtResult {
        f.fmt_line(&format!("{}.{}", self.major, self.minor))
    }
}

#[cfg(test)]
mod test {
    use super::{MimeVersion, MIME_VERSION_1_0};
    use hyperx::Headers;

    #[test]
    fn format_mime_version() {
        let mut headers = Headers::new();

        headers.set(MIME_VERSION_1_0);

        assert_eq!(format!("{}", headers), "MIME-Version: 1.0\r\n");

        headers.set(MimeVersion::new(0, 1));

        assert_eq!(format!("{}", headers), "MIME-Version: 0.1\r\n");
    }

    #[test]
    fn parse_mime_version() {
        let mut headers = Headers::new();

        headers.set_raw("MIME-Version", "1.0");

        assert_eq!(headers.get::<MimeVersion>(), Some(&MIME_VERSION_1_0));

        headers.set_raw("MIME-Version", "0.1");

        assert_eq!(headers.get::<MimeVersion>(), Some(&MimeVersion::new(0, 1)));
    }
}
