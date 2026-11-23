use super::graph::GraphServer;
use crate::{HarnessError, emulator::Emulator};
use aws_wire::sign::Credentials;
use std::{
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

pub struct Stack {
    emulator: Emulator,
    graph: GraphServer,
    credentials: Credentials,
    profile: PathBuf,
    config: PathBuf,
}

impl Stack {
    pub fn start(root: &Path) -> Result<Self, HarnessError> {
        let binary = binary(
            "INVENTORY_EMULATOR_BINARY",
            ".local/runtime/unpacked/opt/aws-emulator/bin/aws-emulator",
        )?;
        let emulator = Emulator::start(&binary, &root.join("emulator"), 900)?;
        let graph = GraphServer::start(root)?;
        let (credentials, profile, config) = issued_profile(root, &emulator)?;
        Ok(Self {
            emulator,
            graph,
            credentials,
            profile,
            config,
        })
    }

    pub fn emulator(&self) -> &Emulator {
        &self.emulator
    }

    pub fn graph(&self) -> &GraphServer {
        &self.graph
    }

    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    pub fn configure(&self, command: &mut tokio::process::Command) {
        command.envs(self.emulator.environment());
        [
            "AWS_ACCESS_KEY_ID",
            "AWS_SECRET_ACCESS_KEY",
            "AWS_SESSION_TOKEN",
            "AWS_SECURITY_TOKEN",
        ]
        .iter()
        .for_each(|name| {
            command.env_remove(name);
        });
        command
            .env("AWS_SHARED_CREDENTIALS_FILE", &self.profile)
            .env("AWS_CONFIG_FILE", &self.config)
            .env("AWS_PROFILE", "default");
    }

    pub fn finish(self) -> Result<(), HarnessError> {
        let emulator = self.emulator.stop();
        let graph = self.graph.stop();
        emulator.and(graph)
    }
}

fn issued_profile(
    root: &Path,
    emulator: &Emulator,
) -> Result<(Credentials, PathBuf, PathBuf), HarnessError> {
    let issued = read_issued(emulator.credentials())?;
    let key = field(&issued, "aws_access_key_id")?;
    let secret = field(&issued, "aws_secret_access_key")?;
    let credentials = Credentials::new(key, secret, None)
        .map_err(|error| HarnessError::invalid(error.to_string()))?;
    let profile = root.join("credentials");
    let config = root.join("config");
    write_secret(
        &profile,
        &format!("[default]\naws_access_key_id = {key}\naws_secret_access_key = {secret}\n"),
    )?;
    write_secret(&config, "[default]\nregion = us-east-1\n")?;
    Ok((credentials, profile, config))
}

fn read_issued(path: &Path) -> Result<String, HarnessError> {
    String::from_utf8(crate::files::read_bytes(path, 65_536)?)
        .map_err(|_| HarnessError::invalid("emulator credentials must be UTF-8"))
}

fn write_secret(path: &Path, content: &str) -> Result<(), HarnessError> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn field<'a>(issued: &'a str, name: &str) -> Result<&'a str, HarnessError> {
    issued
        .lines()
        .map(str::trim)
        .scan(false, |active, line| {
            if line.starts_with('[') {
                *active = line == "[default]";
            }
            Some((*active, line))
        })
        .filter(|(active, _)| *active)
        .find_map(|(_, line)| {
            let (key, value) = line.split_once('=')?;
            (key.trim() == name && !value.trim().is_empty()).then_some(value.trim())
        })
        .ok_or_else(|| HarnessError::invalid("emulator credentials lack default static keys"))
}

pub fn binary(variable: &str, default: &str) -> Result<PathBuf, HarnessError> {
    let path = std::env::var_os(variable)
        .map_or_else(|| super::workspace_root().join(default), PathBuf::from);
    if !path.is_file() {
        return Err(HarnessError::invalid(format!(
            "{variable}: runtime binary missing; install with inventory-harness install or install-graph"
        )));
    }
    Ok(path)
}

#[cfg(test)]
mod tests;
