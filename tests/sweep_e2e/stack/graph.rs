use super::binary;
use anyhow::{Context, Result, anyhow};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const REDIS_DEFAULT: &str = ".local/runtime/graph/redis-8.10.3/src/redis-server";
const FALKOR_DEFAULT: &str = ".local/runtime/graph/falkordb.so";
const READY_ATTEMPTS: usize = 300;
const READY_POLL: Duration = Duration::from_millis(100);
const IO_TIMEOUT: Duration = Duration::from_secs(2);

pub(super) struct GraphServer {
    child: Child,
    url: String,
}

impl Drop for GraphServer {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            drop(self.child.kill());
            drop(self.child.wait());
        }
    }
}

impl GraphServer {
    pub(super) fn start(root: &Path) -> Result<Self> {
        let server = binary("INVENTORY_REDIS_SERVER", REDIS_DEFAULT)?;
        let module = binary("INVENTORY_FALKOR_MODULE", FALKOR_DEFAULT)?;
        let port = free_port()?;
        let mut graph = Self {
            child: spawn(&server, &module, root, port)?,
            url: format!("redis://127.0.0.1:{port}"),
        };
        graph.await_ready(port, &root.join("redis.log"))?;
        Ok(graph)
    }

    pub(super) fn url(&self) -> &str {
        &self.url
    }

    fn await_ready(&mut self, port: u16, log: &Path) -> Result<()> {
        (0..READY_ATTEMPTS)
            .find_map(|_| self.poll(port, log))
            .ok_or_else(|| anyhow!("redis-server with FalkorDB was not ready on port {port}"))?
    }

    fn poll(&mut self, port: u16, log: &Path) -> Option<Result<()>> {
        match self.child.try_wait() {
            Err(error) => Some(Err(error.into())),
            Ok(Some(status)) => Some(Err(exited(status, log))),
            Ok(None) if falkor_loaded(port) => Some(Ok(())),
            Ok(None) => {
                std::thread::sleep(READY_POLL);
                None
            }
        }
    }
}

fn exited(status: std::process::ExitStatus, log: &Path) -> anyhow::Error {
    let output = std::fs::read_to_string(log).unwrap_or_default();
    anyhow!("redis-server with FalkorDB exited with {status}: {output}")
}

fn spawn(server: &Path, module: &Path, root: &Path, port: u16) -> Result<Child> {
    let log = std::fs::File::create(root.join("redis.log"))?;
    Command::new(server)
        .args(["--bind", "127.0.0.1", "--protected-mode", "yes", "--port"])
        .arg(port.to_string())
        .args(["--save", "", "--appendonly", "no", "--dir"])
        .arg(root)
        .arg("--loadmodule")
        .arg(module)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .spawn()
        .context("spawning redis-server with the FalkorDB module")
}

fn free_port() -> Result<u16> {
    Ok(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port())
}

fn falkor_loaded(port: u16) -> bool {
    module_list(port).is_ok_and(|reply| reply.contains("$5\r\ngraph\r\n"))
}

fn module_list(port: u16) -> Result<String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    stream.write_all(b"*2\r\n$6\r\nMODULE\r\n$4\r\nLIST\r\n")?;
    let mut reply = [0_u8; 4096];
    let read = stream.read(&mut reply)?;
    Ok(String::from_utf8_lossy(reply.get(..read).unwrap_or_default()).into_owned())
}
