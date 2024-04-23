use super::Action;
use crate::message::MessageConfig;
use crate::message::MessageFormat;
use crate::message::MessageParams;
use anyhow::{Context, Result};
use async_trait::async_trait;
use lettre::message::header::ContentType;
use lettre::message::MessageBuilder;
use lettre::message::MultiPart;
use lettre::message::SinglePart;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::transport::smtp::client::TlsParametersBuilder;
use lettre::AsyncSmtpTransport;
use lettre::AsyncTransport;
use lettre::Message;
use lettre::Tokio1Executor;
use serde::Deserialize;

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum EmailConnectionType {
    Plain,
    StartTls,
    Tls,
}

#[derive(Deserialize, Debug)]
pub struct EmailConfig {
    server: String,
    port: Option<u16>,
    credentials: Option<Credentials>,
    connection: Option<EmailConnectionType>,
    self_signed_cert: Option<bool>,
    from: String,
    to: Option<String>,
}

impl EmailConfig {
    pub fn server(&self) -> &str {
        &self.server
    }

    pub fn connection(&self) -> EmailConnectionType {
        self.connection.unwrap_or(EmailConnectionType::Tls)
    }

    pub fn self_signed_cert(&self) -> bool {
        self.self_signed_cert.unwrap_or(false)
    }

    pub fn port(&self) -> u16 {
        self.port.unwrap_or(match self.connection() {
            EmailConnectionType::Tls => 587,
            EmailConnectionType::StartTls => 465,
            EmailConnectionType::Plain => 25,
        })
    }

    pub fn to(&self) -> &str {
        self.to.as_deref().unwrap_or(self.from.as_ref())
    }
}

pub struct EmailAction<'a> {
    message_config: &'a MessageConfig,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    message_builder: MessageBuilder,
}
impl<'a> EmailAction<'a> {
    pub fn new(message_config: &'a MessageConfig, email_config: &'a EmailConfig) -> Result<Self> {
        let tls_builder = TlsParametersBuilder::new(email_config.server().into())
            .dangerous_accept_invalid_certs(email_config.self_signed_cert());
        let tls_parameters = tls_builder.build()?;

        let mut smtp_builder =
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(email_config.server())
                .port(email_config.port())
                .tls(match email_config.connection() {
                    EmailConnectionType::Tls => Tls::Wrapper(tls_parameters),
                    EmailConnectionType::StartTls => Tls::Required(tls_parameters),
                    EmailConnectionType::Plain => Tls::None,
                });
        if let Some(cred) = &email_config.credentials {
            smtp_builder = smtp_builder.credentials(cred.clone())
        }
        Ok(Self {
            message_config,
            mailer: smtp_builder.build(),
            message_builder: Message::builder()
                .from(
                    email_config
                        .from
                        .parse()
                        .with_context(|| format!("invalid from address '{}'", email_config.from))?,
                )
                .to(email_config
                    .to()
                    .parse()
                    .with_context(|| format!("invalid to address '{}'", email_config.to()))?),
        })
    }
}

#[async_trait]
impl Action<'_> for EmailAction<'_> {
    fn name(&self) -> &'static str {
        "email"
    }

    async fn run(&self, params: Option<&MessageParams<'_, '_>>) -> Result<()> {
        let body = self.message_config.body(params)?;
        let html_body = match self.message_config.format() {
            MessageFormat::Markdown => format!(
                "<!DOCTYPE html><html><body>{}</body></html>",
                markdown::to_html(&body)
            ),
            MessageFormat::Html => body.clone(),
            MessageFormat::Plain => Default::default(),
        };
        let email_builder = self
            .message_builder
            .clone()
            .subject(self.message_config.subject(params)?);
        let email = match self.message_config.format() {
            MessageFormat::Plain => email_builder
                .header(ContentType::TEXT_PLAIN)
                .body(body.clone())?,
            MessageFormat::Markdown | MessageFormat::Html => email_builder.multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(body.clone()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_body.clone()),
                    ),
            )?,
        };
        self.mailer.send(email).await?;
        Ok(())
    }
}
