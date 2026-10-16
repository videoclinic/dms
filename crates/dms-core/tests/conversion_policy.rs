use std::fs;

use dms_core::{
    apply_controlled_frontmatter, parse_markdown_frontmatter, ControlledMarkdownFields,
    ConversionPolicy, DmsError, TodoProjection, Workspace, SCHEMA_VERSION,
};
use serde_json::{json, Value};

fn source(extra: &str) -> String {
    format!("---\ntitle: Policy\nversion: 1.0\nconfidentiality: internal\n{extra}---\nTODO: source body stays\n")
}

#[test]
fn todo_policy_defaults_and_precedence_are_explicit() {
    assert_eq!(
        ConversionPolicy::default().todo_projection,
        TodoProjection::Disabled
    );
    for library in [TodoProjection::Disabled, TodoProjection::Enabled] {
        let policy = ConversionPolicy {
            todo_projection: library,
        };
        for (extra, expected) in [
            ("", library),
            ("todo_projection: enabled\n", TodoProjection::Enabled),
            ("todo_projection: disabled\n", TodoProjection::Disabled),
            ("todo_projection: 'enabled'\n", TodoProjection::Enabled),
            ("todo_projection: \"disabled\"\n", TodoProjection::Disabled),
            ("TODO_PROJECTION: enabled\n", library),
            ("last_modified: \"\"\n", library),
        ] {
            let markdown = source(extra);
            let before = markdown.clone();
            assert_eq!(policy.resolve_todo_projection(&markdown).unwrap(), expected);
            assert_eq!(markdown, before);
            assert_eq!(policy.resolve_todo_projection(&markdown).unwrap(), expected);
        }
    }
}

#[test]
fn todo_policy_rejects_noncanonical_duplicate_and_nested_overrides_without_source_leaks() {
    for library in [TodoProjection::Disabled, TodoProjection::Enabled] {
        let policy = ConversionPolicy {
            todo_projection: library,
        };
        for extra in [
            "todo_projection: true\n",
            "todo_projection: false\n",
            "todo_projection: ENABLED\n",
            "todo_projection: Disabled\n",
            "todo_projection: 1\n",
            "todo_projection: null\n",
            "todo_projection: \n",
            "todo_projection: ''\n",
            "todo_projection: \" enabled \"\n",
            "todo_projection: enabled # comment\n",
            "todo_projection: [enabled]\n",
            "todo_projection: {value: enabled}\n",
            "todo_projection: |\n  enabled\n",
            "todo_projection:\n  enabled: true\n",
            "todo_projection: enabled\ntodo_projection: disabled\n",
            "todo_projection: \"enabled\n",
            "todo_projection: PRIVATE_SOURCE_VALUE\n",
            "PRIVATE_SOURCE_LINE\n",
        ] {
            let error = policy
                .resolve_todo_projection(&source(extra))
                .unwrap_err()
                .to_string();
            assert!(
                matches!(
                    error.as_str(),
                    "todo_projection must be enabled or disabled"
                        | "invalid or missing flat conversion frontmatter"
                ),
                "{error}"
            );
            assert!(!error.contains("PRIVATE_SOURCE"));
        }
    }
}

#[test]
fn todo_policy_persists_only_the_workspace_setting_and_requires_explicit_save() {
    let temp = tempfile::tempdir().unwrap();
    let edit = temp.path().join("edit");
    fs::create_dir(&edit).unwrap();
    let mut workspace = Workspace::init(&edit, &temp.path().join("publish")).unwrap();
    let path = edit.join(".dms/workspace.json");
    let source_path = edit.join("policy.md");
    fs::write(&source_path, source("todo_projection: enabled\n")).unwrap();
    let document = workspace.add_document(&source_path).unwrap();
    workspace.save().unwrap();
    let before_source = fs::read(&source_path).unwrap();
    let before: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        before["conversion_policy"],
        json!({"todo_projection": "disabled"})
    );
    for setting in [TodoProjection::Enabled, TodoProjection::Disabled] {
        let before_save = fs::read(&path).unwrap();
        let policy = ConversionPolicy {
            todo_projection: setting,
        };
        workspace.set_conversion_policy(policy);
        assert_eq!(fs::read(&path).unwrap(), before_save);
        workspace.save().unwrap();
        let reopened = Workspace::open(&edit).unwrap();
        assert_eq!(reopened.conversion_policy(), policy);
        let mut actual = serde_json::to_value(&reopened).unwrap();
        actual["conversion_policy"] = before["conversion_policy"].clone();
        assert_eq!(actual, before);
        assert_eq!(fs::read(&source_path).unwrap(), before_source);
        assert!(
            !serde_json::to_string(reopened.document(document.id).unwrap())
                .unwrap()
                .contains("todo_projection")
        );
        assert_eq!(fs::read_dir(&reopened.publish_root).unwrap().count(), 0);
    }
}

#[test]
fn todo_policy_schema_21_fixture_migrates_disabled_with_exact_backup() {
    // Build the previous schema fixture from an ordinary persisted workspace,
    // retaining its canonical roots, document and instruction fingerprint.
    let temp = tempfile::tempdir().unwrap();
    let edit = temp.path().join("edit");
    fs::create_dir(&edit).unwrap();
    let mut workspace = Workspace::init(&edit, &temp.path().join("publish")).unwrap();
    fs::write(edit.join("policy.md"), source("todo_projection: enabled\n")).unwrap();
    workspace.add_document(&edit.join("policy.md")).unwrap();
    workspace.save().unwrap();
    let path = edit.join(".dms/workspace.json");
    let mut fixture = serde_json::to_value(&workspace).unwrap();
    fixture["schema_version"] = json!(21);
    fixture.as_object_mut().unwrap().remove("conversion_policy");
    let bytes = serde_json::to_vec_pretty(&fixture).unwrap();
    fs::write(&path, &bytes).unwrap();
    let source_before = fs::read(edit.join("policy.md")).unwrap();
    let instructions_before = fs::read(edit.join(".dms/AGENTS.md")).unwrap();
    let migrated = Workspace::open(&edit).unwrap();
    assert_eq!(migrated.schema_version, SCHEMA_VERSION);
    assert_eq!(migrated.conversion_policy(), ConversionPolicy::default());
    assert_eq!(
        fs::read(edit.join(".dms/workspace.v21.json.bak")).unwrap(),
        bytes
    );
    let mut actual = serde_json::to_value(&migrated).unwrap();
    actual["schema_version"] = json!(21);
    actual.as_object_mut().unwrap().remove("conversion_policy");
    assert_eq!(actual, fixture);
    assert_eq!(fs::read(edit.join("policy.md")).unwrap(), source_before);
    assert_eq!(
        fs::read(edit.join(".dms/AGENTS.md")).unwrap(),
        instructions_before
    );
    let saved = fs::read(&path).unwrap();
    Workspace::open(&edit).unwrap();
    assert_eq!(fs::read(&path).unwrap(), saved);
}

#[test]
fn todo_policy_invalid_persisted_shapes_fail_without_writes_or_migration_backups() {
    for schema in [21, SCHEMA_VERSION] {
        for policy in [
            json!(null),
            json!(true),
            json!({}),
            json!({"todo_projection": true}),
            json!({"todo_projection": "ENABLED"}),
            json!({"todo_projection": "invalid"}),
            json!({"todo_projection": "disabled", "unknown": true}),
        ] {
            let temp = tempfile::tempdir().unwrap();
            let edit = temp.path().join("edit");
            fs::create_dir(&edit).unwrap();
            let workspace = Workspace::init(&edit, &temp.path().join("publish")).unwrap();
            let mut value = serde_json::to_value(&workspace).unwrap();
            value["schema_version"] = json!(schema);
            value["conversion_policy"] = policy;
            let bytes = serde_json::to_vec(&value).unwrap();
            let path = edit.join(".dms/workspace.json");
            fs::write(&path, &bytes).unwrap();
            assert!(matches!(
                Workspace::open(&edit),
                Err(DmsError::InvalidMetadata { .. })
            ));
            assert_eq!(fs::read(&path).unwrap(), bytes);
            assert!(!edit.join(".dms/workspace.v21.json.bak").exists());
        }
    }
}

#[test]
fn todo_override_remains_source_owned_during_controlled_frontmatter_sync() {
    for value in ["enabled", "disabled", "not-a-conversion-policy"] {
        let markdown = source(&format!("todo_projection: {value}\n"));
        let rewritten = apply_controlled_frontmatter(
            &markdown,
            &ControlledMarkdownFields {
                title: "Changed title".into(),
                document_number: None,
                version: "2.0".into(),
                confidentiality: "internal".into(),
            },
        )
        .unwrap();
        let (fields, body) = parse_markdown_frontmatter(&rewritten).unwrap();
        assert_eq!(fields.variables["todo_projection"], value);
        assert_eq!(body, "TODO: source body stays\n");
    }
}
