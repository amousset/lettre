//! This transport is a stub that only logs the message, and always returns
//! success

use EmailTransport;
use SendableEmail;
use std::io::Read;

pub mod error;

/// This transport does nothing except logging the message envelope
#[derive(Debug)]
pub struct StubEmailTransport;

/// SMTP result type
pub type StubResult = Result<(), error::Error>;

impl<U: Read> EmailTransport<StubResult, U> for StubEmailTransport {
    fn send<T: SendableEmail<U>>(&mut self, email: T) -> StubResult {

        info!("{}: from=<{}> to=<{:?}>",
              email.message_id(),
              email.from(),
              email.to());
        Ok(())
    }

    fn close(&mut self) {
        ()
    }
}
