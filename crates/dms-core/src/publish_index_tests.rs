use super::*;
use crate::{
    ConfidentialitySnapshot, DocumentControl, PersonSnapshot, ReleaseRecord, TargetVersionMode,
};
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    workspace: Workspace,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let edit = temp.path().join("edit");
        fs::create_dir(&edit).unwrap();
        let workspace = Workspace::init(&edit, &temp.path().join("publish")).unwrap();
        Self {
            _temp: temp,
            workspace,
        }
    }

    fn document(&mut self, name: &str) -> Uuid {
        let source = self.workspace.edit_root.join(name);
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "# Source draft, not catalog metadata\n").unwrap();
        self.workspace.add_document(&source).unwrap().id
    }

    fn release(&mut self, id: Uuid, path: &str, title: &str, minor: u32, date: &str) -> Uuid {
        let person = PersonSnapshot {
            tenant_id: Uuid::nil(),
            object_id: Uuid::nil(),
            display_name: "Not catalog data".into(),
            email: "".into(),
        };
        let release = ReleaseRecord {
            id: Uuid::new_v4(),
            version: Version { major: 1, minor },
            relative_pdf_path: path.into(),
            source_digest: "unverified source digest".into(),
            pdf_digest: "deliberately not the PDF checksum".into(),
            changelog: "Not catalog data".into(),
            assistance: None,
            mode: TargetVersionMode::NextMinor,
            approval_required: false,
            approval_chain_head: None,
            workflow_chain_head: String::new(),
            confidentiality: ConfidentialitySnapshot {
                type_id: "internal".into(),
                label: "Internal".into(),
            },
            owner: None,
            editor: person.clone(),
            approver: person.clone(),
            requester: person,
            effective_date: None,
            control: Some(DocumentControl {
                title: title.into(),
                document_number: Some("DOC-42".into()),
                document_type: None,
                owner: None,
                legacy_owner_label: None,
            }),
            released_at: date.parse().unwrap(),
            withdrawn: false,
        };
        let release_id = release.id;
        let absolute = self.workspace.publish_root.join(path);
        fs::create_dir_all(absolute.parent().unwrap()).unwrap();
        fs::write(absolute, b"synthetic PDF bytes; not read by the index").unwrap();
        self.workspace
            .document_mut(id)
            .unwrap()
            .releases
            .push(release);
        release_id
    }

    fn index_path(&self) -> PathBuf {
        self.workspace.publish_root.join(PUBLISH_INDEX_FILENAME)
    }
}

const EARLY: &str = "2026-01-01T00:00:00Z";
const LATE: &str = "2026-02-01T00:00:00Z";

fn filenames(root: &Path) -> Vec<String> {
    let mut names: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

#[test]
fn nested_mirrored_tree_and_encoded_relative_links() {
    let mut f = Fixture::new();
    let a = f.document("Policies/People/draft.md");
    f.release(
        a,
        "Policies/People/Handbook #1%_V1.0_internal.pdf",
        "People handbook",
        0,
        EARLY,
    );
    let b = f.document("root.md");
    f.release(b, "Root_V1.0_internal.pdf", "Root document", 0, LATE);
    let html = f.workspace.publish_index().unwrap().render_html();
    assert!(html.contains("<li><button type=\"button\" data-folder=\"Policies\" aria-pressed=\"false\">Policies</button><ul><li><button type=\"button\" data-folder=\"Policies/People\""));
    assert!(html.contains("href=\"./Policies/People/Handbook%20%231%25_V1.0_internal.pdf\""));
    assert!(html.contains("href=\"./Root_V1.0_internal.pdf\""));
    assert!(!html.contains("draft.md"));
    assert!(html.contains("aria-label=\"Release folder tree\""));
    assert!(html.contains("scope=\"col\""));
}

#[test]
fn current_selection_reuses_version_authority_and_withdrawal_fallback() {
    let mut f = Fixture::new();
    let id = f.document("draft.md");
    f.release(id, "old.pdf", "Historic title", 0, LATE);
    let current = f.release(id, "current.pdf", "Captured current title", 1, EARLY);
    let withdrawn = f.release(id, "withdrawn.pdf", "Withdrawn title", 2, LATE);
    let document = f.workspace.document_mut(id).unwrap();
    document.releases.last_mut().unwrap().withdrawn = true;
    document.releases.reverse();
    document.control.title = "Mutable draft title".into();
    document.control.document_number = Some("DRAFT-NUMBER".into());
    let index = f.workspace.publish_index().unwrap();
    assert_eq!(index.rows().len(), 1);
    assert_eq!(index.rows()[0].release_id, current);
    assert_ne!(index.rows()[0].release_id, withdrawn);
    assert_eq!(index.rows()[0].document_number.as_deref(), Some("DOC-42"));
    let html = index.render_html();
    for excluded in [
        "Historic title",
        "Withdrawn title",
        "Mutable draft title",
        "DRAFT-NUMBER",
        "old.pdf",
        "withdrawn.pdf",
    ] {
        assert!(!html.contains(excluded), "{excluded}");
    }
    assert!(html.contains("Captured current title"));
    assert!(html.contains("V1.1"));
}

#[test]
fn excludes_missing_current_without_resurrecting_history_drafts_and_orphans() {
    let mut f = Fixture::new();
    let missing = f.document("missing-current.md");
    f.release(missing, "old.pdf", "Must not resurrect", 0, EARLY);
    f.release(missing, "missing.pdf", "Missing current", 1, LATE);
    fs::remove_file(f.workspace.publish_root.join("missing.pdf")).unwrap();
    let withdrawn = f.document("withdrawn.md");
    f.release(withdrawn, "withdrawn.pdf", "Only withdrawn", 0, EARLY);
    f.workspace.document_mut(withdrawn).unwrap().releases[0].withdrawn = true;
    let orphan = f.document("orphan.md");
    f.release(orphan, "orphan.pdf", "Orphaned release", 0, EARLY);
    f.workspace.document_mut(orphan).unwrap().source_state = SourceState::Unregistered;
    f.document("never-released.md");
    fs::write(
        f.workspace.publish_root.join("manual-drop.pdf"),
        b"not a release",
    )
    .unwrap();
    let index = f.workspace.publish_index().unwrap();
    assert!(index.rows().is_empty());
    let html = index.render_html();
    for excluded in [
        "old.pdf",
        "missing.pdf",
        "withdrawn.pdf",
        "orphan.pdf",
        "never-released",
        "manual-drop",
    ] {
        assert!(!html.contains(excluded));
    }
    assert!(html.contains("<p id=\"empty\">No released documents"));
    assert!(html.contains("0 documents"));
}

#[test]
fn later_release_replaces_row_and_legacy_snapshot_is_not_invented() {
    let mut f = Fixture::new();
    let id = f.document("secret-source-name.md");
    f.release(id, "first.pdf", "First title", 0, EARLY);
    assert_eq!(
        f.workspace.publish_index().unwrap().rows()[0].title,
        "First title"
    );
    let second = f.release(id, "second.pdf", "Second title", 1, LATE);
    f.workspace.document_mut(id).unwrap().releases[1].control = None;
    let index = f.workspace.publish_index().unwrap();
    assert_eq!(index.rows()[0].release_id, second);
    assert_eq!(index.rows()[0].title, LEGACY_TITLE);
    assert_eq!(index.rows()[0].document_number, None);
    let html = index.render_html();
    assert!(!html.contains("secret-source-name"));
    assert!(!html.contains("First title"));
    assert!(!html.contains("Second title"));
    assert!(html.contains("Not recorded"));
}

#[test]
fn metadata_and_path_punctuation_are_text_not_markup_or_script() {
    let mut f = Fixture::new();
    let id = f.document("draft.md");
    f.release(
        id,
        "R&D/Ü ' #%.pdf",
        "</script><img src=x onerror='bad()'> & \"",
        0,
        EARLY,
    );
    let release = &mut f.workspace.document_mut(id).unwrap().releases[0];
    release.control.as_mut().unwrap().document_number = Some("\"><svg/onload=bad()>".into());
    release.confidentiality.type_id = "\" autofocus onfocus='bad()' <&>".into();
    let html = f.workspace.publish_index().unwrap().render_html();
    assert!(html.contains("&lt;/script&gt;&lt;img src=x onerror=&#39;bad()&#39;&gt; &amp; &quot;"));
    assert!(html.contains("&quot;&gt;&lt;svg/onload=bad()&gt;"));
    assert!(html.contains("data-type=\"&quot; autofocus onfocus=&#39;bad()&#39; &lt;&amp;&gt;\""));
    assert!(html.contains("href=\"./R%26D/%C3%9C%20%27%20%23%25.pdf\""));
    assert!(html.contains(">R&amp;D</button>"));
    assert_eq!(html.matches("<script>").count(), 1);
    assert_eq!(html.matches("</script>").count(), 1);
    assert!(!html.contains("<img"));
    assert!(!html.contains("<svg"));
    let script = html
        .split("<script>\n")
        .nth(1)
        .unwrap()
        .split("</script>")
        .next()
        .unwrap();
    assert_eq!(script, include_str!("publish_index.js"));
    assert!(!script.contains("bad()"));
    assert!(!script.contains("innerHTML"));
}

#[cfg(unix)]
#[test]
fn markup_characters_in_paths_are_encoded_and_escaped() {
    let mut f = Fixture::new();
    let id = f.document("draft.md");
    f.release(id, "<folder>/'\"&.pdf", "Path test", 0, EARLY);
    let html = f.workspace.publish_index().unwrap().render_html();
    assert!(html.contains("href=\"./%3Cfolder%3E/%27%22%26.pdf\""));
    assert!(html.contains("data-folder=\"&lt;folder&gt;\""));
    assert!(!html.contains("<folder>"));
}

#[test]
fn newest_first_source_and_name_path_ties_are_deterministic() {
    let mut f = Fixture::new();
    for (source, path, title, date) in [
        ("one.md", "z.pdf", "Alpha", EARLY),
        ("two.md", "a.pdf", "Alpha", EARLY),
        ("three.md", "b.pdf", "beta", EARLY),
        ("four.md", "new.pdf", "Zulu", LATE),
        (
            "five.md",
            "nano.pdf",
            "Nanosecond",
            "2026-02-01T00:00:00.000000001Z",
        ),
    ] {
        let id = f.document(source);
        f.release(id, path, title, 0, date);
    }
    let index = f.workspace.publish_index().unwrap();
    assert_eq!(
        index
            .rows()
            .iter()
            .map(|r| r.relative_pdf_path.as_str())
            .collect::<Vec<_>>(),
        ["nano.pdf", "new.pdf", "a.pdf", "z.pdf", "b.pdf"]
    );
    let html = index.render_html();
    let positions: Vec<_> = ["nano.pdf", "new.pdf", "a.pdf", "z.pdf", "b.pdf"]
        .iter()
        .map(|p| html.find(&format!("href=\"./{p}\"")).unwrap())
        .collect();
    assert!(positions.windows(2).all(|p| p[0] < p[1]));
    assert_eq!(html, index.render_html());
    assert_eq!(html, f.workspace.publish_index().unwrap().render_html());
    assert!(html.contains("data-date-order=\"0\" data-name-order=\"3\""));
}

#[test]
fn standalone_controls_type_ids_and_ownership_are_present() {
    let mut f = Fixture::new();
    for (name, type_id) in [
        ("one", "internal"),
        ("two", "public"),
        ("three", "internal"),
    ] {
        let id = f.document(&format!("{name}.md"));
        f.release(id, &format!("{name}.pdf"), name, 0, EARLY);
        f.workspace.document_mut(id).unwrap().releases[0]
            .confidentiality
            .type_id = type_id.into();
    }
    let html = f.workspace.publish_index().unwrap().render_html();
    assert!(html.starts_with(INDEX_PREFIX));
    assert_eq!(html.matches(PUBLISH_INDEX_OWNERSHIP_MARKER).count(), 1);
    assert_eq!(
        html.matches("type=\"checkbox\" data-type=\"internal\"")
            .count(),
        1
    );
    assert_eq!(
        html.matches("type=\"checkbox\" data-type=\"public\"")
            .count(),
        1
    );
    for control in [
        "id=\"search\"",
        "id=\"all-types\"",
        "value=\"date-desc\"",
        "value=\"date-asc\"",
        "value=\"name-asc\"",
        "value=\"name-desc\"",
        "id=\"result-count\"",
        "id=\"empty\"",
        "<style>",
        "<script>",
    ] {
        assert!(html.contains(control), "{control}");
    }
    for remote in [
        "http:",
        "https:",
        "file:",
        "src=",
        "<link",
        "fetch(",
        "XMLHttpRequest",
        "@import",
        "url(",
    ] {
        assert!(!html.contains(remote), "{remote}");
    }
}

#[test]
fn refresh_changes_only_index_without_reading_sources_or_verifying_pdf() {
    let mut f = Fixture::new();
    let id = f.document("draft.md");
    f.release(id, "release.pdf", "Captured title", 0, EARLY);
    fs::remove_file(f.workspace.edit_root.join("draft.md")).unwrap();
    let before = serde_json::to_vec(&f.workspace).unwrap();
    let metadata = f.workspace.edit_root.join(".dms/workspace.json");
    let metadata_before = fs::read(&metadata).unwrap();
    let pdf = f.workspace.publish_root.join("release.pdf");
    let pdf_before = fs::read(&pdf).unwrap();
    let result = f.workspace.refresh_publish_index().unwrap();
    assert_eq!(result.status, PublishIndexStatus::Written);
    assert_eq!(result.document_count, 1);
    assert_eq!(result.path, f.index_path());
    assert_eq!(serde_json::to_vec(&f.workspace).unwrap(), before);
    assert_eq!(fs::read(metadata).unwrap(), metadata_before);
    assert_eq!(fs::read(pdf).unwrap(), pdf_before);
    assert_eq!(
        filenames(&f.workspace.publish_root),
        ["index.html", "release.pdf"]
    );
}

#[cfg(unix)]
#[test]
fn unreadable_pdf_is_indexed_without_opening_its_bytes() {
    use std::os::unix::fs::PermissionsExt;
    let mut f = Fixture::new();
    let id = f.document("draft.md");
    f.release(id, "release.pdf", "Present but unreadable", 0, EARLY);
    let path = f.workspace.publish_root.join("release.pdf");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    assert_eq!(
        f.workspace.refresh_publish_index().unwrap().document_count,
        1
    );
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
}

#[test]
fn unchanged_skips_sibling_write_and_preserves_file_identity() {
    let f = Fixture::new();
    f.workspace.refresh_publish_index().unwrap();
    let path = f.index_path();
    let before = fs::metadata(&path).unwrap();
    let html = fs::read(&path).unwrap();
    assert_eq!(
        write_index(&path, &html, |_, _| panic!("unchanged must never write")).unwrap(),
        PublishIndexStatus::Unchanged
    );
    assert_eq!(
        f.workspace.refresh_publish_index().unwrap().status,
        PublishIndexStatus::Unchanged
    );
    let after = fs::metadata(&path).unwrap();
    assert_eq!(before.modified().unwrap(), after.modified().unwrap());
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        assert_eq!(before.ino(), after.ino());
    }
    assert_eq!(filenames(&f.workspace.publish_root), ["index.html"]);
}

#[test]
fn changed_index_replaces_old_file_atomically_with_no_sibling_left() {
    let mut f = Fixture::new();
    f.workspace.refresh_publish_index().unwrap();
    let path = f.index_path();
    let before = fs::read(&path).unwrap();
    let id = f.document("new.md");
    f.release(id, "new.pdf", "New release", 0, LATE);
    let html = f.workspace.publish_index().unwrap().render_html();
    let result = write_index(&path, html.as_bytes(), |file, bytes| {
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(
            filenames(&f.workspace.publish_root)
                .iter()
                .filter(|n| n.starts_with(".dms-index-"))
                .count(),
            1
        );
        file.write_all(bytes)?;
        file.flush()?;
        file.sync_all()?;
        assert_eq!(fs::read(&path).unwrap(), before);
        Ok(())
    })
    .unwrap();
    assert_eq!(result, PublishIndexStatus::Written);
    assert_eq!(fs::read(&path).unwrap(), html.as_bytes());
    assert_eq!(
        filenames(&f.workspace.publish_root),
        ["index.html", "new.pdf"]
    );
    assert_eq!(
        f.workspace.refresh_publish_index().unwrap().status,
        PublishIndexStatus::Unchanged
    );
}

#[test]
fn foreign_index_and_marker_imitation_are_refused_byte_identically() {
    let f = Fixture::new();
    for foreign in [
        b"operator's index".to_vec(),
        format!("<html>{PUBLISH_INDEX_OWNERSHIP_MARKER}</html>").into_bytes(),
        b"<!doctype html>\n<!-- dms-owned publish index -->\n".to_vec(),
        vec![0xff, 0xfe],
    ] {
        fs::write(f.index_path(), &foreign).unwrap();
        assert!(
            matches!(f.workspace.refresh_publish_index(), Err(DmsError::ForeignPublishIndex(p)) if p == f.index_path())
        );
        assert_eq!(fs::read(f.index_path()).unwrap(), foreign);
        assert_eq!(filenames(&f.workspace.publish_root), ["index.html"]);
    }
}

#[test]
fn output_directory_is_not_an_owned_index() {
    let f = Fixture::new();
    fs::create_dir(f.index_path()).unwrap();
    assert!(matches!(
        f.workspace.refresh_publish_index(),
        Err(DmsError::ForeignPublishIndex(_))
    ));
    assert!(f.index_path().is_dir());
    assert_eq!(filenames(&f.workspace.publish_root), ["index.html"]);
}

#[test]
fn partial_write_and_flush_failure_preserve_prior_index_and_cleanup_sibling() {
    let f = Fixture::new();
    f.workspace.refresh_publish_index().unwrap();
    let path = f.index_path();
    let before = fs::read(&path).unwrap();
    for failure in [io::ErrorKind::WriteZero, io::ErrorKind::Other] {
        let error = write_index(&path, b"changed bytes", |file, bytes| {
            file.write_all(&bytes[..3])?;
            Err(io::Error::new(failure, "injected write/flush failure"))
        })
        .unwrap_err();
        assert!(matches!(error, DmsError::Io { path: p, .. } if p == path));
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(filenames(&f.workspace.publish_root), ["index.html"]);
    }
}

#[cfg(unix)]
#[test]
fn actual_rename_failure_preserves_prior_index_and_cleans_up() {
    let f = Fixture::new();
    f.workspace.refresh_publish_index().unwrap();
    let path = f.index_path();
    let before = fs::read(&path).unwrap();
    let result = write_index(&path, b"changed bytes", |file, bytes| {
        file.write_all(bytes)?;
        file.sync_all()?;
        let sibling = fs::read_dir(&f.workspace.publish_root)?
            .map(|e| e.unwrap().path())
            .find(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".dms-index-")
            })
            .unwrap();
        fs::remove_file(sibling)?;
        Ok(())
    });
    assert!(matches!(result, Err(DmsError::Io { .. })));
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(filenames(&f.workspace.publish_root), ["index.html"]);
}

#[test]
fn foreign_file_appearing_during_write_is_not_overwritten() {
    let f = Fixture::new();
    let result = write_index(&f.index_path(), INDEX_PREFIX.as_bytes(), |file, bytes| {
        file.write_all(bytes)?;
        fs::write(f.index_path(), b"foreign arrival")
    });
    assert!(matches!(result, Err(DmsError::ForeignPublishIndex(_))));
    assert_eq!(fs::read(f.index_path()).unwrap(), b"foreign arrival");
    assert_eq!(filenames(&f.workspace.publish_root), ["index.html"]);
}

#[test]
fn changed_owned_file_during_write_is_not_overwritten() {
    let f = Fixture::new();
    f.workspace.refresh_publish_index().unwrap();
    let concurrent = format!("{INDEX_PREFIX}another writer");
    let result = write_index(&f.index_path(), b"replacement", |file, bytes| {
        file.write_all(bytes)?;
        fs::write(f.index_path(), &concurrent)
    });
    assert!(matches!(result, Err(DmsError::PublishIndexChanged(_))));
    assert_eq!(fs::read(f.index_path()).unwrap(), concurrent.as_bytes());
    assert_eq!(filenames(&f.workspace.publish_root), ["index.html"]);
}

#[test]
fn unsafe_recorded_paths_fail_before_replacing_owned_index() {
    let mut f = Fixture::new();
    f.workspace.refresh_publish_index().unwrap();
    let before = fs::read(f.index_path()).unwrap();
    let id = f.document("draft.md");
    f.release(id, "valid.pdf", "Title", 0, EARLY);
    let bad_paths = [
        "",
        "/outside.pdf",
        "../outside.pdf",
        "a/../outside.pdf",
        "./valid.pdf",
        "a/./valid.pdf",
        "a//valid.pdf",
        "a/valid.pdf/",
        "C:/outside.pdf",
        "C:outside.pdf",
        "//server/share/file.pdf",
        "a.pdf:stream",
        "a\0.pdf",
        "a\n.pdf",
        "folder./a.pdf",
        "folder /a.pdf",
        "NUL.pdf",
        "CON .pdf",
        "prn  ..pdf",
        "COM1 .pdf",
        "LPT³ .folder/a.pdf",
        "COM1/a.pdf",
        "file.html",
        "a/..",
        "a/.",
        #[cfg(unix)]
        "a\\valid.pdf",
        "\\\\server\\share\\file.pdf",
    ];
    for bad in bad_paths {
        f.workspace.document_mut(id).unwrap().releases[0].relative_pdf_path = bad.into();
        assert!(
            matches!(
                f.workspace.refresh_publish_index(),
                Err(DmsError::InvalidReleasePath(_))
            ),
            "{bad:?}"
        );
        assert_eq!(fs::read(f.index_path()).unwrap(), before);
    }
    assert_eq!(
        filenames(&f.workspace.publish_root),
        ["index.html", "valid.pdf"]
    );
}

#[test]
fn reserved_device_aliases_before_extensions_are_refused() {
    for stem in [
        "CON", "prn", "AuX", "nul", "COM1", "com9", "COM¹", "COM²", "COM³", "LPT1", "lpt9", "LPT¹",
        "LPT²", "LPT³",
    ] {
        for suffix in [" .pdf", "  .pdf", "..pdf", " . .pdf", "  ...pdf"] {
            for path in [
                format!("{stem}{suffix}"),
                format!("{stem}{suffix}/file.pdf"),
            ] {
                assert!(
                    matches!(
                        normalized_release_path(Path::new(&path)),
                        Err(DmsError::InvalidReleasePath(_))
                    ),
                    "{path:?}"
                );
            }
        }
    }
}

#[test]
fn non_device_names_with_spaces_and_dots_are_preserved() {
    for path in [
        "Report .pdf",
        "Report  ...pdf",
        "CON report.pdf",
        "CONSOLE .pdf",
        "COM0 .pdf",
        "COM10 .pdf",
        "LPT0 .pdf",
        "LPT10 .pdf",
        "CON\u{00a0}.pdf",
        "LPT1\u{2003}.pdf",
        "Report .folder/file.pdf",
    ] {
        assert_eq!(normalized_release_path(Path::new(path)).unwrap(), path);
    }
}

#[test]
fn directory_instead_of_pdf_is_invalid_not_discovered() {
    let mut f = Fixture::new();
    let id = f.document("draft.md");
    f.release(id, "valid.pdf", "Title", 0, EARLY);
    fs::remove_file(f.workspace.publish_root.join("valid.pdf")).unwrap();
    fs::create_dir(f.workspace.publish_root.join("valid.pdf")).unwrap();
    assert!(matches!(
        f.workspace.publish_index(),
        Err(DmsError::InvalidReleasePath(_))
    ));
}

#[cfg(unix)]
#[test]
fn non_utf8_release_path_is_refused() {
    use std::os::unix::ffi::OsStringExt;
    let path = PathBuf::from(std::ffi::OsString::from_vec(b"bad\xff.pdf".to_vec()));
    assert!(matches!(
        normalized_release_path(&path),
        Err(DmsError::InvalidReleasePath(_))
    ));
}

#[cfg(unix)]
#[test]
fn release_and_output_symlinks_cannot_escape_or_claim_ownership() {
    use std::os::unix::fs::symlink;
    let mut f = Fixture::new();
    let id = f.document("draft.md");
    f.release(id, "valid.pdf", "Title", 0, EARLY);
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("outside.pdf"), b"outside bytes").unwrap();
    symlink(outside.path(), f.workspace.publish_root.join("escape")).unwrap();
    symlink(
        outside.path().join("outside.pdf"),
        f.workspace.publish_root.join("link.pdf"),
    )
    .unwrap();
    symlink(
        f.workspace.publish_root.join("valid.pdf"),
        f.workspace.publish_root.join("internal.pdf"),
    )
    .unwrap();
    symlink(
        outside.path().join("absent.pdf"),
        f.workspace.publish_root.join("dangling.pdf"),
    )
    .unwrap();
    for path in [
        "escape/outside.pdf",
        "escape/missing.pdf",
        "link.pdf",
        "internal.pdf",
        "dangling.pdf",
    ] {
        f.workspace.document_mut(id).unwrap().releases[0].relative_pdf_path = path.into();
        assert!(
            matches!(
                f.workspace.publish_index(),
                Err(DmsError::InvalidReleasePath(_))
            ),
            "{path}"
        );
    }
    f.workspace.document_mut(id).unwrap().releases[0].relative_pdf_path = "valid.pdf".into();
    let owned = format!("{INDEX_PREFIX}outside owned content");
    let target = outside.path().join("owned.html");
    fs::write(&target, &owned).unwrap();
    symlink(&target, f.index_path()).unwrap();
    assert!(matches!(
        f.workspace.refresh_publish_index(),
        Err(DmsError::ForeignPublishIndex(_))
    ));
    assert_eq!(fs::read(&target).unwrap(), owned.as_bytes());
    fs::remove_file(f.index_path()).unwrap();
    symlink(outside.path().join("absent.html"), f.index_path()).unwrap();
    assert!(matches!(
        f.workspace.refresh_publish_index(),
        Err(DmsError::ForeignPublishIndex(_))
    ));
    assert!(!outside.path().join("absent.html").exists());
}
