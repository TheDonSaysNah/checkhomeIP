use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub async fn send_email(current_ip:&str, new_ip: &str) {
    let email = ("gmail account here", "App password goes here. This is NOT your gmail password"); // https://myaccount.google.com/apppasswords
    let creds = Credentials::new(email.0.to_owned(), email.1.to_owned());
    let builder = Message::builder()
    .from(email.0.parse().unwrap())
    .to(email.0.parse().unwrap())
    .subject("Home IP has changed")
    .header(ContentType::TEXT_PLAIN)
    .body(format!("Your home IP has changed from {current_ip} to {new_ip}")).unwrap();


    // Open a remote connection to gmail
    match SmtpTransport::relay("smtp.gmail.com") {
        Ok(c) => {
            let cc = c.credentials(creds).build();
            match cc.send(&builder) {
                Ok(_) => tracing::info!("Email sent!"),
                Err(e) => tracing::error!("Error: could not send email: {e:?}"),
            }
        }
        Err(e) => tracing::error!("Error: could not connect to Gmail: {e}"),
    }
}
