extern crate lettre;
extern crate env_logger;

use lettre::{EmailAddress, EmailTransport, SimpleSendableEmail, SmtpTransport};

fn main() {
    env_logger::init().unwrap();

    let email = SimpleSendableEmail::new(
        EmailAddress::new("user@localhost".to_string()),
        vec![EmailAddress::new("root@localhost".to_string())],
        "file_id".to_string(),
        "Hello ß☺ example".to_string(),
    );

    // Open a local connection on port 25
    let mut mailer = SmtpTransport::builder_unencrypted_localhost()
        .unwrap()
        .build();
    // Send the email
    let result = mailer.send(&email);

    if result.is_ok() {
        println!("Email sent");
    } else {
        println!("Could not send email: {:?}", result);
    }

    assert!(result.is_ok());
}
