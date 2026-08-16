use std::{collections::BTreeMap, fs, path::Path};

use uuid::Uuid;

use super::{
    DmsError, MarkerStatus, MarkerVerdict, Result, SourceState, TargetSelection, Workspace,
};

/// Controlled export-chrome tokens. Frontmatter may validate these fields but
/// must not fill the matching Word placeholders; release chrome owns them.
pub const RESERVED_MARKDOWN_TEMPLATE_VARIABLES: &[&str] =
    &["TITLE", "DOCUMENT_NUMBER", "VERSION", "CONFIDENTIALITY"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownFrontmatter {
    pub title: Option<String>,
    pub document_number: Option<String>,
    pub version: String,
    pub confidentiality: String,
    /// Every flat scalar frontmatter key → value, original key spelling.
    pub variables: BTreeMap<String, String>,
}

impl MarkdownFrontmatter {
    /// Non-reserved template variables as `{TOKEN}` → value.
    /// Keys must be ASCII identifiers; tokens are uppercased.
    pub fn template_variables(&self) -> BTreeMap<String, String> {
        let mut variables = BTreeMap::new();
        for (key, value) in &self.variables {
            if !is_template_variable_key(key) {
                continue;
            }
            let token = key.to_ascii_uppercase();
            if RESERVED_MARKDOWN_TEMPLATE_VARIABLES
                .iter()
                .any(|reserved| *reserved == token)
            {
                continue;
            }
            variables.insert(format!("{{{token}}}"), value.clone());
        }
        variables
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownFrontmatterCheck {
    pub title: Option<MarkerVerdict>,
    pub document_number: Option<MarkerVerdict>,
    pub version: MarkerVerdict,
    pub confidentiality: MarkerVerdict,
}

impl MarkdownFrontmatterCheck {
    pub fn passes(&self) -> bool {
        self.version.status == MarkerStatus::Match
            && self.confidentiality.status == MarkerStatus::Match
            && self
                .title
                .as_ref()
                .is_none_or(|verdict| verdict.status == MarkerStatus::Match)
            && self
                .document_number
                .as_ref()
                .is_none_or(|verdict| verdict.status == MarkerStatus::Match)
    }
}

pub fn parse_markdown_frontmatter(markdown: &str) -> Result<(MarkdownFrontmatter, &str)> {
    let markdown = markdown.strip_prefix('\u{feff}').unwrap_or(markdown);
    let (opening, rest) = split_line(markdown).ok_or_else(|| {
        DmsError::InvalidMarkdownFrontmatter("missing opening delimiter".to_owned())
    })?;
    if opening.trim_end_matches('\r') != "---" {
        return Err(DmsError::InvalidMarkdownFrontmatter(
            "frontmatter must start with --- on the first line".to_owned(),
        ));
    }

    let mut title = None;
    let mut document_number = None;
    let mut version = None;
    let mut confidentiality = None;
    let mut variables = BTreeMap::new();
    let mut body_offset = 0;
    let mut closed = false;

    for line in rest.split_inclusive('\n') {
        body_offset += line.len();
        let line = line.trim_end_matches(['\r', '\n']);
        if matches!(line, "---" | "...") {
            closed = true;
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with(char::is_whitespace) {
            return Err(DmsError::InvalidMarkdownFrontmatter(
                "nested frontmatter values are not supported".to_owned(),
            ));
        }
        let (key, raw_value) = line.split_once(':').ok_or_else(|| {
            DmsError::InvalidMarkdownFrontmatter(format!(
                "frontmatter line {line:?} is not a key/value scalar"
            ))
        })?;
        let key = key.trim();
        if key.is_empty() || key.contains(char::is_whitespace) {
            return Err(DmsError::InvalidMarkdownFrontmatter(format!(
                "frontmatter key {key:?} is invalid"
            )));
        }
        let value = parse_scalar(key, raw_value)?;
        if variables.insert(key.to_owned(), value.clone()).is_some() {
            return Err(DmsError::InvalidMarkdownFrontmatter(format!(
                "frontmatter key {key} is duplicated"
            )));
        }
        match key {
            "title" => set_once(&mut title, key, value)?,
            "document_number" => set_once(&mut document_number, key, value)?,
            "version" => set_once(&mut version, key, value)?,
            "confidentiality" => set_once(&mut confidentiality, key, value)?,
            _ => {}
        }
    }

    if !closed {
        return Err(DmsError::InvalidMarkdownFrontmatter(
            "frontmatter closing delimiter is missing".to_owned(),
        ));
    }
    let version = version.ok_or_else(|| {
        DmsError::InvalidMarkdownFrontmatter(
            "required frontmatter key version is missing".to_owned(),
        )
    })?;
    let confidentiality = confidentiality.ok_or_else(|| {
        DmsError::InvalidMarkdownFrontmatter(
            "required frontmatter key confidentiality is missing".to_owned(),
        )
    })?;

    Ok((
        MarkdownFrontmatter {
            title,
            document_number,
            version,
            confidentiality,
            variables,
        },
        &rest[body_offset..],
    ))
}

fn is_template_variable_key(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(character) if character.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

pub fn check_markdown_frontmatter(
    markdown: &str,
    expected_title: &str,
    expected_document_number: Option<&str>,
    expected_version: &str,
    expected_confidentiality: &str,
) -> Result<MarkdownFrontmatterCheck> {
    let (frontmatter, _) = parse_markdown_frontmatter(markdown)?;
    Ok(MarkdownFrontmatterCheck {
        title: frontmatter
            .title
            .as_deref()
            .map(|detected| scalar_verdict(expected_title, detected, "frontmatter:title")),
        document_number: frontmatter.document_number.as_deref().map(|detected| {
            scalar_verdict(
                expected_document_number.unwrap_or_default(),
                detected,
                "frontmatter:document_number",
            )
        }),
        version: scalar_verdict(
            expected_version,
            &frontmatter.version,
            "frontmatter:version",
        ),
        confidentiality: scalar_verdict(
            expected_confidentiality,
            &frontmatter.confidentiality,
            "frontmatter:confidentiality",
        ),
    })
}

/// Controlled frontmatter values owned by DMS for registered Markdown members.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlledMarkdownFields {
    pub title: String,
    pub document_number: Option<String>,
    pub version: String,
    pub confidentiality: String,
}

const CONTROLLED_FRONTMATTER_KEYS: &[&str] =
    &["title", "document_number", "version", "confidentiality"];

/// Rewrite controlled frontmatter keys from DMS values while preserving the body
/// and any non-controlled flat scalars (template variables).
pub fn apply_controlled_frontmatter(
    markdown: &str,
    controlled: &ControlledMarkdownFields,
) -> Result<String> {
    let (mut extras, body) = split_frontmatter_map(markdown)?;
    for key in CONTROLLED_FRONTMATTER_KEYS {
        extras.remove(*key);
    }

    let mut output = String::from("---\n");
    push_frontmatter_line(&mut output, "title", &controlled.title);
    if let Some(number) = controlled.document_number.as_deref() {
        push_frontmatter_line(&mut output, "document_number", number);
    }
    push_frontmatter_line(&mut output, "version", &controlled.version);
    push_frontmatter_line(&mut output, "confidentiality", &controlled.confidentiality);
    for (key, value) in extras {
        push_frontmatter_line(&mut output, &key, &value);
    }
    output.push_str("---\n");
    output.push_str(body);
    Ok(output)
}

/// Split optional frontmatter into a key map and body. Missing controlled keys
/// are allowed so DMS can prefill new or incomplete files.
pub fn split_frontmatter_map(markdown: &str) -> Result<(BTreeMap<String, String>, &str)> {
    let markdown = markdown.strip_prefix('\u{feff}').unwrap_or(markdown);
    let Some((opening, rest)) = split_line(markdown) else {
        return Ok((BTreeMap::new(), markdown));
    };
    if opening.trim_end_matches('\r') != "---" {
        return Ok((BTreeMap::new(), markdown));
    }

    let mut variables = BTreeMap::new();
    let mut body_offset = 0;
    let mut closed = false;
    for line in rest.split_inclusive('\n') {
        body_offset += line.len();
        let line = line.trim_end_matches(['\r', '\n']);
        if matches!(line, "---" | "...") {
            closed = true;
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with(char::is_whitespace) {
            return Err(DmsError::InvalidMarkdownFrontmatter(
                "nested frontmatter values are not supported".to_owned(),
            ));
        }
        let (key, raw_value) = line.split_once(':').ok_or_else(|| {
            DmsError::InvalidMarkdownFrontmatter(format!(
                "frontmatter line {line:?} is not a key/value scalar"
            ))
        })?;
        let key = key.trim();
        if key.is_empty() || key.contains(char::is_whitespace) {
            return Err(DmsError::InvalidMarkdownFrontmatter(format!(
                "frontmatter key {key:?} is invalid"
            )));
        }
        let value = parse_scalar(key, raw_value)?;
        if variables.insert(key.to_owned(), value).is_some() {
            return Err(DmsError::InvalidMarkdownFrontmatter(format!(
                "frontmatter key {key} is duplicated"
            )));
        }
    }
    if !closed {
        return Err(DmsError::InvalidMarkdownFrontmatter(
            "frontmatter closing delimiter is missing".to_owned(),
        ));
    }
    Ok((variables, &rest[body_offset..]))
}

fn push_frontmatter_line(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push_str(": ");
    output.push_str(&yaml_scalar(value));
    output.push('\n');
}

fn yaml_scalar(value: &str) -> String {
    let needs_quotes = value.is_empty()
        || value != value.trim()
        || value.bytes().any(|byte| {
            matches!(
                byte,
                b':' | b'#'
                    | b'"'
                    | b'\''
                    | b'['
                    | b'{'
                    | b']'
                    | b'}'
                    | b','
                    | b'&'
                    | b'*'
                    | b'!'
                    | b'|'
                    | b'>'
                    | b'<'
                    | b'%'
                    | b'@'
                    | b'`'
                    | b'\n'
                    | b'\r'
            )
        });
    if needs_quotes {
        format!(
            "\"{}\"",
            value
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
        )
    } else {
        value.to_owned()
    }
}

fn is_markdown_source_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

impl Workspace {
    /// Prefill or overwrite controlled Markdown frontmatter from DMS authority.
    pub fn sync_markdown_control_frontmatter(&self, document_id: Uuid) -> Result<()> {
        self.sync_markdown_control_frontmatter_with_version(document_id, None)
    }

    pub(crate) fn sync_markdown_control_frontmatter_with_version(
        &self,
        document_id: Uuid,
        version_override: Option<&str>,
    ) -> Result<()> {
        let document = self.document(document_id)?;
        if document.source_state != SourceState::Registered {
            return Ok(());
        }
        if !is_markdown_source_path(&document.relative_path) {
            return Ok(());
        }
        let path = self.edit_root.join(&document.relative_path);
        let source = fs::read_to_string(&path).map_err(|source| DmsError::Io {
            path: path.clone(),
            source,
        })?;
        let confidentiality = match self.effective_confidentiality(document_id) {
            Ok(value) => value.type_id,
            Err(DmsError::MissingConfidentialityPolicy) => return Ok(()),
            Err(error) => return Err(error),
        };
        let version = match version_override {
            Some(version) => version.to_owned(),
            None => self.markdown_control_version_label(document_id)?,
        };
        let controlled = ControlledMarkdownFields {
            title: document.control.title.clone(),
            document_number: document.control.document_number.clone(),
            version,
            confidentiality,
        };
        let next = apply_controlled_frontmatter(&source, &controlled)?;
        if next != source {
            fs::write(&path, next.as_bytes()).map_err(|source| DmsError::Io {
                path: path.clone(),
                source,
            })?;
        }
        Ok(())
    }

    pub(crate) fn sync_all_registered_markdown_frontmatter(&self) -> Result<()> {
        let document_ids = self
            .documents
            .values()
            .filter(|document| {
                document.source_state == SourceState::Registered
                    && is_markdown_source_path(&document.relative_path)
            })
            .map(|document| document.id)
            .collect::<Vec<_>>();
        for document_id in document_ids {
            self.sync_markdown_control_frontmatter(document_id)?;
        }
        Ok(())
    }

    fn markdown_control_version_label(&self, document_id: Uuid) -> Result<String> {
        let document = self.document(document_id)?;
        if let Some(candidate_id) = document.active_candidate_id {
            if let Some(candidate) = document
                .candidates
                .iter()
                .find(|candidate| candidate.id == candidate_id)
            {
                return Ok(format!(
                    "{}.{}",
                    candidate.version.major, candidate.version.minor
                ));
            }
        }
        let version = self.resolve_target_version(document_id, TargetSelection::NextMinor)?;
        Ok(format!("{}.{}", version.major, version.minor))
    }
}

fn split_line(value: &str) -> Option<(&str, &str)> {
    value
        .find('\n')
        .map(|index| (&value[..index], &value[index + 1..]))
}

fn set_once(target: &mut Option<String>, key: &str, value: String) -> Result<()> {
    if target.replace(value).is_some() {
        return Err(DmsError::InvalidMarkdownFrontmatter(format!(
            "frontmatter key {key} is duplicated"
        )));
    }
    Ok(())
}

fn parse_scalar(key: &str, raw_value: &str) -> Result<String> {
    let value = raw_value.trim();
    if value.is_empty()
        || matches!(
            value.as_bytes().first(),
            Some(b'[' | b'{' | b'|' | b'>' | b'&' | b'*')
        )
    {
        return Err(DmsError::InvalidMarkdownFrontmatter(format!(
            "frontmatter key {key} must contain one scalar value"
        )));
    }
    let value = match (value.as_bytes().first(), value.as_bytes().last()) {
        (Some(b'"'), Some(b'"')) | (Some(b'\''), Some(b'\'')) if value.len() >= 2 => {
            &value[1..value.len() - 1]
        }
        (Some(b'"'), _) | (_, Some(b'"')) | (Some(b'\''), _) | (_, Some(b'\'')) => {
            return Err(DmsError::InvalidMarkdownFrontmatter(format!(
                "frontmatter key {key} has an unterminated quoted scalar"
            )))
        }
        _ => value,
    };
    if value.is_empty() {
        return Err(DmsError::InvalidMarkdownFrontmatter(format!(
            "frontmatter key {key} must not be empty"
        )));
    }
    Ok(value.to_owned())
}

fn scalar_verdict(expected: &str, detected: &str, location: &str) -> MarkerVerdict {
    MarkerVerdict {
        status: if expected.trim() == detected.trim() {
            MarkerStatus::Match
        } else {
            MarkerStatus::Mismatch
        },
        expected: expected.trim().to_owned(),
        detected: vec![detected.trim().to_owned()],
        locations: vec![location.to_owned()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_flat_fields_and_returns_body() {
        let source = "---\ntitle: Policy\ndocument_number: P-01\nversion: 1.0\nconfidentiality: Internal\nowner: ignored\nauthor: Ada\n---\n# Body\n";
        let (frontmatter, body) = parse_markdown_frontmatter(source).unwrap();
        assert_eq!(frontmatter.title.as_deref(), Some("Policy"));
        assert_eq!(frontmatter.document_number.as_deref(), Some("P-01"));
        assert_eq!(frontmatter.version, "1.0");
        assert_eq!(frontmatter.confidentiality, "Internal");
        assert_eq!(
            frontmatter.variables.get("owner").map(String::as_str),
            Some("ignored")
        );
        assert_eq!(
            frontmatter.variables.get("author").map(String::as_str),
            Some("Ada")
        );
        assert_eq!(
            frontmatter
                .template_variables()
                .get("{AUTHOR}")
                .map(String::as_str),
            Some("Ada")
        );
        assert!(!frontmatter.template_variables().contains_key("{TITLE}"));
        assert!(!frontmatter.template_variables().contains_key("{VERSION}"));
        assert_eq!(body, "# Body\n");
    }

    #[test]
    fn rejects_missing_duplicate_nested_and_malformed_frontmatter() {
        for source in [
            "# Body\n",
            "---\nversion: 1.0\n---\n# Body\n",
            "---\nversion: 1.0\nversion: 2.0\nconfidentiality: Internal\n---\n",
            "---\nversion:\n  major: 1\nconfidentiality: Internal\n---\n",
            "---\nversion: 1.0\nconfidentiality: Internal\n",
        ] {
            assert!(matches!(
                parse_markdown_frontmatter(source),
                Err(DmsError::InvalidMarkdownFrontmatter(_))
            ));
        }
    }

    #[test]
    fn optional_fields_are_compared_only_when_present() {
        let source = "---\nversion: 1.0\nconfidentiality: Internal\n---\nBody\n";
        let check = check_markdown_frontmatter(source, "Policy", None, "1.0", "Internal").unwrap();
        assert!(check.passes());
        assert!(check.title.is_none());
        assert!(check.document_number.is_none());

        let source = "---\ntitle: Wrong\ndocument_number: P-02\nversion: 1.0\nconfidentiality: Internal\n---\nBody\n";
        let check =
            check_markdown_frontmatter(source, "Policy", Some("P-01"), "1.0", "Internal").unwrap();
        assert!(!check.passes());
        assert_eq!(check.title.unwrap().detected, ["Wrong"]);
        assert_eq!(check.document_number.unwrap().expected, "P-01");
    }

    #[test]
    fn apply_controlled_frontmatter_prefills_and_preserves_extras() {
        let controlled = ControlledMarkdownFields {
            title: "Policy <A>".to_owned(),
            document_number: Some("P-1".to_owned()),
            version: "1.0".to_owned(),
            confidentiality: "Vertraulich & intern".to_owned(),
        };
        let written = apply_controlled_frontmatter("# Body\n", &controlled).unwrap();
        assert!(written.starts_with("---\n"));
        assert!(written.contains("title: \"Policy <A>\"") || written.contains("title: Policy"));
        let (parsed, body) = parse_markdown_frontmatter(&written).unwrap();
        assert_eq!(parsed.title.as_deref(), Some("Policy <A>"));
        assert_eq!(parsed.document_number.as_deref(), Some("P-1"));
        assert_eq!(parsed.version, "1.0");
        assert_eq!(parsed.confidentiality, "Vertraulich & intern");
        assert_eq!(body, "# Body\n");

        let merged = apply_controlled_frontmatter(
            "---\ntitle: Old\nversion: 0.1\nconfidentiality: Internal\nauthor: Ada\n---\n# Keep\n",
            &ControlledMarkdownFields {
                title: "New".to_owned(),
                document_number: None,
                version: "2.0".to_owned(),
                confidentiality: "Internal".to_owned(),
            },
        )
        .unwrap();
        let (parsed, body) = parse_markdown_frontmatter(&merged).unwrap();
        assert_eq!(parsed.title.as_deref(), Some("New"));
        assert!(parsed.document_number.is_none());
        assert_eq!(parsed.version, "2.0");
        assert_eq!(
            parsed.variables.get("author").map(String::as_str),
            Some("Ada")
        );
        assert_eq!(body, "# Keep\n");
        assert!(!merged.contains("document_number:"));
    }
}
