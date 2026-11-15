#[path = "stack/graph.rs"]
mod graph;
use super::guard::{Environment, static_keys};
use anyhow::{Context, Result, ensure};
use aws_wire::sign::Credentials;
use graph::GraphServer;
use inventory_harness::emulator::Emulator;
use std::path::{Path, PathBuf};

const EMULATOR_DEFAULT: &str = ".local/runtime/unpacked/opt/aws-emulator/bin/aws-emulator";
const REGION: &str = "us-east-1";
const PROFILE: &str = "default";
const GRAPH_NAME: &str = "aws_inventory_e2e";
const IDLE_SECONDS: u64 = 900;

pub(super) struct Stack {
    environment: Environment,
    credentials: Credentials,
    _graph: GraphServer,
    _emulator: Emulator,
    _directory: tempfile::TempDir,
}

struct Profile {
    config: PathBuf,
    credentials: PathBuf,
    keys: Credentials,
}

impl Stack {
    pub(super) async fn start() -> Result<Self> {
        tokio::task::spawn_blocking(Self::start_blocking)
            .await
            .context("joining E2E stack startup")?
    }

    pub(super) fn environment(&self) -> &Environment {
        &self.environment
    }

    pub(super) fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    fn start_blocking() -> Result<Self> {
        let directory = tempfile::tempdir()?;
        let emulator_binary = binary("INVENTORY_EMULATOR_BINARY", EMULATOR_DEFAULT)?;
        let work = directory.path().join("emulator");
        let emulator = Emulator::start(&emulator_binary, &work, IDLE_SECONDS)
            .context("starting the pinned AWS emulator")?;
        let graph = GraphServer::start(directory.path())?;
        let profile = write_profile(directory.path(), &emulator)?;
        let environment = environment(&emulator, graph.url(), &profile);
        environment.validate()?;
        Ok(Self {
            environment,
            credentials: profile.keys,
            _graph: graph,
            _emulator: emulator,
            _directory: directory,
        })
    }
}

pub(super) fn binary(variable: &str, default: &str) -> Result<PathBuf> {
    let path = std::env::var_os(variable).map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join(default),
        PathBuf::from,
    );
    ensure!(
        path.is_file(),
        "{variable} is unset and {} is missing; install the pinned runtime with \
         `cargo run -p inventory-harness -- install .local/runtime` and \
         `cargo run -p inventory-harness -- install-graph .local/runtime/graph`",
        path.display()
    );
    Ok(path)
}

fn write_profile(root: &Path, emulator: &Emulator) -> Result<Profile> {
    let directory = root.join("aws");
    std::fs::create_dir_all(&directory)?;
    let issued = std::fs::read_to_string(emulator.credentials())
        .context("reading credentials issued by the emulator")?;
    let (id, secret) = static_keys(&issued, PROFILE)?;
    let credentials = directory.join("credentials");
    std::fs::write(
        &credentials,
        format!("[{PROFILE}]\naws_access_key_id = {id}\naws_secret_access_key = {secret}\n"),
    )?;
    let config = directory.join("config");
    std::fs::write(&config, format!("[{PROFILE}]\nregion = {REGION}\n"))?;
    Ok(Profile {
        config,
        credentials,
        keys: Credentials::new(&id, &secret, None)?,
    })
}

fn environment(emulator: &Emulator, graph_url: &str, profile: &Profile) -> Environment {
    Environment {
        endpoint: emulator.endpoint().to_owned(),
        ca_bundle: emulator.ca_bundle().to_owned(),
        graph_url: graph_url.to_owned(),
        graph_name: GRAPH_NAME.to_owned(),
        region: REGION.to_owned(),
        profile: PROFILE.to_owned(),
        catalog: Path::new(env!("CARGO_MANIFEST_DIR")).join("catalog/aws-api.catalog"),
        aws_config: profile.config.clone(),
        aws_credentials: profile.credentials.clone(),
    }
}
