#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MachineEntraPolicy {
    pub(crate) client_id: Option<String>,
    pub(crate) tenant_id: Option<String>,
}

impl MachineEntraPolicy {
    #[cfg(test)]
    pub(crate) fn new(client_id: Option<String>, tenant_id: Option<String>) -> Self {
        Self {
            client_id,
            tenant_id,
        }
    }
}

#[cfg(target_os = "windows")]
const POLICY_KEY: &str = r"SOFTWARE\Policies\Videoclinic\DMS";
#[cfg(target_os = "windows")]
const CLIENT_ID_VALUE: &str = "EntraClientId";
#[cfg(target_os = "windows")]
const TENANT_ID_VALUE: &str = "EntraTenantId";

#[cfg(target_os = "windows")]
pub(crate) fn load_machine_entra_policy() -> Result<MachineEntraPolicy, String> {
    use std::io::ErrorKind;
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

    let machine = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = match machine.open_subkey(POLICY_KEY) {
        Ok(key) => key,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(MachineEntraPolicy::default())
        }
        Err(error) => {
            return Err(format!(
                "cannot read Windows policy key HKLM\\{POLICY_KEY}: {error}"
            ));
        }
    };

    fn read_string(key: &RegKey, name: &str) -> Result<Option<String>, String> {
        use std::io::ErrorKind;

        match key.get_value::<String, _>(name) {
            Ok(value) => Ok(Some(value.trim().to_owned())),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!(
                "cannot read Windows policy value {name} from HKLM\\{POLICY_KEY}: {error}"
            )),
        }
    }

    Ok(MachineEntraPolicy {
        client_id: read_string(&key, CLIENT_ID_VALUE)?,
        tenant_id: read_string(&key, TENANT_ID_VALUE)?,
    })
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn load_machine_entra_policy() -> Result<MachineEntraPolicy, String> {
    Ok(MachineEntraPolicy::default())
}
