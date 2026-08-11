use dms_core::{
    DeliveryReceipt, DeliveryStatus, NotificationClient, NotificationMessage, NotificationSettings,
    NotificationTransport,
};
use keyring::Entry;
use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, Message, SmtpTransport,
    Transport,
};
use uuid::Uuid;

const KEYRING_SERVICE: &str = "dms-desktop";
const SMTP_PASSWORD_PURPOSE: &str = "smtp-password";

pub trait CredentialStore: Send + Sync {
    fn smtp_password(&self, workspace_id: Uuid) -> Result<String, String>;
}

#[derive(Default)]
pub struct OsCredentialStore;

impl CredentialStore for OsCredentialStore {
    fn smtp_password(&self, workspace_id: Uuid) -> Result<String, String> {
        let entry = Entry::new(
            KEYRING_SERVICE,
            &format!("{workspace_id}/{SMTP_PASSWORD_PURPOSE}"),
        )
        .map_err(|error| format!("cannot access the OS credential store: {error}"))?;
        entry.get_password().map_err(|error| {
            format!(
                "SMTP password is not available in the OS credential store for workspace {workspace_id}: {error}"
            )
        })
    }
}

pub struct DesktopNotifier<C> {
    credentials: C,
    workspace_id: Uuid,
    mailto_confirmed: bool,
    open_uri: fn(&str) -> Result<(), String>,
}

impl<C> DesktopNotifier<C> {
    pub fn new(
        credentials: C,
        workspace_id: Uuid,
        mailto_confirmed: bool,
        open_uri: fn(&str) -> Result<(), String>,
    ) -> Self {
        Self {
            credentials,
            workspace_id,
            mailto_confirmed,
            open_uri,
        }
    }
}

pub fn production_notifier(
    workspace_id: Uuid,
    mailto_confirmed: bool,
) -> DesktopNotifier<OsCredentialStore> {
    DesktopNotifier::new(
        OsCredentialStore,
        workspace_id,
        mailto_confirmed,
        open_host_mail_handler,
    )
}

fn open_host_mail_handler(uri: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let mut command = std::process::Command::new("explorer.exe");
    #[cfg(target_os = "macos")]
    let mut command = std::process::Command::new("open");
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = std::process::Command::new("xdg-open");

    command
        .arg(uri)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("cannot open the host mail handler: {error}"))
}

impl<C: CredentialStore> NotificationClient for DesktopNotifier<C> {
    fn send(
        &mut self,
        settings: &NotificationSettings,
        message: &NotificationMessage,
    ) -> Result<DeliveryReceipt, String> {
        match settings.transport {
            NotificationTransport::Mailto => {
                (self.open_uri)(&message.mailto_uri)?;
                Ok(DeliveryReceipt {
                    status: if self.mailto_confirmed {
                        DeliveryStatus::Confirmed
                    } else {
                        DeliveryStatus::Queued
                    },
                    response_code: None,
                    detail: if self.mailto_confirmed {
                        "operator confirmed the host mail message was sent".to_owned()
                    } else {
                        "opened the host mail handler; operator confirmation is required".to_owned()
                    },
                })
            }
            NotificationTransport::Smtp => {
                let smtp = settings
                    .smtp
                    .as_ref()
                    .ok_or_else(|| "SMTP transport requires relay settings".to_owned())?;
                let email =
                    Message::builder()
                        .from(smtp.sender.parse::<Mailbox>().map_err(|error| {
                            format!("invalid SMTP sender {}: {error}", smtp.sender)
                        })?)
                        .to(message.recipient.parse::<Mailbox>().map_err(|error| {
                            format!(
                                "invalid notification recipient {}: {error}",
                                message.recipient
                            )
                        })?)
                        .subject(&message.subject)
                        .body(message.body.clone())
                        .map_err(|error| format!("cannot build notification message: {error}"))?;
                let password = self.credentials.smtp_password(self.workspace_id)?;
                let mailer = SmtpTransport::starttls_relay(&smtp.relay_host)
                    .map_err(|error| format!("cannot configure SMTP relay: {error}"))?
                    .port(smtp.relay_port)
                    .credentials(Credentials::new(smtp.sender.clone(), password))
                    .build();
                let response = mailer
                    .send(&email)
                    .map_err(|error| format!("SMTP delivery failed: {error}"))?;
                let code = response.code().to_string().parse::<u16>().unwrap_or(250);
                Ok(DeliveryReceipt::accepted(
                    code,
                    &format!("SMTP relay accepted message: {response:?}"),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeCredentials;

    impl CredentialStore for FakeCredentials {
        fn smtp_password(&self, _workspace_id: Uuid) -> Result<String, String> {
            Ok("not-used-for-mailto".to_owned())
        }
    }

    fn host_mail_handler(uri: &str) -> Result<(), String> {
        if uri.starts_with("mailto:") {
            Ok(())
        } else {
            Err("not a mailto URI".to_owned())
        }
    }

    fn mailto_settings() -> NotificationSettings {
        NotificationSettings {
            transport: NotificationTransport::Mailto,
            smtp: None,
        }
    }

    fn message() -> NotificationMessage {
        NotificationMessage {
            kind: dms_core::NotificationKind::ReviewRequest,
            recipient: "approver@example.test".to_owned(),
            subject: "Review".to_owned(),
            body: "Review body".to_owned(),
            mailto_uri: "mailto:approver@example.test?subject=Review".to_owned(),
        }
    }

    #[test]
    fn mailto_requires_explicit_operator_confirmation() {
        let mut notifier =
            DesktopNotifier::new(FakeCredentials, Uuid::new_v4(), false, host_mail_handler);

        let receipt = notifier.send(&mailto_settings(), &message()).unwrap();

        assert_eq!(receipt.status, DeliveryStatus::Queued);
    }

    #[test]
    fn confirmed_mailto_records_confirmed_delivery() {
        let mut notifier =
            DesktopNotifier::new(FakeCredentials, Uuid::new_v4(), true, host_mail_handler);

        let receipt = notifier.send(&mailto_settings(), &message()).unwrap();

        assert_eq!(receipt.status, DeliveryStatus::Confirmed);
    }
}
