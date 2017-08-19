extern crate lettre;

use lettre::{ClientSecurity, EmailAddress, EmailTransport, SimpleSendableEmail, SmtpTransport};

#[test]
fn smtp_transport_simple() {
    let mut sender = SmtpTransport::builder("127.0.0.1:2525", ClientSecurity::None)
        .unwrap()
        .build();
    let email = SimpleSendableEmail::new(
        EmailAddress::new("user@localhost".to_string()),
        vec![EmailAddress::new("root@localhost".to_string())],
        "smtp_id".to_string(),
        "Hello smtp".to_string(),
    );

    let result = sender.send(email);
    assert!(result.is_ok());
}
