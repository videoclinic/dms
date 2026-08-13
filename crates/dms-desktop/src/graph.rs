use std::{
    collections::BTreeMap,
    thread,
    time::{Duration, Instant},
};

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use dms_core::{AuthenticatedActor, EntraIdentitySource, EntraPerson, GraphClient};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const KEYRING_SERVICE: &str = "dms-desktop";
const ENTRA_TOKEN_PURPOSE: &str = "entra-delegated-token";
// Windows Credential Manager caps the UTF-16 password blob at 2,560 bytes.
const TOKEN_CHUNK_UTF16_LIMIT: usize = 1_024;
const MAX_TOKEN_CHUNKS: usize = 128;
const GRAPH_SCOPE: &str = "openid profile offline_access User.Read.All GroupMember.Read.All";
const GRAPH_API: &str = "https://graph.microsoft.com/v1.0";
const GRAPH_LIMITED_USER_INFORMATION_ERROR: &str =
    "Microsoft Graph returned limited information for a direct user member; verify delegated User.Read.All tenant-admin consent and the signed-in user's permission to read group members, then sign in again";
const OVERSIZED_CREDENTIAL_FRAGMENT_ERROR: &str =
    "cannot save the delegated Microsoft Entra token in the OS credential store: credential fragment exceeds the supported UTF-16 size limit";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeEntraConfiguration {
    pub client_id: String,
    pub tenant_id: Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeviceLoginChallenge {
    pub challenge_id: Uuid,
    pub user_code: String,
    pub verification_uri: String,
    pub message: String,
    pub expires_in_seconds: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct IdentitySourcePreview {
    pub preview_id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_display: String,
    pub group_id: Uuid,
    pub group_label: String,
    pub eligible_people: Vec<EntraPerson>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DelegatedToken {
    access_token: String,
    refresh_token: String,
    expires_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DelegatedTokenManifest {
    version: u8,
    generation: Uuid,
    chunk_count: usize,
}

#[derive(Clone, Debug)]
struct PendingDeviceLogin {
    tenant_id: Uuid,
    group_id: Option<Uuid>,
    device_code: String,
    expires_at: Instant,
    poll_interval: Duration,
}

#[derive(Clone, Debug)]
struct PreparedPreview {
    tenant_id: Uuid,
    tenant_display: String,
    group_id: Uuid,
    group_label: String,
    people: Vec<EntraPerson>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    #[serde(default = "default_poll_interval")]
    interval: u64,
    message: String,
}

#[derive(Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
}

#[derive(Deserialize)]
struct OAuthErrorResponse {
    error: Option<String>,
}

#[derive(Deserialize)]
struct GraphGroup {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct GraphOrganization {
    id: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct GraphOrganizations {
    value: Vec<GraphOrganization>,
}

#[derive(Deserialize)]
struct GraphUser {
    id: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    mail: Option<String>,
    #[serde(rename = "userPrincipalName")]
    user_principal_name: Option<String>,
    #[serde(rename = "accountEnabled")]
    account_enabled: Option<bool>,
}

#[derive(Deserialize)]
struct GraphUsers {
    value: Vec<GraphUser>,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
}

#[derive(Deserialize)]
struct GraphActor {
    id: String,
}

fn default_poll_interval() -> u64 {
    5
}

pub(crate) trait TokenStore: Send {
    fn load(&self, tenant_id: Uuid) -> Result<Option<DelegatedToken>, String>;
    fn save(&self, tenant_id: Uuid, token: &DelegatedToken) -> Result<(), String>;
}

trait TokenCredentials {
    fn get(&self, account: &str) -> Result<Option<String>, String>;
    fn set(&self, account: &str, password: &str) -> Result<(), String>;
    fn delete(&self, account: &str) -> Result<(), String>;
}

struct OsTokenCredentials;

impl TokenCredentials for OsTokenCredentials {
    fn get(&self, account: &str) -> Result<Option<String>, String> {
        match credential_entry(account)?.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(format!(
                "cannot access the OS credential store for the delegated Microsoft Entra token: {error}"
            )),
        }
    }

    fn set(&self, account: &str, password: &str) -> Result<(), String> {
        validate_os_credential_password(password)?;
        credential_entry(account)?.set_password(password).map_err(|error| {
            format!("cannot save the delegated Microsoft Entra token in the OS credential store: {error}")
        })
    }

    fn delete(&self, account: &str) -> Result<(), String> {
        match credential_entry(account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(format!(
                "cannot delete the delegated Microsoft Entra token from the OS credential store: {error}"
            )),
        }
    }
}

#[derive(Default)]
pub(crate) struct OsTokenStore;

impl TokenStore for OsTokenStore {
    fn load(&self, tenant_id: Uuid) -> Result<Option<DelegatedToken>, String> {
        load_delegated_token_with_credentials(&OsTokenCredentials, tenant_id)
    }

    fn save(&self, tenant_id: Uuid, token: &DelegatedToken) -> Result<(), String> {
        save_delegated_token_with_credentials(&OsTokenCredentials, tenant_id, token)
    }
}

fn credential_entry(account: &str) -> Result<Entry, String> {
    Entry::new(KEYRING_SERVICE, account)
        .map_err(|error| format!("cannot access the OS credential store: {error}"))
}

fn delegated_token_account(tenant_id: Uuid) -> String {
    format!("{tenant_id}/{ENTRA_TOKEN_PURPOSE}")
}

fn delegated_token_chunk_account(tenant_id: Uuid, generation: Uuid, index: usize) -> String {
    format!(
        "{}/{generation}/{index}",
        delegated_token_account(tenant_id)
    )
}

fn validate_os_credential_password(password: &str) -> Result<(), String> {
    if password.encode_utf16().count() > TOKEN_CHUNK_UTF16_LIMIT {
        return Err(OVERSIZED_CREDENTIAL_FRAGMENT_ERROR.to_owned());
    }
    Ok(())
}

fn load_delegated_token_with_credentials<C: TokenCredentials>(
    credentials: &C,
    tenant_id: Uuid,
) -> Result<Option<DelegatedToken>, String> {
    let Some(primary) = credentials.get(&delegated_token_account(tenant_id))? else {
        return Ok(None);
    };
    let Ok(manifest) = serde_json::from_str::<DelegatedTokenManifest>(&primary) else {
        return parse_delegated_token(&primary).map(Some);
    };
    if manifest.version != 1 || !(1..=MAX_TOKEN_CHUNKS).contains(&manifest.chunk_count) {
        return invalid_delegated_token_cache();
    }
    let mut serialized = String::new();
    for index in 0..manifest.chunk_count {
        let chunk = credentials
            .get(&delegated_token_chunk_account(
                tenant_id,
                manifest.generation,
                index,
            ))
            .map_err(|_| invalid_delegated_token_cache_error())?
            .ok_or_else(invalid_delegated_token_cache_error)?;
        if chunk.encode_utf16().count() > TOKEN_CHUNK_UTF16_LIMIT {
            return invalid_delegated_token_cache();
        }
        serialized.push_str(&chunk);
    }
    parse_delegated_token(&serialized).map(Some)
}

fn save_delegated_token_with_credentials<C: TokenCredentials>(
    credentials: &C,
    tenant_id: Uuid,
    token: &DelegatedToken,
) -> Result<(), String> {
    let serialized = serde_json::to_string(token)
        .map_err(|error| format!("cannot serialize delegated Microsoft Entra token: {error}"))?;
    let chunks = split_utf16_safe(&serialized, TOKEN_CHUNK_UTF16_LIMIT);
    if chunks.is_empty() || chunks.len() > MAX_TOKEN_CHUNKS {
        return Err(
            "cannot save the delegated Microsoft Entra token: token is too large".to_owned(),
        );
    }
    let primary_account = delegated_token_account(tenant_id);
    let previous_manifest = credentials
        .get(&primary_account)?
        .and_then(|primary| serde_json::from_str::<DelegatedTokenManifest>(&primary).ok())
        .filter(|manifest| manifest.version == 1);
    let generation = Uuid::new_v4();
    let mut saved_accounts: Vec<String> = Vec::with_capacity(chunks.len());
    for (index, chunk) in chunks.iter().enumerate() {
        let account = delegated_token_chunk_account(tenant_id, generation, index);
        if let Err(error) = credentials.set(&account, chunk) {
            for account in saved_accounts {
                let _ = credentials.delete(&account);
            }
            return Err(error);
        }
        saved_accounts.push(account);
    }
    let manifest = serde_json::to_string(&DelegatedTokenManifest {
        version: 1,
        generation,
        chunk_count: chunks.len(),
    })
    .map_err(|error| format!("cannot serialize delegated Microsoft Entra token: {error}"))?;
    if let Err(error) = credentials.set(&primary_account, &manifest) {
        for account in saved_accounts {
            let _ = credentials.delete(&account);
        }
        return Err(error);
    }
    if let Some(previous_manifest) = previous_manifest {
        for index in 0..previous_manifest.chunk_count.min(MAX_TOKEN_CHUNKS) {
            let _ = credentials.delete(&delegated_token_chunk_account(
                tenant_id,
                previous_manifest.generation,
                index,
            ));
        }
    }
    Ok(())
}

fn split_utf16_safe(value: &str, limit: usize) -> Vec<&str> {
    let mut chunks = Vec::new();
    let mut chunk_start = 0;
    let mut chunk_units = 0;
    for (index, character) in value.char_indices() {
        let character_units = character.len_utf16();
        if chunk_units > 0 && chunk_units + character_units > limit {
            chunks.push(&value[chunk_start..index]);
            chunk_start = index;
            chunk_units = 0;
        }
        chunk_units += character_units;
    }
    if chunk_start < value.len() {
        chunks.push(&value[chunk_start..]);
    }
    chunks
}

fn parse_delegated_token(serialized: &str) -> Result<DelegatedToken, String> {
    serde_json::from_str(serialized).map_err(|_| invalid_delegated_token_cache_error())
}

fn invalid_delegated_token_cache<T>() -> Result<T, String> {
    Err(invalid_delegated_token_cache_error())
}

fn invalid_delegated_token_cache_error() -> String {
    "the delegated Microsoft Entra token cache is invalid; sign in again".to_owned()
}

pub(crate) trait HttpClient: Send {
    fn get(
        &mut self,
        url: &str,
        bearer: Option<&str>,
        eventual_consistency: bool,
    ) -> Result<HttpResponse, String>;
    fn post_form(&mut self, url: &str, form: &[(&str, &str)]) -> Result<HttpResponse, String>;
}

pub(crate) struct HttpResponse {
    status: u16,
    body: String,
}

#[derive(Default)]
pub(crate) struct UreqHttpClient;

impl HttpClient for UreqHttpClient {
    fn get(
        &mut self,
        url: &str,
        bearer: Option<&str>,
        eventual_consistency: bool,
    ) -> Result<HttpResponse, String> {
        let mut request = ureq::get(url).config().http_status_as_error(false).build();
        if let Some(token) = bearer {
            request = request.header("Authorization", &format!("Bearer {token}"));
        }
        if eventual_consistency {
            request = request.header("ConsistencyLevel", "eventual");
        }
        read_response(request.call())
    }

    fn post_form(&mut self, url: &str, form: &[(&str, &str)]) -> Result<HttpResponse, String> {
        read_response(
            ureq::post(url)
                .config()
                .http_status_as_error(false)
                .build()
                .send_form(form.iter().copied()),
        )
    }
}

fn read_response(
    response: Result<ureq::http::Response<ureq::Body>, ureq::Error>,
) -> Result<HttpResponse, String> {
    let mut response =
        response.map_err(|error| format!("Microsoft Entra or Graph request failed: {error}"))?;
    let status = response.status().as_u16();
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|error| format!("cannot read Microsoft Entra or Graph response: {error}"))?;
    Ok(HttpResponse { status, body })
}

pub(crate) struct MicrosoftGraphClient<H = UreqHttpClient, S = OsTokenStore> {
    client_id: Option<String>,
    tenant_id: Option<Uuid>,
    http: H,
    tokens: S,
    pending: BTreeMap<Uuid, PendingDeviceLogin>,
    previews: BTreeMap<Uuid, PreparedPreview>,
}

impl MicrosoftGraphClient {
    pub fn production(configuration: Option<RuntimeEntraConfiguration>) -> Self {
        Self {
            client_id: configuration.as_ref().map(|value| value.client_id.clone()),
            tenant_id: configuration.map(|value| value.tenant_id),
            http: UreqHttpClient,
            tokens: OsTokenStore,
            pending: BTreeMap::new(),
            previews: BTreeMap::new(),
        }
    }
}

impl<H, S> MicrosoftGraphClient<H, S>
where
    H: HttpClient,
    S: TokenStore,
{
    #[cfg(test)]
    fn with_parts(client_id: &str, tenant_id: Uuid, http: H, tokens: S) -> Self {
        Self {
            client_id: Some(client_id.to_owned()),
            tenant_id: Some(tenant_id),
            http,
            tokens,
            pending: BTreeMap::new(),
            previews: BTreeMap::new(),
        }
    }

    pub fn begin_identity_source_setup(
        &mut self,
        group_id: Uuid,
    ) -> Result<DeviceLoginChallenge, String> {
        self.begin_delegated_sign_in(self.configured_tenant_id()?, Some(group_id))
    }

    pub fn begin_approver_sign_in(
        &mut self,
        tenant_id: Uuid,
    ) -> Result<DeviceLoginChallenge, String> {
        self.begin_delegated_sign_in(tenant_id, None)
    }

    fn begin_delegated_sign_in(
        &mut self,
        tenant_id: Uuid,
        group_id: Option<Uuid>,
    ) -> Result<DeviceLoginChallenge, String> {
        let client_id = self.client_id()?.to_owned();
        let response = self.http.post_form(
            &oauth_endpoint(tenant_id, "devicecode"),
            &[("client_id", &client_id), ("scope", GRAPH_SCOPE)],
        )?;
        let device =
            parse_success::<DeviceCodeResponse>(response, "start Microsoft Entra sign-in")?;
        let challenge_id = Uuid::new_v4();
        self.pending.insert(
            challenge_id,
            PendingDeviceLogin {
                tenant_id,
                group_id,
                device_code: device.device_code,
                expires_at: Instant::now() + Duration::from_secs(device.expires_in),
                poll_interval: Duration::from_secs(device.interval.max(1)),
            },
        );
        Ok(DeviceLoginChallenge {
            challenge_id,
            user_code: device.user_code,
            verification_uri: device.verification_uri,
            message: device.message,
            expires_in_seconds: device.expires_in,
        })
    }

    pub fn complete_identity_source_setup(
        &mut self,
        challenge_id: Uuid,
    ) -> Result<IdentitySourcePreview, String> {
        let pending = self.pending.remove(&challenge_id).ok_or_else(|| {
            "Microsoft Entra sign-in challenge is no longer available; start again".to_owned()
        })?;
        let group_id = pending.group_id.ok_or_else(|| {
            "this sign-in is for an approval decision, not an identity-source preview".to_owned()
        })?;
        let token = self.wait_for_device_token(&pending)?;
        self.tokens.save(pending.tenant_id, &token)?;
        let preview = self.prepare_preview(pending.tenant_id, group_id, &token.access_token)?;
        let preview_id = Uuid::new_v4();
        let result = IdentitySourcePreview {
            preview_id,
            tenant_id: preview.tenant_id,
            tenant_display: preview.tenant_display.clone(),
            group_id: preview.group_id,
            group_label: preview.group_label.clone(),
            eligible_people: preview.people.clone(),
        };
        self.previews.insert(preview_id, preview);
        Ok(result)
    }

    pub fn complete_approver_sign_in(
        &mut self,
        challenge_id: Uuid,
    ) -> Result<AuthenticatedActor, String> {
        let pending = self.pending.remove(&challenge_id).ok_or_else(|| {
            "Microsoft Entra sign-in challenge is no longer available; start again".to_owned()
        })?;
        if pending.group_id.is_some() {
            return Err(
                "this sign-in is for an identity-source preview, not an approval decision"
                    .to_owned(),
            );
        }
        let token = self.wait_for_device_token(&pending)?;
        self.tokens.save(pending.tenant_id, &token)?;
        self.authenticated_actor_with_token(pending.tenant_id, &token.access_token)
    }

    pub fn apply_identity_source_preview(
        &mut self,
        preview_id: Uuid,
    ) -> Result<(Uuid, String, Uuid, String, Vec<EntraPerson>), String> {
        let preview = self.previews.remove(&preview_id).ok_or_else(|| {
            "Microsoft Entra preview is no longer available; sign in and preview again".to_owned()
        })?;
        Ok((
            preview.tenant_id,
            preview.tenant_display,
            preview.group_id,
            preview.group_label,
            preview.people,
        ))
    }

    fn client_id(&self) -> Result<&str, String> {
        self.client_id.as_deref().ok_or_else(|| {
            "this desktop build has no Microsoft Entra client ID; rebuild with DMS_ENTRA_CLIENT_ID set to the registered public-client application ID".to_owned()
        })
    }

    fn configured_tenant_id(&self) -> Result<Uuid, String> {
        self.tenant_id.ok_or_else(|| {
            "Microsoft Entra tenant ID is not configured; set it in Application Entra configuration or DMS_ENTRA_TENANT_ID".to_owned()
        })
    }

    fn wait_for_device_token(
        &mut self,
        pending: &PendingDeviceLogin,
    ) -> Result<DelegatedToken, String> {
        let client_id = self.client_id()?.to_owned();
        let mut interval = pending.poll_interval;
        loop {
            if Instant::now() >= pending.expires_at {
                return Err("Microsoft Entra sign-in expired; start again".to_owned());
            }
            let response = self.http.post_form(
                &oauth_endpoint(pending.tenant_id, "token"),
                &[
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                    ("client_id", &client_id),
                    ("device_code", &pending.device_code),
                ],
            )?;
            if (200..300).contains(&response.status) {
                return delegated_token(
                    parse_success::<OAuthTokenResponse>(
                        response,
                        "complete Microsoft Entra sign-in",
                    )?,
                    None,
                );
            }
            match oauth_error(&response.body).as_deref() {
                Some("authorization_pending") => thread::sleep(interval),
                Some("slow_down") => {
                    interval += Duration::from_secs(5);
                    thread::sleep(interval);
                }
                Some("authorization_declined") => {
                    return Err("Microsoft Entra sign-in was declined".to_owned())
                }
                Some("expired_token") => {
                    return Err("Microsoft Entra sign-in expired; start again".to_owned())
                }
                Some(error) => return Err(format!("Microsoft Entra sign-in failed: {error}")),
                None => {
                    return Err(
                        "Microsoft Entra sign-in returned an invalid error response".to_owned()
                    )
                }
            }
        }
    }

    fn token_for(&mut self, tenant_id: Uuid) -> Result<String, String> {
        let token = self.tokens.load(tenant_id)?.ok_or_else(|| {
            "sign in to Microsoft Entra before refreshing this identity source".to_owned()
        })?;
        if token.expires_at > Utc::now() + ChronoDuration::seconds(60) {
            return Ok(token.access_token);
        }
        let refreshed = self.refresh_token(tenant_id, &token.refresh_token)?;
        self.tokens.save(tenant_id, &refreshed)?;
        Ok(refreshed.access_token)
    }

    fn refresh_token(
        &mut self,
        tenant_id: Uuid,
        refresh_token: &str,
    ) -> Result<DelegatedToken, String> {
        let client_id = self.client_id()?.to_owned();
        let response = self.http.post_form(
            &oauth_endpoint(tenant_id, "token"),
            &[
                ("grant_type", "refresh_token"),
                ("client_id", &client_id),
                ("refresh_token", refresh_token),
                ("scope", GRAPH_SCOPE),
            ],
        )?;
        let response =
            parse_success::<OAuthTokenResponse>(response, "refresh Microsoft Entra sign-in")?;
        delegated_token(response, Some(refresh_token))
    }

    fn prepare_preview(
        &mut self,
        tenant_id: Uuid,
        group_id: Uuid,
        access_token: &str,
    ) -> Result<PreparedPreview, String> {
        let organization = self.graph_get::<GraphOrganizations>(
            &format!("{GRAPH_API}/organization?$select=id,displayName"),
            access_token,
            false,
        )?;
        let tenant_display = organization
            .value
            .into_iter()
            .find(|organization| organization.id.eq_ignore_ascii_case(&tenant_id.to_string()))
            .and_then(|organization| organization.display_name)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                "Microsoft Graph did not return the selected tenant display name".to_owned()
            })?;
        let group = self.graph_get::<GraphGroup>(
            &format!("{GRAPH_API}/groups/{group_id}?$select=id,displayName"),
            access_token,
            false,
        )?;
        let group_label = group
            .display_name
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                "Microsoft Graph did not return the selected group display name".to_owned()
            })?;
        let people = self.direct_user_members_with_token(group_id, access_token)?;
        Ok(PreparedPreview {
            tenant_id,
            tenant_display,
            group_id,
            group_label,
            people,
        })
    }

    fn direct_user_members_with_token(
        &mut self,
        group_id: Uuid,
        access_token: &str,
    ) -> Result<Vec<EntraPerson>, String> {
        let mut next = Some(format!(
            "{GRAPH_API}/groups/{group_id}/members/microsoft.graph.user?$select=id,displayName,mail,userPrincipalName,accountEnabled&$top=999"
        ));
        let mut people = BTreeMap::new();
        while let Some(url) = next.take() {
            let page = self.graph_get::<GraphUsers>(&url, access_token, true)?;
            for user in page.value {
                let object_id = Uuid::parse_str(&user.id).map_err(|_| {
                    "Microsoft Graph returned a user without a valid object ID".to_owned()
                })?;
                if user.display_name.is_none()
                    && user.mail.is_none()
                    && user.user_principal_name.is_none()
                    && user.account_enabled.is_none()
                {
                    return Err(GRAPH_LIMITED_USER_INFORMATION_ERROR.to_owned());
                }
                let display_name = user
                    .display_name
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| {
                        format!("Microsoft Graph returned user {object_id} without a display name")
                    })?;
                let email = user
                    .mail
                    .or(user.user_principal_name)
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| {
                        format!(
                            "Microsoft Graph returned user {object_id} without an email address"
                        )
                    })?;
                let account_enabled = user.account_enabled.ok_or_else(|| {
                    format!("Microsoft Graph did not authorize the account status required for user {object_id}")
                })?;
                if account_enabled {
                    people.insert(
                        object_id,
                        EntraPerson {
                            object_id,
                            display_name,
                            email,
                            account_enabled,
                        },
                    );
                }
            }
            next = page.next_link;
        }
        Ok(people.into_values().collect())
    }

    fn graph_get<T: for<'de> Deserialize<'de>>(
        &mut self,
        url: &str,
        access_token: &str,
        eventual_consistency: bool,
    ) -> Result<T, String> {
        let response = self
            .http
            .get(url, Some(access_token), eventual_consistency)?;
        parse_success(response, "call Microsoft Graph")
    }

    fn authenticated_actor_with_token(
        &mut self,
        tenant_id: Uuid,
        access_token: &str,
    ) -> Result<AuthenticatedActor, String> {
        let actor = self.graph_get::<GraphActor>(
            &format!("{GRAPH_API}/me?$select=id"),
            access_token,
            false,
        )?;
        let object_id = Uuid::parse_str(&actor.id).map_err(|_| {
            "Microsoft Graph returned the signed-in user without a valid object ID".to_owned()
        })?;
        Ok(AuthenticatedActor {
            tenant_id,
            object_id,
        })
    }
}

impl<H, S> GraphClient for MicrosoftGraphClient<H, S>
where
    H: HttpClient,
    S: TokenStore,
{
    fn tenant_id(&self) -> Result<Uuid, String> {
        self.configured_tenant_id()
    }

    fn direct_user_members(
        &mut self,
        source: &EntraIdentitySource,
    ) -> Result<Vec<EntraPerson>, String> {
        let token = self.token_for(self.configured_tenant_id()?)?;
        self.direct_user_members_with_token(source.group_id, &token)
    }

    fn authenticated_actor(
        &mut self,
        _source: &EntraIdentitySource,
    ) -> Result<AuthenticatedActor, String> {
        let tenant_id = self.configured_tenant_id()?;
        let token = self.token_for(tenant_id)?;
        self.authenticated_actor_with_token(tenant_id, &token)
    }
}

fn delegated_token(
    response: OAuthTokenResponse,
    previous_refresh_token: Option<&str>,
) -> Result<DelegatedToken, String> {
    let refresh_token = response
        .refresh_token
        .or_else(|| previous_refresh_token.map(str::to_owned))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            "Microsoft Entra did not issue a delegated refresh token; sign in again".to_owned()
        })?;
    Ok(DelegatedToken {
        access_token: response.access_token,
        refresh_token,
        expires_at: Utc::now() + ChronoDuration::seconds(response.expires_in.max(1)),
    })
}

fn oauth_endpoint(tenant_id: Uuid, action: &str) -> String {
    format!("https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/{action}")
}

fn parse_success<T: for<'de> Deserialize<'de>>(
    response: HttpResponse,
    action: &str,
) -> Result<T, String> {
    if !(200..300).contains(&response.status) {
        return Err(format!(
            "cannot {action}: Microsoft service returned HTTP {}",
            response.status
        ));
    }
    serde_json::from_str(&response.body)
        .map_err(|_| format!("cannot {action}: Microsoft service returned an invalid response"))
}

fn oauth_error(body: &str) -> Option<String> {
    serde_json::from_str::<OAuthErrorResponse>(body).ok()?.error
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, VecDeque},
        sync::Mutex,
    };

    use super::*;

    struct MemoryTokenStore {
        token: std::sync::Mutex<Option<DelegatedToken>>,
    }

    impl Default for MemoryTokenStore {
        fn default() -> Self {
            Self {
                token: std::sync::Mutex::new(None),
            }
        }
    }

    impl TokenStore for MemoryTokenStore {
        fn load(&self, _tenant_id: Uuid) -> Result<Option<DelegatedToken>, String> {
            Ok(self.token.lock().unwrap().clone())
        }

        fn save(&self, _tenant_id: Uuid, token: &DelegatedToken) -> Result<(), String> {
            *self.token.lock().unwrap() = Some(token.clone());
            Ok(())
        }
    }

    #[derive(Default)]
    struct MemoryTokenCredentials {
        passwords: Mutex<BTreeMap<String, String>>,
    }

    impl MemoryTokenCredentials {
        fn passwords(&self) -> BTreeMap<String, String> {
            self.passwords.lock().unwrap().clone()
        }
    }

    impl TokenCredentials for MemoryTokenCredentials {
        fn get(&self, account: &str) -> Result<Option<String>, String> {
            Ok(self.passwords.lock().unwrap().get(account).cloned())
        }

        fn set(&self, account: &str, password: &str) -> Result<(), String> {
            self.passwords
                .lock()
                .unwrap()
                .insert(account.to_owned(), password.to_owned());
            Ok(())
        }

        fn delete(&self, account: &str) -> Result<(), String> {
            self.passwords.lock().unwrap().remove(account);
            Ok(())
        }
    }

    struct FakeHttp {
        responses: VecDeque<HttpResponse>,
        calls: Vec<String>,
    }

    impl FakeHttp {
        fn with_responses(responses: Vec<HttpResponse>) -> Self {
            Self {
                responses: responses.into(),
                calls: Vec::new(),
            }
        }

        fn next(&mut self, url: &str) -> Result<HttpResponse, String> {
            self.calls.push(url.to_owned());
            self.responses
                .pop_front()
                .ok_or_else(|| "unexpected HTTP request".to_owned())
        }
    }

    impl HttpClient for FakeHttp {
        fn get(
            &mut self,
            url: &str,
            _bearer: Option<&str>,
            _eventual_consistency: bool,
        ) -> Result<HttpResponse, String> {
            self.next(url)
        }

        fn post_form(&mut self, url: &str, _form: &[(&str, &str)]) -> Result<HttpResponse, String> {
            self.next(url)
        }
    }

    fn response(status: u16, body: &str) -> HttpResponse {
        HttpResponse {
            status,
            body: body.to_owned(),
        }
    }

    #[test]
    fn device_flow_token_response_accepts_documented_optional_fields() {
        let token = parse_success::<OAuthTokenResponse>(
            response(
                200,
                r#"{"token_type":"Bearer","scope":"openid profile offline_access User.Read.All GroupMember.Read.All","expires_in":3600,"access_token":"access","id_token":"identity","refresh_token":"refresh"}"#,
            ),
            "complete Microsoft Entra sign-in",
        )
        .unwrap();

        assert_eq!(token.access_token, "access");
        assert_eq!(token.refresh_token.as_deref(), Some("refresh"));
        assert_eq!(token.expires_in, 3600);
    }

    #[test]
    fn delegated_scope_requests_user_directory_read_access() {
        assert!(GRAPH_SCOPE
            .split_ascii_whitespace()
            .any(|scope| scope == "User.Read.All"));
    }

    #[test]
    fn oversized_delegated_token_round_trips_through_chunked_credentials() {
        let tenant_id = Uuid::new_v4();
        let credentials = MemoryTokenCredentials::default();
        let token = DelegatedToken {
            access_token: "access".repeat(600),
            refresh_token: "refresh".repeat(600),
            expires_at: Utc::now(),
        };

        assert!(
            serde_json::to_string(&token)
                .unwrap()
                .encode_utf16()
                .count()
                > 2_560
        );
        save_delegated_token_with_credentials(&credentials, tenant_id, &token).unwrap();

        assert!(credentials
            .passwords()
            .values()
            .all(|password| password.encode_utf16().count() <= TOKEN_CHUNK_UTF16_LIMIT));
        assert_eq!(
            load_delegated_token_with_credentials(&credentials, tenant_id).unwrap(),
            Some(token)
        );
    }

    #[test]
    fn delegated_token_loads_legacy_single_credential() {
        let tenant_id = Uuid::new_v4();
        let credentials = MemoryTokenCredentials::default();
        let token = DelegatedToken {
            access_token: "synthetic-access".to_owned(),
            refresh_token: "synthetic-refresh".to_owned(),
            expires_at: Utc::now(),
        };
        credentials
            .set(
                &delegated_token_account(tenant_id),
                &serde_json::to_string(&token).unwrap(),
            )
            .unwrap();

        assert_eq!(
            load_delegated_token_with_credentials(&credentials, tenant_id).unwrap(),
            Some(token)
        );
    }

    #[test]
    fn token_chunks_do_not_split_unicode_scalar_boundaries() {
        let chunks = split_utf16_safe("a😀b", 2);

        assert_eq!(chunks, vec!["a", "😀", "b"]);
        assert!(chunks.iter().all(|chunk| chunk.encode_utf16().count() <= 2));
    }

    #[test]
    fn os_credential_password_rejects_values_above_the_chunk_limit() {
        let error =
            validate_os_credential_password(&"x".repeat(TOKEN_CHUNK_UTF16_LIMIT + 1)).unwrap_err();

        assert_eq!(
            error,
            "cannot save the delegated Microsoft Entra token in the OS credential store: credential fragment exceeds the supported UTF-16 size limit"
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_credential_store_accepts_a_password_at_the_chunk_limit() {
        for password in [
            "x".repeat(TOKEN_CHUNK_UTF16_LIMIT),
            "😀".repeat(TOKEN_CHUNK_UTF16_LIMIT / 2),
        ] {
            let entry = Entry::new(
                KEYRING_SERVICE,
                &format!("test/{ENTRA_TOKEN_PURPOSE}/{}", Uuid::new_v4()),
            )
            .unwrap();

            entry.set_password(&password).unwrap();
            let stored_password = entry.get_password().unwrap();
            entry.delete_credential().unwrap();

            assert_eq!(stored_password, password);
        }
    }

    #[test]
    fn setup_previews_direct_enabled_users_and_keeps_tokens_out_of_the_preview() {
        let tenant_id = Uuid::new_v4();
        let group_id = Uuid::new_v4();
        let enabled_user = Uuid::new_v4();
        let disabled_user = Uuid::new_v4();
        let http = FakeHttp::with_responses(vec![
            response(
                200,
                r#"{"device_code":"device","user_code":"ABCD-EFGH","verification_uri":"https://microsoft.com/devicelogin","expires_in":900,"interval":1,"message":"Sign in"}"#,
            ),
            response(
                200,
                r#"{"access_token":"access","refresh_token":"refresh","expires_in":3600}"#,
            ),
            response(
                200,
                &format!(r#"{{"value":[{{"id":"{tenant_id}","displayName":"Contoso"}}]}}"#),
            ),
            response(200, r#"{"displayName":"Quality group"}"#),
            response(
                200,
                &format!(
                    r#"{{"value":[{{"id":"{enabled_user}","displayName":"Alex","mail":"alex@example.test","accountEnabled":true}},{{"id":"{disabled_user}","displayName":"Disabled","mail":"disabled@example.test","accountEnabled":false}}]}}"#
                ),
            ),
        ]);
        let mut graph = MicrosoftGraphClient::with_parts(
            "client",
            tenant_id,
            http,
            MemoryTokenStore::default(),
        );

        let challenge = graph.begin_identity_source_setup(group_id).unwrap();
        assert_eq!(challenge.user_code, "ABCD-EFGH");
        let preview = graph
            .complete_identity_source_setup(challenge.challenge_id)
            .unwrap();

        assert_eq!(preview.tenant_display, "Contoso");
        assert_eq!(preview.group_label, "Quality group");
        assert_eq!(preview.eligible_people.len(), 1);
        assert_eq!(preview.eligible_people[0].object_id, enabled_user);
        assert!(!serde_json::to_string(&preview).unwrap().contains("access"));
    }

    #[test]
    fn limited_group_member_info_reports_required_user_permission() {
        let tenant_id = Uuid::new_v4();
        let group_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let http = FakeHttp::with_responses(vec![response(
            200,
            &format!(r#"{{"value":[{{"id":"{user_id}"}}]}}"#),
        )]);
        let mut graph = MicrosoftGraphClient::with_parts(
            "client",
            tenant_id,
            http,
            MemoryTokenStore::default(),
        );

        let error = graph
            .direct_user_members_with_token(group_id, "synthetic-access")
            .unwrap_err();

        assert_eq!(
            error,
            "Microsoft Graph returned limited information for a direct user member; verify delegated User.Read.All tenant-admin consent and the signed-in user's permission to read group members, then sign in again"
        );
    }

    #[test]
    fn approver_sign_in_caches_a_delegated_token_and_returns_the_graph_actor() {
        let tenant_id = Uuid::new_v4();
        let actor_id = Uuid::new_v4();
        let http = FakeHttp::with_responses(vec![
            response(
                200,
                r#"{"device_code":"device","user_code":"ABCD-EFGH","verification_uri":"https://microsoft.com/devicelogin","expires_in":900,"interval":1,"message":"Sign in"}"#,
            ),
            response(
                200,
                r#"{"access_token":"access","refresh_token":"refresh","expires_in":3600}"#,
            ),
            response(200, &format!(r#"{{"id":"{actor_id}"}}"#)),
        ]);
        let mut graph = MicrosoftGraphClient::with_parts(
            "client",
            tenant_id,
            http,
            MemoryTokenStore::default(),
        );

        let challenge = graph.begin_approver_sign_in(tenant_id).unwrap();
        let actor = graph
            .complete_approver_sign_in(challenge.challenge_id)
            .unwrap();

        assert_eq!(actor.tenant_id, tenant_id);
        assert_eq!(actor.object_id, actor_id);
    }

    #[test]
    fn refreshes_a_expired_token_before_loading_direct_users() {
        let tenant_id = Uuid::new_v4();
        let group_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let tokens = MemoryTokenStore::default();
        tokens
            .save(
                tenant_id,
                &DelegatedToken {
                    access_token: "expired".to_owned(),
                    refresh_token: "refresh".to_owned(),
                    expires_at: Utc::now() - ChronoDuration::seconds(1),
                },
            )
            .unwrap();
        let http = FakeHttp::with_responses(vec![
            response(
                200,
                r#"{"access_token":"fresh","refresh_token":"rotated","expires_in":3600}"#,
            ),
            response(
                200,
                &format!(
                    r#"{{"value":[{{"id":"{user_id}","displayName":"Alex","mail":"alex@example.test","accountEnabled":true}}]}}"#
                ),
            ),
        ]);
        let mut graph = MicrosoftGraphClient::with_parts("client", tenant_id, http, tokens);
        let source = EntraIdentitySource {
            binding_id: Uuid::new_v4(),
            group_id,
            group_label: "Quality".to_owned(),
            last_refreshed_at: None,
        };

        let people = graph.direct_user_members(&source).unwrap();
        assert_eq!(people[0].object_id, user_id);
    }
}
