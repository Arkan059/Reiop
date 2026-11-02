use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct BackupEntry {
    pub hive: String,
    pub path: String,
    pub value_name: String,
    pub data: Option<serde_json::Value>,
    pub kind: Option<u32>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Backup {
    pub registry: Vec<BackupEntry>,
    pub power_scheme_guid: Option<String>,
}

pub struct BackupManager {
    path: PathBuf,
    pub data: Backup,
}

impl BackupManager {
    pub fn new() -> Self {
        let path = PathBuf::from("backup.json");
        let data = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Backup::default()
        };
        Self { path, data }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        fs::write(&self.path, serde_json::to_string_pretty(&self.data)?)?;
        Ok(())
    }

    pub fn record_registry(
        &mut self,
        hive: &str,
        path: &str,
        value_name: &str,
        data: Option<serde_json::Value>,
        kind: Option<u32>,
    ) {
        self.data.registry.push(BackupEntry {
            hive: hive.to_string(),
            path: path.to_string(),
            value_name: value_name.to_string(),
            data,
            kind,
        });
    }

    pub fn record_power_scheme(&mut self, guid: &str) {
        self.data.power_scheme_guid = Some(guid.to_string());
    }

    pub fn restore_all(&self) -> anyhow::Result<()> {
        use winreg::enums::*;
        use winreg::RegKey;

        for entry in &self.data.registry {
            let hive = match entry.hive.as_str() {
                "HKCU" => HKEY_CURRENT_USER,
                "HKLM" => HKEY_LOCAL_MACHINE,
                _ => continue,
            };
            let key = RegKey::predef(hive).open_subkey_with_flags(&entry.path, KEY_SET_VALUE)?;
            match &entry.data {
                Some(v) if v.is_string() => {
                    key.set_value(&entry.value_name, &v.as_str().unwrap())?;
                }
                Some(v) if v.is_number() => {
                    key.set_value(&entry.value_name, &(v.as_u64().unwrap() as u32))?;
                }
                Some(v) if v.is_array() => {
                    let bytes: Vec<u8> = v
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_u64().unwrap() as u8)
                        .collect();
                    key.set_raw_value(
                        &entry.value_name,
                        &winreg::RegValue {
                            bytes,
                            vtype: REG_BINARY,
                        },
                    )?;
                }
                None => {
                    let _ = key.delete_value(&entry.value_name);
                }
                _ => {}
            }
        }
        if let Some(guid) = &self.data.power_scheme_guid {
            crate::tweaks::activate_power_scheme_by_guid(guid)?;
        }
        Ok(())
    }
}