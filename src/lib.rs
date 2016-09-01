// Copyright © 2014 - 2015 by Optimal Computing Limited (of New Zealand)
// This code is licensed under the MIT license (see LICENSE-MIT for details)

extern crate time;
extern crate email;
extern crate lettre;
extern crate rustc_serialize;

use std::path::PathBuf;
use std::net::ToSocketAddrs;
use std::fs::File;
use std::io::Read;

use email::{MimeMessage, Header, MimeMultipartType};
use lettre::email::EmailBuilder;
use lettre::email::{ToMailbox, ToHeader};
use lettre::transport::smtp::{SecurityLevel, SmtpTransportBuilder};
pub use lettre::transport::smtp::authentication::Mechanism;
use lettre::transport::EmailTransport;
use time::Tm;

pub use error::Error;

pub mod error;

/// Email structure
pub struct Email {
    plain_body: Option<String>,
    html_body: Option<String>,
    attachments: Vec<PathBuf>,
    builder: EmailBuilder,
}

impl Email {
    pub fn new() -> Email
    {
        Email {
            plain_body: None,
            html_body: None,
            attachments: Vec::new(),
            builder: EmailBuilder::new()
                .header(Header::new("X-Mailer".to_owned(), "JoistMailer".to_owned()))
                .header(Header::new("MIME-Version".to_owned(), "1.0".to_owned())),
        }
    }

    /// Add a `To` address
    pub fn to<A: ToMailbox>(mut self, address: A) -> Email {
        self.builder.add_to(address);
        self
    }
    /// Add a `To` address
    pub fn add_to<A: ToMailbox>(&mut self, address: A) {
        self.builder.add_to(address);
    }

    /// Add a `From` address
    pub fn from<A: ToMailbox>(mut self, address: A) -> Email {
        self.builder.add_from(address);
        self
    }
    /// Add a `From` address
    pub fn add_from<A: ToMailbox>(&mut self, address: A) {
        self.builder.add_from(address);
    }

    /// Add a `Cc` address
    pub fn cc<A: ToMailbox>(mut self, address: A) -> Email {
        self.builder.add_cc(address);
        self
    }
    /// Add a `Cc` address
    pub fn add_cc<A: ToMailbox>(&mut self, address: A) {
        self.builder.add_cc(address);
    }

    /// Add a `Sender` address
    pub fn sender<A: ToMailbox>(mut self, address: A) -> Email {
        self.builder.set_sender(address);
        self
    }
    /// Add a `Sender` address
    pub fn set_sender<A: ToMailbox>(&mut self, address: A) {
        self.builder.set_sender(address);
    }

    /// Add a `Reply-to` address
    pub fn reply_to<A: ToMailbox>(mut self, address: A) -> Email {
        self.builder.add_reply_to(address);
        self
    }
    /// Add a `Reply-to` address
    pub fn add_reply_to<A: ToMailbox>(&mut self, address: A) {
        self.builder.add_reply_to(address);
    }

    /// Add a header.  Use this for headers not handled by `Email` directly.
    pub fn header<H: ToHeader>(mut self, header: H) -> Email {
        self.builder.add_header(header);
        self
    }
    /// Add a header.  Use this for headers not handled by `Email` directly.
    pub fn add_header<H: ToHeader>(&mut self, header: H) {
        self.builder.add_header(header);
    }

    /// Add a date header
    pub fn date(mut self, date: &Tm) -> Email {
        self.builder.set_date(date);
        self
    }
    /// Add a date header
    pub fn set_date(&mut self, date: &Tm) {
        self.builder.set_date(date);
    }

    /// Add a subject header
    pub fn subject(mut self, subject: &str) -> Email {
        self.builder.set_subject(subject);
        self
    }
    /// Add a subject header
    pub fn set_subject(&mut self, subject: &str) {
        self.builder.set_subject(subject);
    }

    /// Set a plain-text body
    pub fn plain_body(mut self, body: &str) -> Email {
        self.plain_body = Some(body.to_owned());
        self
    }
    /// Set a plain-text body
    pub fn set_plain_body(&mut self, body: &str) {
        self.plain_body = Some(body.to_owned());
    }

    /// Set an HTML body.
    pub fn html_body(mut self, body: &str) -> Email {
        self.html_body = Some(body.to_owned());
        self
    }
    /// Set an HTML body.
    pub fn set_html_body(&mut self, body: &str) {
        self.html_body = Some(body.to_owned());
    }

    /// Add an attachment
    pub fn attach(mut self, attachment: PathBuf) -> Email {
        self.attachments.push(attachment);
        self
    }
    /// Add an attachment
    pub fn add_attachment(&mut self, attachment: PathBuf) {
        self.attachments.push(attachment);
    }

    fn build_mime_message(&self) -> Result<MimeMessage, Error>
    {
        let html = if self.html_body.is_some() {
            // FIXME - transfer encode!
            let mut html = MimeMessage::new(
                self.html_body.as_ref().unwrap().clone());
            html.message_type = None;
            html.headers.insert(Header::new("Content-Type".to_owned(),
                                            "text/html; charset=\"ascii\"".to_owned()));
            html.headers.insert(Header::new("Content-Transfer-Encoding".to_owned(),
                                            "7bit".to_owned()));
            Some(html)
        } else {
            None
        };

        let plain = if self.plain_body.is_some() {
            // FIXME - transfer encode!
            let mut plain = MimeMessage::new(
                self.plain_body.as_ref().unwrap().clone());
            plain.message_type = None;
            plain.headers.insert(Header::new("Content-Type".to_owned(),
                                             "text/plain; charset=\"ascii\"".to_owned()));
            plain.headers.insert(Header::new("Content-Transfer-Encoding".to_owned(),
                                             "7bit".to_owned()));
            Some(plain)
        } else {
            None
        };

        let main_body = if html.is_some() && plain.is_some() {
            let mut body = MimeMessage::new(
                "This is a multipart message in MIME format.".to_owned());
            body.message_type = Some(MimeMultipartType::Alternative);
            body.children = vec![ html.unwrap(), plain.unwrap() ];
            body.update_headers();
            body
        } else if html.is_some() {
            html.unwrap()
        } else if plain.is_some() {
            plain.unwrap()
        } else {
            return Err( Error::BodyRequired )
        };

        let message = if self.attachments.len() > 0 {
            let mut message = MimeMessage::new(
                "This is a multipart message in MIME format.".to_owned());
            message.message_type = Some(MimeMultipartType::Mixed);
            let mut children: Vec<MimeMessage> = Vec::new();
            children.push(main_body);
            for attachment in self.attachments.iter() {
                let mut file = try!(File::open(attachment));
                let mut s = String::new();
                try!(file.read_to_string(&mut s));
                let base64 = to_base64(s.as_bytes());

                let mut mime_attachment = MimeMessage::new(base64);
                mime_attachment.message_type = None;
                mime_attachment.headers.insert(
                    Header::new("Content-Disposition".to_owned(),
                                format!("attachment; filename=\"{}\"",
                                        attachment.file_name().unwrap().to_string_lossy())));
                mime_attachment.headers.insert(
                    Header::new("Content-Type".to_owned(),
                                "application/octet-stream".to_owned()));
                mime_attachment.headers.insert(
                    Header::new("Content-Transfer-Encoding".to_owned(),
                                "base64".to_owned()));
                children.push(mime_attachment)
            }
            message.children = children;
            message.update_headers();
            message
        } else {
            main_body
        };

        Ok(message)
    }

    // [TEMPORARY] output as string
    pub fn debug_display(&self)
    {
        let mime_message = match self.build_mime_message() {
            Ok(m) => m,
            Err(e) => {
                println!("ERROR: {:?}", e);
                return;
            }
        };

        // For testing:
        println!("YOUR EMAIL WILL LOOK LIKE THIS:\n\n");

        for header in mime_message.headers.iter() {
            println!("EMAIL HEADER = {}", header);
        }
        println!("\n{}", mime_message.as_string_without_headers());
    }

    pub fn send<A: ToSocketAddrs>(&self, smtp_address: A, hello_name: &str,
                                  username: &str, password: &str,
                                  auth_mechanism: Mechanism)
                                  -> Result<(), Error>
    {
        let mime_message = try!(self.build_mime_message());

        let builder = self.builder.clone(); // FIXME, annoying upstream chain-only API

        // Set the body from the mime_message
        let body = mime_message.as_string_without_headers();
        let mut builder = builder.body(&body[..]);

        // Add headers from the mime_message
        for header in mime_message.headers.iter() {
            builder.add_header(header.clone());
        }

        let email = try!(builder.build());

        let mailer = try!(SmtpTransportBuilder::new(smtp_address));

        let mut mailer = mailer
            .hello_name(hello_name)
            .credentials(username, password)
            .authentication_mechanism(auth_mechanism)
            .security_level(SecurityLevel::Opportunistic)
            .smtp_utf8(true)
            .build();

        let response = try!(mailer.send(email));
        if response.is_positive() {
            return Ok(())
        }

        Err( Error::SendFailed( format!("{}/{}/{} {}",
                                        response.severity(),
                                        response.category(),
                                        response.detail(),
                                        response.message().join("\r\n")) ) )
    }
}

// This uses the standard original Base64 definition in RFC 2045 section 6.8
// which is designed for email
fn to_base64(bytes: &[u8]) -> String {
    use rustc_serialize::base64::{ToBase64,Config,CharacterSet,Newline};
    bytes.to_base64(Config {
        char_set: CharacterSet::Standard,
        newline: Newline::CRLF,
        pad: true,
        line_length: Some(76),
    })
}


#[cfg(test)]
mod tests {
    use Email;
    use std::path::PathBuf;
    use time::{self,Tm};

    #[test]
    fn test_display() {
        let mut email = Email::new();
        email.add_to( "mike@efx.co.nz" );
        email.add_to( "callum@efx.co.nz" );
        email.add_from( "mailer@onestart.nz" );
        //email.add_cc( "copy@onestart.nz" );
        email.add_sender( "mailer@onestart.nz" );
        let now: Tm = time::now();
        email.add_date( &now );
        email.add_subject( "This is a test from OneStart" );
        email.set_plain_body( "This is a test email
You should ignore it.");
        email.set_html_body( "<hr><p>This is a <b>test</b> email<br>
You should ignore it.</p><hr>
");
        email.add_attachment( PathBuf::from("/tmp/main.rs") );

        email.debug_display();
    }
}
