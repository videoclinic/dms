use std::{fs, io::Write, path::Path};

use chrono::{Duration, Utc};
use dms_core::{
    restore_workspace_backup, BackupManifest, BackupManifestEntry, DmsError, RestoreRequest,
    Workspace, WorkspaceLock, WorkspaceLockState, DEFAULT_LOCK_STALE_AFTER_HOURS,
};
use sha2::{Digest, Sha256};
use zip::{write::SimpleFileOptions, ZipWriter};

fn initialized_workspace() -> (tempfile::TempDir, tempfile::TempDir, Workspace) {
    let edit_root = tempfile::tempdir().unwrap();
    let publish_root = tempfile::tempdir().unwrap();
    let workspace = Workspace::init(edit_root.path(), publish_root.path()).unwrap();
    (edit_root, publish_root, workspace)
}

#[test]
fn advisory_lock_requires_explicit_stale_takeover_and_release() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    assert_eq!(
        workspace.lock_status().unwrap().state,
        WorkspaceLockState::Unlocked
    );

    let acquired = workspace.acquire_lock(false).unwrap();
    assert_eq!(acquired.state, WorkspaceLockState::Current);
    let original_owner = acquired.lock.unwrap();
    assert!(matches!(
        workspace.acquire_lock(false),
        Err(DmsError::WorkspaceLocked)
    ));

    let path = edit_root.path().join(".dms/lock");
    let mut stale: WorkspaceLock = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    stale.acquired_at = Utc::now() - Duration::hours(25);
    fs::write(&path, serde_json::to_vec_pretty(&stale).unwrap()).unwrap();
    assert_eq!(
        workspace.lock_status().unwrap().state,
        WorkspaceLockState::Stale
    );
    assert!(matches!(
        workspace.acquire_lock(false),
        Err(DmsError::StaleWorkspaceLockTakeoverRequired)
    ));
    assert_eq!(
        workspace.acquire_lock(true).unwrap().state,
        WorkspaceLockState::Current
    );
    assert!(matches!(
        dms_core::release_workspace_lock_owned(edit_root.path(), &original_owner),
        Err(DmsError::WorkspaceLockOwnershipChanged)
    ));

    workspace.release_lock().unwrap();
    assert_eq!(
        workspace.lock_status().unwrap().state,
        WorkspaceLockState::Unlocked
    );
    workspace.configure_lock_staleness(48).unwrap();
    assert_eq!(
        Workspace::open(edit_root.path())
            .unwrap()
            .lock_stale_after_hours(),
        48
    );
    assert!(matches!(
        workspace.configure_lock_staleness(0),
        Err(DmsError::InvalidLockStaleness)
    ));
}

#[test]
fn schema_v7_migrates_lock_staleness_and_retains_backup() {
    let (edit_root, _publish_root, workspace) = initialized_workspace();
    let metadata_path = edit_root.path().join(".dms/workspace.json");
    let mut metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(&metadata_path).unwrap()).unwrap();
    metadata["schema_version"] = serde_json::Value::from(7);
    metadata
        .as_object_mut()
        .unwrap()
        .remove("lock_stale_after_hours");
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();

    let migrated = Workspace::open(edit_root.path()).unwrap();
    assert_eq!(
        migrated.lock_stale_after_hours(),
        DEFAULT_LOCK_STALE_AFTER_HOURS
    );
    assert_eq!(migrated.workspace_id, workspace.workspace_id);
    assert!(edit_root
        .path()
        .join(".dms/workspace.v7.json.bak")
        .is_file());
}

#[test]
fn restore_verifies_manifest_rewrites_roots_and_requires_confirmed_replacement() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    let source = edit_root.path().join("Policies/Access.md");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "# Access\n").unwrap();
    let document = workspace.add_document(&source).unwrap();
    workspace.save().unwrap();
    let backup_directory = tempfile::tempdir().unwrap();
    let archive = backup_directory.path().join("workspace-restore.zip");
    let backup = workspace.backup_workspace(&archive).unwrap();

    let destination = tempfile::tempdir().unwrap();
    let replacement_edit = destination.path().join("edit");
    let replacement_publish = destination.path().join("publish");
    fs::create_dir(&replacement_edit).unwrap();
    fs::create_dir(&replacement_publish).unwrap();

    let request = |confirmed, replace_existing| RestoreRequest {
        archive_path: &archive,
        edit_root: &replacement_edit,
        publish_root: &replacement_publish,
        replace_existing,
        take_over_stale_lock: false,
        confirmed,
    };
    assert!(matches!(
        restore_workspace_backup(request(false, false)),
        Err(DmsError::RestoreConfirmationRequired)
    ));
    assert!(!replacement_edit.join(".dms/workspace.json").exists());

    let restored = restore_workspace_backup(request(true, false)).unwrap();
    assert_eq!(restored.workspace_id, workspace.workspace_id);
    assert_eq!(restored.manifest_digest, backup.manifest_digest);
    let reopened = Workspace::open(&replacement_edit).unwrap();
    assert_eq!(reopened.workspace_id, workspace.workspace_id);
    assert_eq!(
        reopened.document(document.id).unwrap().relative_path,
        document.relative_path
    );
    assert_eq!(
        fs::read_to_string(replacement_edit.join("Policies/Access.md")).unwrap(),
        "# Access\n"
    );

    assert!(matches!(
        restore_workspace_backup(request(true, false)),
        Err(DmsError::RestorePathExists(_))
    ));
    assert!(!replacement_edit.join(".dms/lock").exists());
    restore_workspace_backup(request(true, true)).unwrap();
    assert!(!replacement_edit.join(".dms/lock").exists());
}

#[test]
fn restore_refuses_fresh_locks_and_unsafe_archive_paths() {
    let (_edit_root, _publish_root, workspace) = initialized_workspace();
    let backup_directory = tempfile::tempdir().unwrap();
    let archive = backup_directory.path().join("workspace-lock.zip");
    workspace.backup_workspace(&archive).unwrap();
    let destination = tempfile::tempdir().unwrap();
    let replacement_edit = destination.path().join("edit");
    let replacement_publish = destination.path().join("publish");
    fs::create_dir(&replacement_edit).unwrap();
    fs::create_dir(&replacement_publish).unwrap();
    fs::create_dir(replacement_edit.join(".dms")).unwrap();
    fs::write(
        replacement_edit.join(".dms/lock"),
        serde_json::to_vec_pretty(&WorkspaceLock::current()).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        restore_workspace_backup(RestoreRequest {
            archive_path: &archive,
            edit_root: &replacement_edit,
            publish_root: &replacement_publish,
            replace_existing: true,
            take_over_stale_lock: false,
            confirmed: true,
        }),
        Err(DmsError::WorkspaceLocked)
    ));

    let unsafe_archive = destination.path().join("unsafe.zip");
    write_unsafe_archive(&unsafe_archive, workspace.workspace_id, "edit/../escape");
    let clean_edit = destination.path().join("clean-edit");
    let clean_publish = destination.path().join("clean-publish");
    fs::create_dir(&clean_edit).unwrap();
    fs::create_dir(&clean_publish).unwrap();
    assert!(matches!(
        restore_workspace_backup(RestoreRequest {
            archive_path: &unsafe_archive,
            edit_root: &clean_edit,
            publish_root: &clean_publish,
            replace_existing: false,
            take_over_stale_lock: false,
            confirmed: true,
        }),
        Err(DmsError::RestoreArchive(_))
    ));
    assert!(!destination.path().join("escape").exists());

    let lock_alias_archive = destination.path().join("lock-alias.zip");
    write_unsafe_archive(
        &lock_alias_archive,
        workspace.workspace_id,
        "edit/.dms/LOCK",
    );
    assert!(matches!(
        restore_workspace_backup(RestoreRequest {
            archive_path: &lock_alias_archive,
            edit_root: &clean_edit,
            publish_root: &clean_publish,
            replace_existing: false,
            take_over_stale_lock: false,
            confirmed: true,
        }),
        Err(DmsError::RestoreArchive(_))
    ));
}

#[test]
fn restore_uses_the_destination_workspace_lock_threshold() {
    let (_source_edit, _source_publish, mut source) = initialized_workspace();
    source.configure_lock_staleness(1).unwrap();
    let backup_directory = tempfile::tempdir().unwrap();
    let archive = backup_directory
        .path()
        .join("workspace-target-threshold.zip");
    source.backup_workspace(&archive).unwrap();

    let (destination_edit, destination_publish, mut destination) = initialized_workspace();
    destination.configure_lock_staleness(24).unwrap();
    let mut lock = WorkspaceLock::current();
    lock.acquired_at = Utc::now() - Duration::hours(2);
    fs::write(
        destination_edit.path().join(".dms/lock"),
        serde_json::to_vec_pretty(&lock).unwrap(),
    )
    .unwrap();

    assert!(matches!(
        restore_workspace_backup(RestoreRequest {
            archive_path: &archive,
            edit_root: destination_edit.path(),
            publish_root: destination_publish.path(),
            replace_existing: true,
            take_over_stale_lock: true,
            confirmed: true,
        }),
        Err(DmsError::WorkspaceLocked)
    ));
}

#[cfg(unix)]
#[test]
fn restore_refuses_symlink_targets() {
    use std::os::unix::fs::symlink;

    let (_edit_root, _publish_root, workspace) = initialized_workspace();
    let backup_directory = tempfile::tempdir().unwrap();
    let archive = backup_directory.path().join("workspace-symlink.zip");
    workspace.backup_workspace(&archive).unwrap();
    let destination = tempfile::tempdir().unwrap();
    let replacement_edit = destination.path().join("edit");
    let replacement_publish = destination.path().join("publish");
    fs::create_dir(&replacement_edit).unwrap();
    fs::create_dir(&replacement_publish).unwrap();
    let outside = destination.path().join("outside");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, replacement_edit.join(".dms")).unwrap();

    assert!(matches!(
        restore_workspace_backup(RestoreRequest {
            archive_path: &archive,
            edit_root: &replacement_edit,
            publish_root: &replacement_publish,
            replace_existing: true,
            take_over_stale_lock: false,
            confirmed: true,
        }),
        Err(DmsError::RestoreTargetInvalid(_))
    ));
    assert!(fs::read_dir(&outside).unwrap().next().is_none());
}

fn write_unsafe_archive(path: &Path, workspace_id: uuid::Uuid, archive_path: &str) {
    let bytes = b"escape";
    let manifest = BackupManifest {
        workspace_id,
        created_at: Utc::now(),
        entries: vec![BackupManifestEntry {
            archive_path: archive_path.to_owned(),
            size: bytes.len() as u64,
            sha256: format!("{:x}", Sha256::digest(bytes)),
        }],
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    let file = fs::File::create(path).unwrap();
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default();
    archive.start_file(archive_path, options).unwrap();
    archive.write_all(bytes).unwrap();
    archive.start_file("manifest.json", options).unwrap();
    archive.write_all(&manifest_bytes).unwrap();
    archive.finish().unwrap();
}
