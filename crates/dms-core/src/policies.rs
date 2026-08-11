use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    configured_text, validate_portable_id, DmsError, Result, Workspace, METADATA_DIRECTORY,
};

pub(crate) const ROOT_FOLDER: &str = ".";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConfidentialityType {
    pub id: String,
    pub label: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConfidentialityPolicy {
    pub folder: String,
    pub type_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffectiveConfidentiality {
    pub type_id: String,
    pub label: String,
    pub source_folder: String,
    pub document_override: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyFolder {
    pub relative_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EntraIdentitySource {
    pub binding_id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_display: String,
    pub group_id: Uuid,
    pub group_label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EntraPerson {
    pub object_id: Uuid,
    pub display_name: String,
    pub email: String,
    pub account_enabled: bool,
}

impl EntraPerson {
    pub fn eligible(object_id: Uuid, display_name: &str, email: &str) -> Self {
        Self {
            object_id,
            display_name: display_name.trim().to_owned(),
            email: email.trim().to_owned(),
            account_enabled: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRoleRef {
    pub binding_id: Uuid,
    pub object_id: Uuid,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowPolicy {
    pub editor: Option<WorkflowRoleRef>,
    pub approver: Option<WorkflowRoleRef>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowPolicyAssignment {
    pub folder: String,
    pub editor: Option<WorkflowRoleRef>,
    pub approver: Option<WorkflowRoleRef>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DocumentWorkflowOverrides {
    pub editor: Option<WorkflowRoleRef>,
    pub approver: Option<WorkflowRoleRef>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoleUpdate {
    Unchanged,
    Clear,
    Replace(Uuid),
}

impl RoleUpdate {
    pub fn replace(object_id: Uuid) -> Self {
        Self::Replace(object_id)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionState {
    Resolved,
    Unresolved,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffectiveWorkflowRole {
    pub object_id: Uuid,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub source_folder: String,
    pub document_override: bool,
    pub state: ResolutionState,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffectiveWorkflowRoles {
    pub editor: Option<EffectiveWorkflowRole>,
    pub approver: Option<EffectiveWorkflowRole>,
}

impl Workspace {
    pub fn confidentiality_types(&self) -> Vec<&ConfidentialityType> {
        self.confidentiality_types.values().collect()
    }

    pub fn confidentiality_policies(&self) -> Vec<&ConfidentialityPolicy> {
        self.confidentiality_policies.values().collect()
    }

    pub fn configure_confidentiality_type(
        &mut self,
        id: &str,
        label: &str,
        enabled: bool,
    ) -> Result<ConfidentialityType> {
        let id = id.trim();
        validate_portable_id(id, DmsError::InvalidConfidentialityTypeId)?;
        if !enabled
            && (self
                .confidentiality_policies
                .values()
                .any(|policy| policy.type_id == id)
                || self
                    .documents
                    .values()
                    .any(|document| document.confidentiality_override.as_deref() == Some(id)))
        {
            return Err(DmsError::ConfidentialityTypeInUse(id.to_owned()));
        }
        let configured = ConfidentialityType {
            id: id.to_owned(),
            label: configured_text(label, "confidentiality label")?,
            enabled,
        };
        self.confidentiality_types
            .insert(configured.id.clone(), configured.clone());
        Ok(configured)
    }

    pub fn set_confidentiality_policy(
        &mut self,
        folder: &str,
        type_id: &str,
    ) -> Result<ConfidentialityPolicy> {
        self.require_enabled_confidentiality_type(type_id)?;
        let folder = self.resolve_policy_folder(folder)?;
        let policy = ConfidentialityPolicy {
            folder: folder.clone(),
            type_id: type_id.to_owned(),
        };
        self.confidentiality_policies.insert(folder, policy.clone());
        self.invalidate_stale_candidates();
        Ok(policy)
    }

    pub fn remove_confidentiality_policy(&mut self, folder: &str) -> Result<()> {
        let folder = self.resolve_policy_folder(folder)?;
        if folder == ROOT_FOLDER {
            return Err(DmsError::RequiredRootPolicy);
        }
        self.confidentiality_policies.remove(&folder);
        self.invalidate_stale_candidates();
        Ok(())
    }

    pub fn set_document_confidentiality(
        &mut self,
        document_id: Uuid,
        type_id: Option<&str>,
    ) -> Result<()> {
        if let Some(type_id) = type_id {
            self.require_enabled_confidentiality_type(type_id)?;
        }
        let document = self
            .documents
            .get_mut(&document_id)
            .ok_or(DmsError::DocumentNotFound(document_id))?;
        document.confidentiality_override = type_id.map(str::to_owned);
        self.invalidate_stale_candidates();
        Ok(())
    }

    pub fn document_confidentiality_override(&self, document_id: Uuid) -> Result<Option<&str>> {
        Ok(self
            .document(document_id)?
            .confidentiality_override
            .as_deref())
    }

    pub fn effective_confidentiality(&self, document_id: Uuid) -> Result<EffectiveConfidentiality> {
        let document = self.document(document_id)?;
        if let Some(type_id) = document.confidentiality_override.as_deref() {
            let configured = self.require_confidentiality_type(type_id)?;
            return Ok(EffectiveConfidentiality {
                type_id: configured.id.clone(),
                label: configured.label.clone(),
                source_folder: document_parent(&document.relative_path),
                document_override: true,
            });
        }
        let document_folder = document_parent(&document.relative_path);
        let policy = ancestor_folders(&document_folder)
            .find_map(|folder| self.confidentiality_policies.get(&folder))
            .ok_or(DmsError::MissingConfidentialityPolicy)?;
        let configured = self.require_confidentiality_type(&policy.type_id)?;
        Ok(EffectiveConfidentiality {
            type_id: configured.id.clone(),
            label: configured.label.clone(),
            source_folder: policy.folder.clone(),
            document_override: false,
        })
    }

    pub fn policy_folders(&self) -> Result<Vec<PolicyFolder>> {
        let mut folders = vec![PolicyFolder {
            relative_path: ROOT_FOLDER.to_owned(),
        }];
        collect_policy_folders(&self.edit_root, &self.edit_root, &mut folders)?;
        folders.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        Ok(folders)
    }

    pub fn identity_source(&self) -> Option<&EntraIdentitySource> {
        self.identity_source.as_ref()
    }

    pub fn eligible_people(&self) -> Vec<&EntraPerson> {
        self.identity_cache
            .values()
            .filter(|person| person.account_enabled)
            .collect()
    }

    pub fn workflow_policies(&self) -> Vec<WorkflowPolicyAssignment> {
        self.workflow_policies
            .iter()
            .map(|(folder, policy)| WorkflowPolicyAssignment {
                folder: folder.clone(),
                editor: policy.editor,
                approver: policy.approver,
            })
            .collect()
    }

    pub fn replace_identity_source(
        &mut self,
        tenant_id: Uuid,
        tenant_display: &str,
        group_id: Uuid,
        group_label: &str,
        people: Vec<EntraPerson>,
    ) -> Result<EntraIdentitySource> {
        let source = EntraIdentitySource {
            binding_id: Uuid::new_v4(),
            tenant_id,
            tenant_display: configured_text(tenant_display, "tenant display")?,
            group_id,
            group_label: configured_text(group_label, "group label")?,
        };
        let mut cache = BTreeMap::new();
        for mut person in people {
            person.display_name = configured_text(&person.display_name, "person display name")?;
            person.email = configured_text(&person.email, "person email")?;
            cache.insert(person.object_id, person);
        }
        self.identity_source = Some(source.clone());
        self.identity_cache = cache;
        self.invalidate_stale_candidates();
        Ok(source)
    }

    pub fn update_workflow_policy(
        &mut self,
        folder: &str,
        editor: RoleUpdate,
        approver: RoleUpdate,
    ) -> Result<WorkflowPolicy> {
        let folder = self.resolve_policy_folder(folder)?;
        let mut policy = self
            .workflow_policies
            .get(&folder)
            .copied()
            .unwrap_or_default();
        policy.editor = self.apply_role_update(policy.editor, editor)?;
        policy.approver = self.apply_role_update(policy.approver, approver)?;
        if folder == ROOT_FOLDER && (policy.editor.is_none() || policy.approver.is_none()) {
            return Err(DmsError::RequiredRootWorkflowPolicy);
        }
        if policy.editor.is_none() && policy.approver.is_none() {
            self.workflow_policies.remove(&folder);
        } else {
            self.workflow_policies.insert(folder, policy);
        }
        self.invalidate_stale_candidates();
        Ok(policy)
    }

    pub fn set_document_workflow_roles(
        &mut self,
        document_id: Uuid,
        editor: RoleUpdate,
        approver: RoleUpdate,
    ) -> Result<DocumentWorkflowOverrides> {
        let current = self
            .documents
            .get(&document_id)
            .ok_or(DmsError::DocumentNotFound(document_id))?
            .workflow_overrides;
        let updated = DocumentWorkflowOverrides {
            editor: self.apply_role_update(current.editor, editor)?,
            approver: self.apply_role_update(current.approver, approver)?,
        };
        self.documents
            .get_mut(&document_id)
            .expect("document checked above")
            .workflow_overrides = updated;
        self.invalidate_stale_candidates();
        Ok(updated)
    }

    pub fn effective_workflow_roles(&self, document_id: Uuid) -> Result<EffectiveWorkflowRoles> {
        let document = self.document(document_id)?;
        let folder = document_parent(&document.relative_path);
        let editor = self.effective_role(folder.as_str(), document.workflow_overrides.editor, true);
        let approver =
            self.effective_role(folder.as_str(), document.workflow_overrides.approver, false);
        Ok(EffectiveWorkflowRoles { editor, approver })
    }

    pub(crate) fn validate_policies(&self) -> Result<()> {
        if !self.confidentiality_types.is_empty()
            && !self.confidentiality_policies.contains_key(ROOT_FOLDER)
        {
            return Err(DmsError::RequiredRootPolicy);
        }
        for policy in self.confidentiality_policies.values() {
            self.require_enabled_confidentiality_type(&policy.type_id)?;
        }
        for document in self.documents.values() {
            if let Some(type_id) = document.confidentiality_override.as_deref() {
                self.require_enabled_confidentiality_type(type_id)?;
            }
        }
        if self.identity_source.is_some() {
            let root = self
                .workflow_policies
                .get(ROOT_FOLDER)
                .ok_or(DmsError::RequiredRootWorkflowPolicy)?;
            if root.editor.is_none() || root.approver.is_none() {
                return Err(DmsError::RequiredRootWorkflowPolicy);
            }
        }
        for (object_id, person) in &self.identity_cache {
            if object_id != &person.object_id {
                return Err(DmsError::IdentityCacheKeyMismatch {
                    key: *object_id,
                    stored: person.object_id,
                });
            }
        }
        Ok(())
    }

    fn resolve_policy_folder(&self, folder: &str) -> Result<String> {
        if folder == ROOT_FOLDER {
            return Ok(ROOT_FOLDER.to_owned());
        }
        if folder.is_empty() || folder.contains('\\') {
            return Err(DmsError::InvalidPolicyFolder(folder.to_owned()));
        }
        let relative = Path::new(folder);
        if relative.is_absolute()
            || relative.components().any(|component| {
                matches!(
                    component,
                    Component::CurDir
                        | Component::ParentDir
                        | Component::RootDir
                        | Component::Prefix(_)
                )
            })
            || relative
                .components()
                .next()
                .is_some_and(|component| component.as_os_str() == METADATA_DIRECTORY)
        {
            return Err(DmsError::InvalidPolicyFolder(folder.to_owned()));
        }
        let absolute =
            fs::canonicalize(self.edit_root.join(relative)).map_err(|source| DmsError::Io {
                path: self.edit_root.join(relative),
                source,
            })?;
        if !absolute.is_dir() || !absolute.starts_with(&self.edit_root) {
            return Err(DmsError::InvalidPolicyFolder(folder.to_owned()));
        }
        relative_folder_string(
            absolute
                .strip_prefix(&self.edit_root)
                .map_err(|_| DmsError::InvalidPolicyFolder(folder.to_owned()))?,
        )
    }

    fn require_confidentiality_type(&self, type_id: &str) -> Result<&ConfidentialityType> {
        self.confidentiality_types
            .get(type_id)
            .ok_or_else(|| DmsError::UnknownConfidentialityType(type_id.to_owned()))
    }

    fn require_enabled_confidentiality_type(&self, type_id: &str) -> Result<&ConfidentialityType> {
        let configured = self.require_confidentiality_type(type_id)?;
        if !configured.enabled {
            return Err(DmsError::DisabledConfidentialityType(type_id.to_owned()));
        }
        Ok(configured)
    }

    fn apply_role_update(
        &self,
        current: Option<WorkflowRoleRef>,
        update: RoleUpdate,
    ) -> Result<Option<WorkflowRoleRef>> {
        match update {
            RoleUpdate::Unchanged => Ok(current),
            RoleUpdate::Clear => Ok(None),
            RoleUpdate::Replace(object_id) => {
                let source = self
                    .identity_source
                    .as_ref()
                    .ok_or(DmsError::IdentitySourceRequired)?;
                let person = self
                    .identity_cache
                    .get(&object_id)
                    .filter(|person| person.account_enabled)
                    .ok_or(DmsError::IneligibleEntraPerson(object_id))?;
                Ok(Some(WorkflowRoleRef {
                    binding_id: source.binding_id,
                    object_id: person.object_id,
                }))
            }
        }
    }

    fn effective_role(
        &self,
        document_folder: &str,
        document_override: Option<WorkflowRoleRef>,
        editor: bool,
    ) -> Option<EffectiveWorkflowRole> {
        if let Some(role) = document_override {
            return Some(self.resolve_role(role, document_folder, true));
        }
        ancestor_folders(document_folder).find_map(|folder| {
            self.workflow_policies.get(&folder).and_then(|policy| {
                let role = if editor {
                    policy.editor
                } else {
                    policy.approver
                }?;
                Some(self.resolve_role(role, &folder, false))
            })
        })
    }

    fn resolve_role(
        &self,
        role: WorkflowRoleRef,
        source_folder: &str,
        document_override: bool,
    ) -> EffectiveWorkflowRole {
        let source_matches = self
            .identity_source
            .as_ref()
            .is_some_and(|source| source.binding_id == role.binding_id);
        let person = self.identity_cache.get(&role.object_id);
        let resolved = source_matches && person.is_some_and(|person| person.account_enabled);
        EffectiveWorkflowRole {
            object_id: role.object_id,
            display_name: person.map(|person| person.display_name.clone()),
            email: person.map(|person| person.email.clone()),
            source_folder: source_folder.to_owned(),
            document_override,
            state: if resolved {
                ResolutionState::Resolved
            } else {
                ResolutionState::Unresolved
            },
        }
    }
}

fn collect_policy_folders(
    edit_root: &Path,
    current: &Path,
    folders: &mut Vec<PolicyFolder>,
) -> Result<()> {
    let mut entries = fs::read_dir(current)
        .map_err(|source| DmsError::Io {
            path: current.to_path_buf(),
            source,
        })?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|source| DmsError::Io {
            path: current.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if !path.is_dir() || entry.file_name() == METADATA_DIRECTORY {
            continue;
        }
        let relative = path
            .strip_prefix(edit_root)
            .map_err(|_| DmsError::InvalidPolicyFolder(path.display().to_string()))?;
        folders.push(PolicyFolder {
            relative_path: relative_folder_string(relative)?,
        });
        collect_policy_folders(edit_root, &path, folders)?;
    }
    Ok(())
}

fn relative_folder_string(path: &Path) -> Result<String> {
    let components = path
        .components()
        .map(|component| match component {
            Component::Normal(value) => value
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| DmsError::InvalidPolicyFolder(path.display().to_string())),
            _ => Err(DmsError::InvalidPolicyFolder(path.display().to_string())),
        })
        .collect::<Result<Vec<_>>>()?;
    if components.is_empty() {
        Ok(ROOT_FOLDER.to_owned())
    } else {
        Ok(components.join("/"))
    }
}

fn document_parent(path: &Path) -> String {
    path.parent()
        .and_then(|parent| relative_folder_string(parent).ok())
        .unwrap_or_else(|| ROOT_FOLDER.to_owned())
}

fn ancestor_folders(folder: &str) -> impl Iterator<Item = String> {
    let mut folders = Vec::new();
    let mut current = folder.to_owned();
    loop {
        folders.push(current.clone());
        if current == ROOT_FOLDER {
            break;
        }
        current = current
            .rsplit_once('/')
            .map(|(parent, _)| parent.to_owned())
            .unwrap_or_else(|| ROOT_FOLDER.to_owned());
    }
    folders.into_iter()
}
