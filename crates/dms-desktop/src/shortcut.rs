use std::path::Path;

#[cfg(target_os = "windows")]
use std::path::PathBuf;

use dms_core::{Workspace, DMS_WORKSPACE_SHORTCUT_FILENAME};

#[cfg(target_os = "windows")]
const SHORTCUT_DESCRIPTION: &str = "Open DMS workspace";

pub fn ensure_workspace_shortcut(workspace: &Workspace) -> Result<(), String> {
    let path = workspace.edit_root.join(DMS_WORKSPACE_SHORTCUT_FILENAME);
    let uri = workspace.workspace_permalink();

    platform::write_shortcut(&path, &uri).map_err(|error| {
        format!(
            "cannot write DMS workspace shortcut {}: {error}",
            path.display()
        )
    })
}

#[cfg(target_os = "windows")]
pub(crate) mod platform {
    use super::*;
    use windows::{
        core::{Interface, HSTRING},
        Win32::{
            System::Com::{
                CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile,
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
            },
            UI::Shell::{IShellLinkW, ShellLink},
        },
    };

    #[cfg(test)]
    use windows::Win32::System::Com::STGM_READ;

    #[cfg(test)]
    const BUFFER_LENGTH: usize = 32_768;

    struct ComApartment;

    impl ComApartment {
        fn initialize() -> windows::core::Result<Self> {
            unsafe {
                CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
            }
            Ok(Self)
        }
    }

    impl Drop for ComApartment {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }

    fn system_rundll32_path() -> Result<PathBuf, String> {
        let system_root = std::env::var_os("SystemRoot")
            .ok_or_else(|| "cannot resolve the Windows system directory".to_owned())?;
        let path = PathBuf::from(system_root)
            .join("System32")
            .join("rundll32.exe");
        if !path.is_file() {
            return Err(format!(
                "Windows rundll32.exe is unavailable at {}",
                path.display()
            ));
        }
        Ok(path)
    }

    fn wide_text(path: &Path) -> HSTRING {
        HSTRING::from(path.to_string_lossy().into_owned())
    }

    fn write_shortcut_inner(path: &Path, uri: &str) -> Result<(), String> {
        let _com = ComApartment::initialize().map_err(|error| error.to_string())?;
        let target = system_rundll32_path()?;
        let arguments = HSTRING::from(format!("url.dll,FileProtocolHandler \"{uri}\""));
        let description = HSTRING::from(SHORTCUT_DESCRIPTION);
        let target = wide_text(&target);
        let path = wide_text(path);
        let shortcut: IShellLinkW = unsafe {
            CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
                .map_err(|error| error.to_string())?
        };

        unsafe {
            shortcut
                .SetPath(&target)
                .map_err(|error| error.to_string())?;
            shortcut
                .SetArguments(&arguments)
                .map_err(|error| error.to_string())?;
            shortcut
                .SetDescription(&description)
                .map_err(|error| error.to_string())?;
            let persisted: IPersistFile = shortcut.cast().map_err(|error| error.to_string())?;
            persisted
                .Save(&path, true)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub(super) fn write_shortcut(path: &Path, uri: &str) -> Result<(), String> {
        write_shortcut_inner(path, uri)
    }

    #[cfg(test)]
    #[derive(Debug, PartialEq, Eq)]
    pub(crate) struct ShortcutDetails {
        pub target: String,
        pub arguments: String,
        pub description: String,
    }

    #[cfg(test)]
    fn decode_wide(buffer: &[u16]) -> String {
        let end = buffer
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(buffer.len());
        String::from_utf16_lossy(&buffer[..end])
    }

    #[cfg(test)]
    pub(crate) fn read_shortcut(path: &Path) -> Result<ShortcutDetails, String> {
        let _com = ComApartment::initialize().map_err(|error| error.to_string())?;
        let path = wide_text(path);
        let shortcut: IShellLinkW = unsafe {
            CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
                .map_err(|error| error.to_string())?
        };
        let persisted: IPersistFile = shortcut.cast().map_err(|error| error.to_string())?;
        unsafe {
            persisted
                .Load(&path, STGM_READ)
                .map_err(|error| error.to_string())?;
            let mut target = [0_u16; BUFFER_LENGTH];
            shortcut
                .GetPath(&mut target, std::ptr::null_mut(), 0)
                .map_err(|error| error.to_string())?;
            let mut arguments = [0_u16; BUFFER_LENGTH];
            shortcut
                .GetArguments(&mut arguments)
                .map_err(|error| error.to_string())?;
            let mut description = [0_u16; BUFFER_LENGTH];
            shortcut
                .GetDescription(&mut description)
                .map_err(|error| error.to_string())?;
            Ok(ShortcutDetails {
                target: decode_wide(&target),
                arguments: decode_wide(&arguments),
                description: decode_wide(&description),
            })
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) mod platform {
    use super::*;

    pub(super) fn write_shortcut(_path: &Path, _uri: &str) -> Result<(), String> {
        Ok(())
    }
}
