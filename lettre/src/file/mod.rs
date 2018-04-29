//! The file transport writes the emails to the given directory. The name of the file will be
//! `message_id.txt`.
//! It can be useful for testing purposes, or if you want to keep track of sent messages.
//!

use Transport;
use Envelope;
use SendableEmail;
use file::error::FileResult;
use serde_json;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

pub mod error;

/// Writes the content and the envelope information to a file
#[derive(Debug)]
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
pub struct FileTransport {
    path: PathBuf,
}

impl FileTransport {
    /// Creates a new transport to the given directory
    pub fn new<P: AsRef<Path>>(path: P) -> FileTransport {
        let mut path_buf = PathBuf::new();
        path_buf.push(path);
        FileTransport { path: path_buf }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
struct SerializableEmail {
    envelope: Envelope,
    message_id: String,
    message: Vec<u8>,
}

impl<'a> Transport<'a> for FileTransport {
    type Result = FileResult;

    fn send(&mut self, email: SendableEmail) -> FileResult {
        let message_id = email.message_id().to_string();
        let envelope = email.envelope().clone();

        let mut file = self.path.clone();
        file.push(format!("{}.json", message_id));

        let mut f = File::create(file.as_path())?;

        let serialized = serde_json::to_string(&SerializableEmail {
            envelope,
            message_id,
            message: email.message_to_string()?.as_bytes().to_vec(),
        })?;

        f.write_all(serialized.as_bytes())?;

        Ok(())
    }
}
