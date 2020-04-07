//! Provides a strongly typed way to build emails

/*!

## Usage

### Format email messages

#### With string body

The easiest way how we can create email message with simple string.

```rust
use lettre::message::Message;

fn main() {
    let m: Message<&str> = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .body("Be happy!")
        .unwrap();

    println!("{}", m);
}
```

Run this example:

```sh
$ cargo run --example format_string

From: NoBody <nobody@domain.tld>
Reply-To: Yuin <yuin@domain.tld>
To: Hei <hei@domain.tld>
Subject: Happy new year

Be happy!
```

The unicode header data will be encoded using _UTF8-Base64_ encoding.

#### With mime body

##### Single part

The more complex way is using MIME contents
(see [format\_mime.rs](examples/format_mime.rs)).

```rust
use lettre::message::{header, Message, SinglePart};

fn main() {
    let m: Message<SinglePart<&str>> = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .mime_body(
            SinglePart::builder()
                .header(header::ContentType(
                    "text/plain; charset=utf8".parse().unwrap(),
                )).header(header::ContentTransferEncoding::QuotedPrintable)
                .body("Привет, мир!"),
        )
        .unwrap();

    println!("{}", m);
}
```

The body will be encoded using selected `Content-Transfer-Encoding`.

```sh
$ cargo run --example format_mime

From: NoBody <nobody@domain.tld>
Reply-To: Yuin <yuin@domain.tld>
To: Hei <hei@domain.tld>
Subject: Happy new year
MIME-Version: 1.0
Content-Type: text/plain; charset=utf8
Content-Transfer-Encoding: quoted-printable

=D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!

```

##### Multiple parts

And more advanced way of building message by using multipart MIME contents
(see [format\_multipart.rs](examples/format_multipart.rs)).

```rust
use lettre::message::{header, Message, MultiPart, SinglePart};

fn main() {
    let m: Message<MultiPart<&str>> = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .mime_body(
            MultiPart::mixed()
            .multipart(
                MultiPart::alternative()
                .singlepart(
                    SinglePart::quoted_printable()
                    .header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                    .body("Привет, мир!")
                )
                .multipart(
                    MultiPart::related()
                    .singlepart(
                        SinglePart::eight_bit()
                        .header(header::ContentType("text/html; charset=utf8".parse().unwrap()))
                        .body("<p><b>Hello</b>, <i>world</i>! <img src=smile.png></p>")
                    )
                    .singlepart(
                        SinglePart::base64()
                        .header(header::ContentType("image/png".parse().unwrap()))
                        .header(header::ContentDisposition {
                            disposition: header::DispositionType::Inline,
                            parameters: vec![],
                        })
                        .body("<smile-raw-image-data>")
                    )
                )
            )
            .singlepart(
                SinglePart::seven_bit()
                .header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                .header(header::ContentDisposition {
                                 disposition: header::DispositionType::Attachment,
                                 parameters: vec![
                                     header::DispositionParam::Filename(
                                         header::Charset::Ext("utf-8".into()),
                                         None, "example.c".as_bytes().into()
                                     )
                                 ]
                             })
                .body("int main() { return 0; }")
            )
        ).unwrap();

    println!("{}", m);
}
```

```sh
$ cargo run --example format_multipart

From: NoBody <nobody@domain.tld>
Reply-To: Yuin <yuin@domain.tld>
To: Hei <hei@domain.tld>
Subject: Happy new year
MIME-Version: 1.0
Content-Type: multipart/mixed; boundary="RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m"

--RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m
Content-Type: multipart/alternative; boundary="qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy"

--qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy
Content-Transfer-Encoding: quoted-printable
Content-Type: text/plain; charset=utf8

=D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!
--qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy
Content-Type: multipart/related; boundary="BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8"

--BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8
Content-Transfer-Encoding: 8bit
Content-Type: text/html; charset=utf8

<p><b>Hello</b>, <i>world</i>! <img src=smile.png></p>
--BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8
Content-Transfer-Encoding: base64
Content-Type: image/png
Content-Disposition: inline

PHNtaWxlLXJhdy1pbWFnZS1kYXRhPg==
--BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8--
--qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy--
--RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m
Content-Transfer-Encoding: 7bit
Content-Type: text/plain; charset=utf8
Content-Disposition: attachment; filename="example.c"

int main() { return 0; }
--RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m--

```

 */

pub use encoder::*;
pub use mailbox::*;
pub use message::*;
pub use mimebody::*;

pub use mime;

mod encoder;
pub mod header;
mod mailbox;
mod message;
mod mimebody;
mod utf8_b;
