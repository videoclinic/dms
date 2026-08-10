use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use zip::ZipArchive;

use crate::{
    default_author, write_workspace_metadata_atomic, BackupManifest, DmsError, Result, Workspace,
    METADATA_DIRECTORY, METADATA_FILENAME,
};

pub const LOCK_FILENAME: &str = "lock";
pub const DEFAULT_LOCK_STALE_AFTER_HOURS: u32 = 24;

pub(crate) fn default_lock_stale_after_hours() -> u32 {
    DEFAULT_LOCK_STALE_AFTER_HOURS
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceLock {
    pub os_user: String,
    pub hostname: String,
    pub process_id: u32,
    pub acquired_at: DateTime<Utc>,
}

impl WorkspaceLock {
    pub fn current() -> Self {
        Self {
            os_user: default_author(),
            hostname: env::var("COMPUTERNAME")
                .or_else(|_| env::var("HOSTNAME"))
                .ok()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "unknown-host".to_owned()),
            process_id: std::process::id(),
            acquired_at: Utc::now(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceLockState {
    Unlocked,
    Current,
    Stale,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceLockStatus {
    pub state: WorkspaceLockState,
    pub stale_after_hours: u32,
    pub lock: Option<WorkspaceLock>,
}

#[derive(Clone, Debug)]
pub struct RestoreRequest<'a> {
    pub archive_path: &'a Path,
    pub edit_root: &'a Path,
    pub publish_root: &'a Path,
    pub replace_existing: bool,
    pub take_over_stale_lock: bool,
    pub confirmed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestoreOutcome {
    pub workspace_id: Uuid,
    pub edit_root: PathBuf,
    pub publish_root: PathBuf,
    pub manifest_digest: String,
    pub entry_count: usize,
}

impl Workspace {
    pub fn lock_stale_after_hours(&self) -> u32 {
        self.lock_stale_after_hours
    }

    pub fn configure_lock_staleness(&mut self, hours: u32) -> Result<()> {
        if hours == 0 {
            return Err(DmsError::InvalidLockStaleness);
        }
        self.lock_stale_after_hours = hours;
        self.save()
    }

    pub fn lock_status(&self) -> Result<WorkspaceLockStatus> {
        workspace_lock_status_at(&self.edit_root, self.lock_stale_after_hours, Utc::now())
    }

    pub fn acquire_lock(&self, take_over_stale: bool) -> Result<WorkspaceLockStatus> {
        acquire_workspace_lock_at(
            &self.edit_root,
            self.lock_stale_after_hours,
            WorkspaceLock::current(),
            take_over_stale,
        )
    }

    pub fn release_lock(&self) -> Result<()> {
        release_workspace_lock(&self.edit_root)
    }
}

pub fn workspace_lock_status(
    edit_root: &Path,
    stale_after_hours: u32,
) -> Result<WorkspaceLockStatus> {
    workspace_lock_status_at(edit_root, stale_after_hours, Utc::now())
}

pub fn acquire_workspace_lock(
    edit_root: &Path,
    stale_after_hours: u32,
    take_over_stale: bool,
) -> Result<WorkspaceLockStatus> {
    acquire_workspace_lock_at(
        edit_root,
        stale_after_hours,
        WorkspaceLock::current(),
        take_over_stale,
    )
}

pub fn release_workspace_lock(edit_root: &Path) -> Result<()> {
    let path = lock_path(edit_root);
    let metadata = fs::symlink_metadata(&path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            DmsError::WorkspaceLockNotFound(path.clone())
        } else {
            DmsError::Io {
                path: path.clone(),
                source,
            }
        }
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(DmsError::InvalidWorkspaceLock(path));
    }
    fs::remove_file(&path).map_err(|source| DmsError::Io { path, source })
}

pub fn release_workspace_lock_owned(edit_root: &Path, owner: &WorkspaceLock) -> Result<()> {
    let status = workspace_lock_status(edit_root, DEFAULT_LOCK_STALE_AFTER_HOURS)?;
    if status.lock.as_ref() != Some(owner) {
        return Err(DmsError::WorkspaceLockOwnershipChanged);
    }
    release_workspace_lock(edit_root)
}

pub fn restore_workspace_backup(request: RestoreRequest<'_>) -> Result<RestoreOutcome> {
    if !request.confirmed {
        return Err(DmsError::RestoreConfirmationRequired);
    }
    let edit_root = canonical_restore_root(request.edit_root, "replacement edit root")?;
    let publish_root = canonical_restore_root(request.publish_root, "replacement publish root")?;
    if edit_root == publish_root
        || edit_root.starts_with(&publish_root)
        || publish_root.starts_with(&edit_root)
    {
        return Err(DmsError::RestoreRootConflict);
    }

    let archive_file = fs::File::open(request.archive_path).map_err(|source| DmsError::Io {
        path: request.archive_path.to_path_buf(),
        source,
    })?;
    let mut archive = ZipArchive::new(archive_file)
        .map_err(|error| DmsError::RestoreArchive(error.to_string()))?;
    let manifest_bytes = read_manifest(&mut archive)?;
    let manifest: BackupManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| DmsError::RestoreArchive(format!("manifest.json is invalid: {error}")))?;
    let manifest_digest = digest_bytes(&manifest_bytes);
    let mut expected = BTreeMap::new();
    let mut portable_paths = BTreeSet::new();
    for entry in &manifest.entries {
        validate_archive_path(&entry.archive_path)?;
        if !portable_paths.insert(portable_archive_key(&entry.archive_path)) {
            return Err(DmsError::RestoreArchive(format!(
                "manifest contains a cross-platform path collision at {}",
                entry.archive_path
            )));
        }
        if expected.insert(entry.archive_path.clone(), entry).is_some() {
            return Err(DmsError::RestoreArchive(format!(
                "manifest contains duplicate path {}",
                entry.archive_path
            )));
        }
    }

    let mut restored = BTreeMap::new();
    let mut archive_names = BTreeSet::new();
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| DmsError::RestoreArchive(error.to_string()))?;
        let name = file.name().to_owned();
        if !archive_names.insert(name.clone()) {
            return Err(DmsError::RestoreArchive(format!(
                "archive contains duplicate path {name}"
            )));
        }
        if name == "manifest.json" {
            continue;
        }
        validate_archive_path(&name)?;
        if file.is_dir()
            || file
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(DmsError::RestoreArchive(format!(
                "archive entry {name} is not a regular file"
            )));
        }
        let expected_entry = expected.get(&name).ok_or_else(|| {
            DmsError::RestoreArchive(format!("archive entry {name} is not in the manifest"))
        })?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|error| DmsError::RestoreArchive(error.to_string()))?;
        if bytes.len() as u64 != expected_entry.size
            || digest_bytes(&bytes) != expected_entry.sha256
        {
            return Err(DmsError::RestoreArchive(format!(
                "archive entry {name} does not match its manifest size and digest"
            )));
        }
        restored.insert(name, bytes);
    }
    if restored.len() != expected.len() || expected.keys().any(|path| !restored.contains_key(path))
    {
        return Err(DmsError::RestoreArchive(
            "archive files do not match the manifest".to_owned(),
        ));
    }

    let metadata_name = format!("edit/{METADATA_DIRECTORY}/{METADATA_FILENAME}");
    let metadata_bytes = restored
        .get_mut(&metadata_name)
        .ok_or_else(|| DmsError::RestoreArchive(format!("archive is missing {metadata_name}")))?;
    let mut workspace: Workspace = serde_json::from_slice(metadata_bytes).map_err(|error| {
        DmsError::RestoreArchive(format!("workspace metadata is invalid: {error}"))
    })?;
    if workspace.workspace_id != manifest.workspace_id {
        return Err(DmsError::RestoreArchive(
            "manifest workspace ID does not match workspace metadata".to_owned(),
        ));
    }
    workspace.edit_root = edit_root.clone();
    workspace.publish_root = publish_root.clone();
    *metadata_bytes = serde_json::to_vec_pretty(&workspace).map_err(|error| {
        DmsError::RestoreArchive(format!("cannot rewrite restored workspace roots: {error}"))
    })?;

    verify_restore_lock_available(
        &edit_root,
        restore_target_stale_after_hours(&edit_root),
        request.take_over_stale_lock,
    )?;

    let mut targets = Vec::new();
    for (archive_path, bytes) in restored {
        let (root, relative) = restore_target(&edit_root, &publish_root, &archive_path)?;
        let target = root.join(relative);
        validate_restore_target(root, &target, request.replace_existing)?;
        targets.push((
            archive_path == metadata_name,
            root.to_path_buf(),
            target,
            bytes,
        ));
    }
    targets.sort_by_key(|(metadata, _, target, _)| (*metadata, target.clone()));
    for (_, root, target, _) in &targets {
        create_restore_parents(root, target.parent().expect("restored file has parent"))?;
    }
    let restore_lock = acquire_workspace_lock(
        &edit_root,
        restore_target_stale_after_hours(&edit_root),
        request.take_over_stale_lock,
    )?;
    let restore_owner = restore_lock
        .lock
        .expect("acquired workspace lock has owner");
    let restore_result = (|| -> Result<RestoreOutcome> {
        for (metadata, root, target, bytes) in targets {
            validate_restore_target(&root, &target, request.replace_existing)?;
            if metadata {
                write_workspace_metadata_atomic(&edit_root, &bytes)?;
                continue;
            }
            let mut options = fs::OpenOptions::new();
            options.write(true);
            if request.replace_existing {
                options.create(true).truncate(true);
            } else {
                options.create_new(true);
            }
            let mut output = options.open(&target).map_err(|source| DmsError::Io {
                path: target.clone(),
                source,
            })?;
            output.write_all(&bytes).map_err(|source| DmsError::Io {
                path: target.clone(),
                source,
            })?;
            output.sync_all().map_err(|source| DmsError::Io {
                path: target,
                source,
            })?;
        }

        let restored_workspace = Workspace::open(&edit_root)?;
        Ok(RestoreOutcome {
            workspace_id: restored_workspace.workspace_id,
            edit_root: edit_root.clone(),
            publish_root: publish_root.clone(),
            manifest_digest,
            entry_count: manifest.entries.len(),
        })
    })();
    let release_result = release_workspace_lock_owned(&edit_root, &restore_owner);
    match restore_result {
        Ok(outcome) => release_result.map(|()| outcome),
        Err(error) => {
            let _ = release_result;
            Err(error)
        }
    }
}

pub(crate) fn workspace_lock_status_at(
    edit_root: &Path,
    stale_after_hours: u32,
    now: DateTime<Utc>,
) -> Result<WorkspaceLockStatus> {
    if stale_after_hours == 0 {
        return Err(DmsError::InvalidLockStaleness);
    }
    let path = lock_path(edit_root);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(WorkspaceLockStatus {
                state: WorkspaceLockState::Unlocked,
                stale_after_hours,
                lock: None,
            });
        }
        Err(source) => return Err(DmsError::Io { path, source }),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(DmsError::InvalidWorkspaceLock(path));
    }
    let bytes = fs::read(&path).map_err(|source| DmsError::Io {
        path: path.clone(),
        source,
    })?;
    let lock: WorkspaceLock = serde_json::from_slice(&bytes)
        .map_err(|error| DmsError::InvalidWorkspaceLockData(path, error.to_string()))?;
    let stale = now.signed_duration_since(lock.acquired_at)
        >= Duration::hours(i64::from(stale_after_hours));
    Ok(WorkspaceLockStatus {
        state: if stale {
            WorkspaceLockState::Stale
        } else {
            WorkspaceLockState::Current
        },
        stale_after_hours,
        lock: Some(lock),
    })
}

pub(crate) fn acquire_workspace_lock_at(
    edit_root: &Path,
    stale_after_hours: u32,
    owner: WorkspaceLock,
    take_over_stale: bool,
) -> Result<WorkspaceLockStatus> {
    let path = lock_path(edit_root);
    let status = workspace_lock_status_at(edit_root, stale_after_hours, owner.acquired_at)?;
    match status.state {
        WorkspaceLockState::Current => return Err(DmsError::WorkspaceLocked),
        WorkspaceLockState::Stale if !take_over_stale => {
            return Err(DmsError::StaleWorkspaceLockTakeoverRequired)
        }
        WorkspaceLockState::Unlocked | WorkspaceLockState::Stale => {}
    }
    let bytes = serde_json::to_vec_pretty(&owner)
        .map_err(|error| DmsError::InvalidWorkspaceLockData(path.clone(), error.to_string()))?;
    let displaced = if status.state == WorkspaceLockState::Stale {
        let displaced = edit_root
            .join(METADATA_DIRECTORY)
            .join(format!(".workspace-lock-takeover-{}.tmp", Uuid::new_v4()));
        fs::rename(&path, &displaced).map_err(|source| DmsError::Io {
            path: path.clone(),
            source,
        })?;
        Some(displaced)
    } else {
        None
    };
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = match options.open(&path) {
        Ok(file) => file,
        Err(source) => {
            if let Some(displaced) = displaced {
                let _ = fs::remove_file(displaced);
            }
            return Err(DmsError::Io { path, source });
        }
    };
    let result = file
        .write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|source| DmsError::Io {
            path: path.clone(),
            source,
        });
    if result.is_err() {
        let _ = fs::remove_file(&path);
    }
    if let Some(displaced) = displaced {
        let _ = fs::remove_file(displaced);
    }
    result?;
    Ok(WorkspaceLockStatus {
        state: WorkspaceLockState::Current,
        stale_after_hours,
        lock: Some(owner),
    })
}

fn read_manifest(archive: &mut ZipArchive<fs::File>) -> Result<Vec<u8>> {
    let mut manifest = archive
        .by_name("manifest.json")
        .map_err(|_| DmsError::RestoreArchive("archive is missing manifest.json".to_owned()))?;
    if manifest.is_dir()
        || manifest
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
    {
        return Err(DmsError::RestoreArchive(
            "manifest.json is not a regular file".to_owned(),
        ));
    }
    let mut bytes = Vec::new();
    manifest
        .read_to_end(&mut bytes)
        .map_err(|error| DmsError::RestoreArchive(error.to_string()))?;
    Ok(bytes)
}

pub(crate) fn validate_archive_path(path: &str) -> Result<()> {
    if path.is_empty() || path.contains('\\') {
        return Err(DmsError::RestoreArchive(format!(
            "archive path {path:?} is unsafe"
        )));
    }
    let mut parts = path.split('/');
    let scope = parts.next().unwrap_or_default();
    if !matches!(scope, "edit" | "publish") {
        return Err(DmsError::RestoreArchive(format!(
            "archive path {path:?} has an unknown root"
        )));
    }
    if parts.clone().next().is_none()
        || parts.any(|part| {
            part.is_empty() || matches!(part, "." | "..") || !is_portable_archive_component(part)
        })
    {
        return Err(DmsError::RestoreArchive(format!(
            "archive path {path:?} is unsafe"
        )));
    }
    let portable_path = portable_archive_key(path);
    if portable_path == format!("edit/{METADATA_DIRECTORY}/{LOCK_FILENAME}") {
        return Err(DmsError::RestoreArchive(
            "workspace lock files must not be restored".to_owned(),
        ));
    }
    if portable_path
        .strip_prefix(&format!("edit/{METADATA_DIRECTORY}/.workspace-"))
        .is_some_and(|suffix| suffix.ends_with(".tmp"))
    {
        return Err(DmsError::RestoreArchive(
            "workspace metadata temporary files must not be restored".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) fn portable_archive_key(path: &str) -> String {
    path.to_lowercase()
}

fn is_portable_archive_component(component: &str) -> bool {
    if component.ends_with([' ', '.'])
        || component
            .chars()
            .any(|character| character.is_control() || r#"<>:"|?*"#.contains(character))
    {
        return false;
    }
    let stem = component.split('.').next().unwrap_or_default();
    let upper = stem.to_ascii_uppercase();
    let numbered_device = |prefix| {
        upper.strip_prefix(prefix).is_some_and(|suffix| {
            matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
        })
    };
    !matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        && !numbered_device("COM")
        && !numbered_device("LPT")
}

fn restore_target<'a>(
    edit_root: &'a Path,
    publish_root: &'a Path,
    archive_path: &str,
) -> Result<(&'a Path, PathBuf)> {
    validate_archive_path(archive_path)?;
    let (scope, relative) = archive_path
        .split_once('/')
        .expect("validated archive path has scope");
    let relative = PathBuf::from(relative);
    if relative
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(DmsError::RestoreArchive(format!(
            "archive path {archive_path:?} is unsafe"
        )));
    }
    Ok((
        if scope == "edit" {
            edit_root
        } else {
            publish_root
        },
        relative,
    ))
}

fn canonical_restore_root(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| DmsError::ExpectedDirectory(label.to_owned()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(DmsError::ExpectedDirectory(label.to_owned()));
    }
    fs::canonicalize(path).map_err(|source| DmsError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn validate_restore_target(root: &Path, target: &Path, replace_existing: bool) -> Result<()> {
    if !target.starts_with(root) {
        return Err(DmsError::RestoreArchive(format!(
            "restore target {} is outside {}",
            target.display(),
            root.display()
        )));
    }
    let mut current = root.to_path_buf();
    let relative = target
        .strip_prefix(root)
        .expect("target prefix checked above");
    for component in relative.components() {
        let Component::Normal(part) = component else {
            return Err(DmsError::RestoreArchive(format!(
                "restore target {} is unsafe",
                target.display()
            )));
        };
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(DmsError::RestoreTargetInvalid(current))
            }
            Ok(metadata) if current == target && !metadata.is_file() => {
                return Err(DmsError::RestoreTargetInvalid(current))
            }
            Ok(metadata) if current != target && !metadata.is_dir() => {
                return Err(DmsError::RestoreTargetInvalid(current))
            }
            Ok(_) if current == target && !replace_existing => {
                return Err(DmsError::RestorePathExists(current))
            }
            Ok(_) | Err(_) => {}
        }
    }
    Ok(())
}

fn create_restore_parents(root: &Path, parent: &Path) -> Result<()> {
    let relative = parent
        .strip_prefix(root)
        .map_err(|_| DmsError::RestoreTargetInvalid(parent.to_path_buf()))?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            return Err(DmsError::RestoreTargetInvalid(parent.to_path_buf()));
        };
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(DmsError::RestoreTargetInvalid(current))
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&current).map_err(|source| DmsError::Io {
                    path: current.clone(),
                    source,
                })?;
            }
            Err(source) => {
                return Err(DmsError::Io {
                    path: current,
                    source,
                })
            }
        }
    }
    Ok(())
}

fn verify_restore_lock_available(
    edit_root: &Path,
    stale_after_hours: u32,
    take_over_stale: bool,
) -> Result<()> {
    let status = workspace_lock_status(edit_root, stale_after_hours)?;
    match status.state {
        WorkspaceLockState::Unlocked => Ok(()),
        WorkspaceLockState::Stale if take_over_stale => Ok(()),
        WorkspaceLockState::Current => Err(DmsError::WorkspaceLocked),
        WorkspaceLockState::Stale => Err(DmsError::StaleWorkspaceLockTakeoverRequired),
    }
}

fn restore_target_stale_after_hours(edit_root: &Path) -> u32 {
    let metadata_path = edit_root.join(METADATA_DIRECTORY).join(METADATA_FILENAME);
    fs::read(&metadata_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|metadata| {
            metadata
                .get("lock_stale_after_hours")
                .and_then(serde_json::Value::as_u64)
        })
        .and_then(|hours| u32::try_from(hours).ok())
        .filter(|hours| *hours > 0)
        .unwrap_or(DEFAULT_LOCK_STALE_AFTER_HOURS)
}

fn lock_path(edit_root: &Path) -> PathBuf {
    edit_root.join(METADATA_DIRECTORY).join(LOCK_FILENAME)
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
