use std::path::{Path, PathBuf};
#[cfg(any(windows, target_os = "macos"))]
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaudeDesktopApp {
    launch_target: PathBuf,
}

impl ClaudeDesktopApp {
    pub fn locate() -> Option<Self> {
        locate_from_candidates(platform_candidates())
    }

    pub fn launch(&self) -> Result<(), String> {
        launch_target(&self.launch_target)
    }
}

fn locate_from_candidates(candidates: Vec<PathBuf>) -> Option<ClaudeDesktopApp> {
    candidates
        .into_iter()
        .find(|candidate| candidate.exists())
        .map(|launch_target| ClaudeDesktopApp { launch_target })
}

#[cfg(windows)]
fn platform_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let root = PathBuf::from(local_app_data);
        candidates.push(root.join("AnthropicClaude/claude.exe"));
        candidates.push(root.join("Programs/Claude/Claude.exe"));
    }
    candidates
}

#[cfg(target_os = "macos")]
fn platform_candidates() -> Vec<PathBuf> {
    let mut candidates = vec![PathBuf::from("/Applications/Claude.app")];
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(PathBuf::from(home).join("Applications/Claude.app"));
    }
    candidates
}

#[cfg(not(any(windows, target_os = "macos")))]
fn platform_candidates() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(windows)]
fn launch_target(target: &Path) -> Result<(), String> {
    Command::new(target).spawn().map(|_| ()).map_err(|error| {
        format!(
            "cannot launch Claude Desktop at {}: {error}",
            target.display()
        )
    })
}

#[cfg(target_os = "macos")]
fn launch_target(target: &Path) -> Result<(), String> {
    Command::new("open")
        .arg(target)
        .spawn()
        .map(|_| ())
        .map_err(|error| {
            format!(
                "cannot launch Claude Desktop at {}: {error}",
                target.display()
            )
        })
}

#[cfg(not(any(windows, target_os = "macos")))]
fn launch_target(_target: &Path) -> Result<(), String> {
    Err("Claude Desktop handoff is supported only on Windows and macOS".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_desktop_app_is_non_fatal_and_existing_candidate_is_selected() {
        let directory = tempfile::tempdir().unwrap();
        assert!(locate_from_candidates(vec![directory.path().join("missing")]).is_none());
        let existing = directory.path().join("Claude.app");
        std::fs::create_dir(&existing).unwrap();
        assert_eq!(
            locate_from_candidates(vec![directory.path().join("missing"), existing.clone()])
                .unwrap()
                .launch_target,
            existing
        );
    }
}
