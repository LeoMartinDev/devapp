use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    domain::config::{DevappConfig, ReadyConfig},
    error::AppError,
};

const PROJECT_CONFIG_FILE_NAME: &str = "devapp.yml";

#[derive(Debug, Clone)]
pub struct LoadedProjectConfig {
    pub config_path: PathBuf,
    pub base_dir: PathBuf,
    pub config: DevappConfig,
    pub raw_yaml: String,
}

pub fn load_config(config_path: &Path) -> Result<LoadedProjectConfig, AppError> {
    let canonical_path = fs::canonicalize(config_path).map_err(|error| {
        AppError::config(format!(
            "unable to resolve config path {}: {error}",
            config_path.display()
        ))
    })?;
    let raw_yaml = fs::read_to_string(&canonical_path)?;
    let config: DevappConfig = serde_yaml::from_str(&raw_yaml)?;
    validate_graph(&config)?;

    let base_dir = canonical_path
        .parent()
        .ok_or_else(|| AppError::config("config file must have a parent directory"))?
        .to_path_buf();

    Ok(LoadedProjectConfig {
        config_path: canonical_path,
        base_dir,
        config,
        raw_yaml,
    })
}

pub fn parse_config_document(
    config_path: &Path,
    raw_yaml: &str,
) -> Result<LoadedProjectConfig, AppError> {
    let config: DevappConfig = serde_yaml::from_str(raw_yaml)?;
    validate_graph(&config)?;
    let canonical_path = canonicalize_for_write_target(config_path)?;
    let base_dir = canonical_path
        .parent()
        .ok_or_else(|| AppError::config("config file must have a parent directory"))?
        .to_path_buf();

    Ok(LoadedProjectConfig {
        config_path: canonical_path,
        base_dir,
        config,
        raw_yaml: raw_yaml.to_string(),
    })
}

pub fn find_project_config(base_dir: &Path) -> Result<Option<PathBuf>, AppError> {
    let canonical_base_dir = fs::canonicalize(base_dir).map_err(|error| {
        AppError::config(format!(
            "unable to resolve project directory {}: {error}",
            base_dir.display()
        ))
    })?;
    let config_path = canonical_base_dir.join(PROJECT_CONFIG_FILE_NAME);
    if config_path.is_file() {
        Ok(Some(config_path))
    } else {
        Ok(None)
    }
}

pub fn validate_graph(config: &DevappConfig) -> Result<(), AppError> {
    if config.processes.is_empty() {
        return Err(AppError::validation(
            "configuration must declare at least one process",
        ));
    }

    for (name, process) in &config.processes {
        if name.trim().is_empty() {
            return Err(AppError::validation("process name cannot be empty"));
        }
        if process.cmd.trim().is_empty() {
            return Err(AppError::validation(format!(
                "process `{name}` must declare a non-empty cmd"
            )));
        }
        if let Some(ready) = &process.ready {
            validate_ready_config(name, ready)?;
        }
        for dependency_name in process.depends_on.keys() {
            if !config.processes.contains_key(dependency_name) {
                return Err(AppError::validation(format!(
                    "process `{name}` depends on unknown process `{dependency_name}`"
                )));
            }
        }
    }

    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    for name in config.processes.keys() {
        visit_process(name, config, &mut visiting, &mut visited)?;
    }

    Ok(())
}

fn validate_ready_config(process_name: &str, ready: &ReadyConfig) -> Result<(), AppError> {
    match ready {
        ReadyConfig::Http(config) => {
            let url = reqwest::Url::parse(&config.url).map_err(|error| {
                AppError::validation(format!(
                    "process `{process_name}` has invalid http readiness URL `{}`: {error}",
                    config.url
                ))
            })?;
            if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
                return Err(AppError::validation(format!(
                    "process `{process_name}` http readiness URL must be an absolute http(s) URL"
                )));
            }
        }
        ReadyConfig::Log(config) if config.pattern.trim().is_empty() => {
            return Err(AppError::validation(format!(
                "process `{process_name}` log readiness pattern cannot be empty"
            )));
        }
        ReadyConfig::Command(config) if config.cmd.trim().is_empty() => {
            return Err(AppError::validation(format!(
                "process `{process_name}` command readiness cmd cannot be empty"
            )));
        }
        ReadyConfig::Delay(_) | ReadyConfig::Log(_) | ReadyConfig::Command(_) => {}
    }

    Ok(())
}

fn visit_process(
    name: &str,
    config: &DevappConfig,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
) -> Result<(), AppError> {
    if visited.contains(name) {
        return Ok(());
    }
    if !visiting.insert(name.to_string()) {
        return Err(AppError::validation(format!(
            "dependency cycle detected at process `{name}`"
        )));
    }

    let process = config
        .processes
        .get(name)
        .ok_or_else(|| AppError::validation(format!("unknown process `{name}`")))?;
    for dependency_name in process.depends_on.keys() {
        visit_process(dependency_name, config, visiting, visited)?;
    }

    visiting.remove(name);
    visited.insert(name.to_string());
    Ok(())
}

fn canonicalize_for_write_target(path: &Path) -> Result<PathBuf, AppError> {
    if path.exists() {
        return fs::canonicalize(path).map_err(AppError::from);
    }

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    let parent = absolute_path
        .parent()
        .ok_or_else(|| AppError::config("config path must have a parent directory"))?;
    let canonical_parent = fs::canonicalize(parent).map_err(|error| {
        AppError::config(format!(
            "unable to resolve config parent directory {}: {error}",
            parent.display()
        ))
    })?;
    let file_name = absolute_path
        .file_name()
        .ok_or_else(|| AppError::config("config path must include a file name"))?;

    Ok(canonical_parent.join(file_name))
}

pub fn serialize_config(config: &DevappConfig) -> Result<String, AppError> {
    serde_yaml::to_string(config).map_err(|error| AppError::config(error.to_string()))
}

pub fn process_dependency_statuses(config: &DevappConfig) -> HashMap<String, Vec<String>> {
    config
        .processes
        .iter()
        .map(|(name, process)| {
            (
                name.clone(),
                process.depends_on.keys().cloned().collect::<Vec<_>>(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::*;

    fn parse(raw_yaml: &str) -> Result<LoadedProjectConfig, AppError> {
        let base_dir = std::env::temp_dir().join("devapp-config-loader");
        fs::create_dir_all(&base_dir).expect("create config loader temp dir");
        parse_config_document(&base_dir.join("devapp.yml"), raw_yaml)
    }

    #[test]
    fn parses_realistic_yaml_document() {
        let loaded = parse(
            r#"
version: 1
env:
  PORT: "5173"
processes:
  setup:
    kind: task
    cmd: deno task check
  web:
    kind: service
    cmd: deno task dev
    dependsOn:
      setup: success
    ready:
      type: http
      url: http://127.0.0.1:5173/health
      intervalMs: 25
      timeoutMs: 500
"#,
        )
        .expect("config should parse");

        assert_eq!(loaded.base_dir, Path::new("/tmp/devapp-config-loader"));
        assert_eq!(loaded.config.version, 1);
        assert_eq!(loaded.config.env["PORT"], "5173");
        assert!(loaded.config.processes.contains_key("setup"));
        assert_eq!(
            loaded.config.processes["web"].depends_on["setup"],
            crate::domain::config::DependencyCondition::Success
        );
    }

    #[test]
    fn rejects_unknown_dependency() {
        let error = parse(
            r#"
version: 1
processes:
  web:
    kind: service
    cmd: deno task dev
    dependsOn:
      missing: ready
"#,
        )
        .expect_err("unknown dependency should fail");

        assert!(error
            .to_string()
            .contains("depends on unknown process `missing`"));
    }

    #[test]
    fn rejects_dependency_cycle() {
        let error = parse(
            r#"
version: 1
processes:
  api:
    kind: service
    cmd: cargo run
    dependsOn:
      web: ready
  web:
    kind: service
    cmd: deno task dev
    dependsOn:
      api: ready
"#,
        )
        .expect_err("cycle should fail");

        assert!(error.to_string().contains("dependency cycle detected"));
    }

    #[test]
    fn rejects_unsupported_version() {
        let error = parse(
            r#"
version: 2
processes:
  web:
    kind: service
    cmd: deno task dev
"#,
        )
        .expect_err("unsupported version should fail");

        assert!(error.to_string().contains("unsupported config version 2"));
    }

    #[test]
    fn rejects_relative_http_readiness_url() {
        let error = parse(
            r#"
version: 1
processes:
  web:
    kind: service
    cmd: deno task dev
    ready:
      type: http
      url: /health
"#,
        )
        .expect_err("relative readiness URL should fail");

        assert!(error.to_string().contains("invalid http readiness URL"));
    }

    #[test]
    fn finds_project_devapp_yml() {
        let root = std::env::temp_dir().join(format!(
            "devapp-config-loader-test-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).expect("create temp project");
        let config_path = root.join(PROJECT_CONFIG_FILE_NAME);
        fs::write(&config_path, "version: 1\nprocesses: {}\n").expect("write config");

        assert_eq!(
            find_project_config(&root).expect("find config"),
            Some(config_path)
        );

        let _ = fs::remove_dir_all(root);
    }
}
