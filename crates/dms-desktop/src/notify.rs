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
    fn set_smtp_password(&self, workspace_id: Uuid, password: &str) -> Result<(), String>;
    fn delete_smtp_password(&self, workspace_id: Uuid) -> Result<(), String>;
    fn smtp_password_exists(&self, workspace_id: Uuid) -> Result<bool, String>;
}

#[derive(Default)]
pub struct OsCredentialStore;

impl CredentialStore for OsCredentialStore {
    fn smtp_password(&self, workspace_id: Uuid) -> Result<String, String> {
        let entry = smtp_password_entry(workspace_id)?;
        entry.get_password().map_err(|error| {
            format!(
                "SMTP password is not available in the OS credential store for workspace {workspace_id}: {error}"
            )
        })
    }

    fn set_smtp_password(&self, workspace_id: Uuid, password: &str) -> Result<(), String> {
        if password.trim().is_empty() {
            return Err("SMTP app password cannot be empty".to_owned());
        }
        smtp_password_entry(workspace_id)?
            .set_password(password)
            .map_err(|error| {
                format!("cannot save SMTP app password in the OS credential store: {error}")
            })
    }

    fn delete_smtp_password(&self, workspace_id: Uuid) -> Result<(), String> {
        match smtp_password_entry(workspace_id)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(format!(
                "cannot delete SMTP app password from the OS credential store: {error}"
            )),
        }
    }

    fn smtp_password_exists(&self, workspace_id: Uuid) -> Result<bool, String> {
        match smtp_password_entry(workspace_id)?.get_password() {
            Ok(_) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(error) => Err(format!(
                "cannot access the OS credential store for the SMTP app password: {error}"
            )),
        }
    }
}

impl OsCredentialStore {
    pub fn smtp_password_exists(workspace_id: Uuid) -> Result<bool, String> {
        CredentialStore::smtp_password_exists(&Self, workspace_id)
    }
}

fn smtp_password_entry(workspace_id: Uuid) -> Result<Entry, String> {
    Entry::new(
        KEYRING_SERVICE,
        &format!("{workspace_id}/{SMTP_PASSWORD_PURPOSE}"),
    )
    .map_err(|error| format!("cannot access the OS credential store: {error}"))
}

pub(crate) trait SmtpSender {
    fn send(
        &self,
        settings: &dms_core::SmtpSettings,
        password: String,
        message: &Message,
    ) -> Result<DeliveryReceipt, String>;
}

#[derive(Default)]
pub(crate) struct ProductionSmtpSender;

impl SmtpSender for ProductionSmtpSender {
    fn send(
        &self,
        settings: &dms_core::SmtpSettings,
        password: String,
        message: &Message,
    ) -> Result<DeliveryReceipt, String> {
        let mailer = SmtpTransport::starttls_relay(&settings.relay_host)
            .map_err(|error| format!("cannot configure SMTP relay: {error}"))?
            .port(settings.relay_port)
            .credentials(Credentials::new(settings.login_user.clone(), password))
            .build();
        let response = mailer
            .send(message)
            .map_err(|error| format!("SMTP delivery failed: {error}"))?;
        let code = response.code().to_string().parse::<u16>().unwrap_or(250);
        Ok(DeliveryReceipt::accepted(
            code,
            &format!("SMTP relay accepted message: {response:?}"),
        ))
    }
}

pub(crate) struct DesktopNotifier<C, S = ProductionSmtpSender> {
    credentials: C,
    smtp_sender: S,
    workspace_id: Uuid,
    mailto_confirmed: bool,
    open_uri: fn(&str) -> Result<(), String>,
}

impl<C> DesktopNotifier<C, ProductionSmtpSender> {
    pub fn new(
        credentials: C,
        workspace_id: Uuid,
        mailto_confirmed: bool,
        open_uri: fn(&str) -> Result<(), String>,
    ) -> Self {
        Self {
            credentials,
            smtp_sender: ProductionSmtpSender,
            workspace_id,
            mailto_confirmed,
            open_uri,
        }
    }
}

pub(crate) fn production_notifier(
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

impl<C: CredentialStore, S: SmtpSender> NotificationClient for DesktopNotifier<C, S> {
    fn send(
        &mut self,
        settings: &NotificationSettings,
        message: &NotificationMessage,
    ) -> Result<DeliveryReceipt, String> {
        match settings.transport {
            NotificationTransport::Mailto => {
                if !self.mailto_confirmed {
                    (self.open_uri)(&message.mailto_uri)?;
                }
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
                let email = Message::builder()
                    .from(smtp.from_mailbox.parse::<Mailbox>().map_err(|error| {
                        format!("invalid SMTP From address {}: {error}", smtp.from_mailbox)
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
                self.smtp_sender.send(smtp, password, &email)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct FakeCredentials;

    impl CredentialStore for FakeCredentials {
        fn smtp_password(&self, _workspace_id: Uuid) -> Result<String, String> {
            Ok("not-used-for-mailto".to_owned())
        }

        fn set_smtp_password(&self, _workspace_id: Uuid, _password: &str) -> Result<(), String> {
            Ok(())
        }

        fn delete_smtp_password(&self, _workspace_id: Uuid) -> Result<(), String> {
            Ok(())
        }

        fn smtp_password_exists(&self, _workspace_id: Uuid) -> Result<bool, String> {
            Ok(false)
        }
    }

    #[derive(Default)]
    struct FakeSmtpSender {
        observed: Mutex<Option<(String, String, String, String)>>,
    }

    impl SmtpSender for FakeSmtpSender {
        fn send(
            &self,
            settings: &dms_core::SmtpSettings,
            password: String,
            message: &Message,
        ) -> Result<DeliveryReceipt, String> {
            *self.observed.lock().expect("SMTP observation") = Some((
                settings.login_user.clone(),
                settings.from_mailbox.clone(),
                password,
                String::from_utf8(message.formatted()).expect("formatted message"),
            ));
            Ok(DeliveryReceipt::accepted(250, "fake accepted"))
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

    #[test]
    fn confirmed_mailto_does_not_open_a_second_compose_window() {
        fn unexpected_handler(_uri: &str) -> Result<(), String> {
            Err("the confirmation must not open mail again".to_owned())
        }

        let mut notifier =
            DesktopNotifier::new(FakeCredentials, Uuid::new_v4(), true, unexpected_handler);

        let receipt = notifier.send(&mailto_settings(), &message()).unwrap();

        assert_eq!(receipt.status, DeliveryStatus::Confirmed);
    }

    #[test]
    fn smtp_uses_login_user_only_for_auth_and_formatted_from_mailbox_for_message() {
        let mut notifier = DesktopNotifier {
            credentials: FakeCredentials,
            smtp_sender: FakeSmtpSender::default(),
            workspace_id: Uuid::new_v4(),
            mailto_confirmed: false,
            open_uri: host_mail_handler,
        };
        let settings = NotificationSettings {
            transport: NotificationTransport::Smtp,
            smtp: Some(dms_core::SmtpSettings {
                relay_host: "smtp.example.test".to_owned(),
                relay_port: 587,
                login_user: "smtp-login@example.test".to_owned(),
                from_mailbox: "\"Doc Mgmt\" <sender@example.test>".to_owned(),
            }),
        };

        let receipt = notifier
            .send(&settings, &message())
            .expect("fake SMTP delivery");

        assert_eq!(receipt.response_code, Some(250));
        let observed = notifier
            .smtp_sender
            .observed
            .lock()
            .expect("SMTP observation")
            .clone()
            .expect("SMTP delivery");
        assert_eq!(observed.0, "smtp-login@example.test");
        assert_eq!(observed.1, "\"Doc Mgmt\" <sender@example.test>");
        assert_eq!(observed.2, "not-used-for-mailto");
        assert!(observed
            .3
            .contains("From: \"Doc Mgmt\" <sender@example.test>"));
        assert!(observed.3.contains("To: approver@example.test"));
    }
}
