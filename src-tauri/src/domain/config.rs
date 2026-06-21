use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize};

const SUPPORTED_CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DevappConfig {
    pub version: u32,
    #[serde(default)]
    pub env: IndexMap<String, String>,
    pub processes: IndexMap<String, ProcessConfig>,
}

impl DevappConfig {
    pub fn validate_version(&self) -> Result<(), String> {
        if self.version == SUPPORTED_CONFIG_VERSION {
            Ok(())
        } else {
            Err(format!(
                "unsupported config version {}, expected {}",
                self.version, SUPPORTED_CONFIG_VERSION
            ))
        }
    }
}

impl<'de> Deserialize<'de> for DevappConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct RawDevappConfig {
            version: u32,
            #[serde(default)]
            env: IndexMap<String, String>,
            processes: IndexMap<String, ProcessConfig>,
        }

        let raw = RawDevappConfig::deserialize(deserializer)?;
        let config = DevappConfig {
            version: raw.version,
            env: raw.env,
            processes: raw.processes,
        };

        config
            .validate_version()
            .map_err(serde::de::Error::custom)?;

        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProcessConfig {
    pub kind: ProcessKind,
    pub cmd: String,
    #[serde(default, rename = "dependsOn")]
    pub depends_on: IndexMap<String, DependencyCondition>,
    #[serde(default)]
    pub ready: Option<ReadyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProcessKind {
    Task,
    Service,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DependencyCondition {
    Success,
    Ready,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ReadyConfig {
    Http(HttpReadyConfig),
    Log(LogReadyConfig),
    Delay(DelayReadyConfig),
    Command(CommandReadyConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HttpReadyConfig {
    pub url: String,
    #[serde(default)]
    pub interval_ms: Option<u64>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LogReadyConfig {
    pub pattern: String,
    #[serde(default)]
    pub regex: bool,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DelayReadyConfig {
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandReadyConfig {
    pub cmd: String,
    #[serde(default)]
    pub interval_ms: Option<u64>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}
