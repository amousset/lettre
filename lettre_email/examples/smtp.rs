extern crate lettre;
extern crate lettre_email;
extern crate mime;

use lettre::{SmtpTransport, Transport};
use lettre_email::Email;
use std::path::Path;

fn main() {
    let email = Email::builder()
        // Addresses can be specified by the tuple (email, alias)
        .to(("user@example.org", "Firstname Lastname"))
        // ... or by an address only
        .from("user@example.com")
        .subject("Hi, Hello world")
        .text("Hello world.")
        .attachment(Path::new("Cargo.toml"), None, &mime::TEXT_PLAIN).unwrap()
        .build()
        .unwrap();

    // Open a local connection on port 25
    let mut mailer = SmtpTransport::builder_unencrypted_localhost()
        .unwrap()
        .build();
    // Send the email
    let result = mailer.send(email.into());

    if result.is_ok() {
        println!("Email sent");
    } else {
        println!("Could not send email: {:?}", result);
    }

    assert!(result.is_ok());
}
