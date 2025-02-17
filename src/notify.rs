use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use reqwest::Client;

pub async fn send_email(body: String, client: Option<String>) {
	let email = (std::env::var("SMTP_USERNAME").unwrap(), std::env::var("SMTP_PASSWORD").unwrap());
	let creds = Credentials::new(email.0.to_owned(), email.1.to_owned());
	let builder = Message::builder()
		.from(email.0.parse().unwrap())
		.to(email.0.parse().unwrap())
		.subject(if client.is_some() { format!("{} IP has changed", client.unwrap()) } else { "Home IP has changed".to_string() })
		.header(ContentType::TEXT_PLAIN)
		.body(body).unwrap();

	// Open a remote connection to gmail
	match SmtpTransport::relay(&std::env::var("SMTP_HOST").unwrap()) {
		Ok(c) => {
			let cc = c.credentials(creds).build();
			match cc.send(&builder) {
				Ok(_) => tracing::info!("Email sent!"),
				Err(e) => tracing::error!("Error: could not send email: {e:?}"),
			}
		}
		Err(e) => tracing::error!("Error: could not connect to SMTP host: {e}"),
	}
}

pub async fn send_ntfy(reqwest_client: &Client, body: String, client: Option<String>) {
	let ntfy_host = std::env::var("NTFY_HOST").unwrap();
	let ntfy_tok = std::env::var("NTFY_TOKEN").unwrap();
	let ntfy_priority = std::env::var("NTFY_PRIORITY").unwrap();

	match reqwest_client.post(ntfy_host)
		.header("Authorization", ntfy_tok)
		.header("Priority", ntfy_priority)
		.header("Title", if client.is_some() { format!("{} IP has changed", client.unwrap()) } else { "Home IP has changed".to_string() })
		.body(body)
		.send().await {
		Ok(_) => tracing::info!("NTFY sent!"),
		Err(e) => tracing::error!("Failed to send notification via NTFY: {e:?}")
	}
}
