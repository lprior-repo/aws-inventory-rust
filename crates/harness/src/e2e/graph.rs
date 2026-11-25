use crate::HarnessError;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const IO_TIMEOUT: Duration = Duration::from_secs(2);

pub struct GraphServer {
    child: Child,
    url: String,
}

impl Drop for GraphServer {
    fn drop(&mut self) {
        if let Err(error) = self.terminate() {
            eprintln!("graph child cleanup failed: {error}");
        }
    }
}

impl GraphServer {
    pub fn start(root: &Path) -> Result<Self, HarnessError> {
        let server = super::stack::binary(
            "INVENTORY_REDIS_SERVER",
            ".local/runtime/graph/redis-8.10.3/src/redis-server",
        )?;
        let module = super::stack::binary(
            "INVENTORY_FALKOR_MODULE",
            ".local/runtime/graph/falkordb.so",
        )?;
        let port = TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
        let mut graph = Self {
            child: spawn(&server, &module, root, port)?,
            url: format!("redis://127.0.0.1:{port}"),
        };
        graph.await_ready(port)?;
        Ok(graph)
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn stop(mut self) -> Result<(), HarnessError> {
        self.terminate()
    }

    fn terminate(&mut self) -> Result<(), HarnessError> {
        if self.child.try_wait()?.is_none() {
            self.child.kill()?;
        }
        self.child.wait()?;
        Ok(())
    }

    fn await_ready(&mut self, port: u16) -> Result<(), HarnessError> {
        (0..300)
            .find_map(|_| self.poll(port))
            .ok_or_else(|| HarnessError::invalid("FalkorDB readiness exceeded 300 attempts"))?
    }

    fn poll(&mut self, port: u16) -> Option<Result<(), HarnessError>> {
        match self.child.try_wait() {
            Err(error) => Some(Err(error.into())),
            Ok(Some(status)) => Some(Err(HarnessError::invalid(format!(
                "FalkorDB exited with {status}; see redis.log"
            )))),
            Ok(None) if module_list(port).is_ok_and(|reply| reply.contains("$5\r\ngraph\r\n")) => {
                Some(Ok(()))
            }
            Ok(None) => {
                std::thread::sleep(Duration::from_millis(100));
                None
            }
        }
    }
}

fn spawn(server: &Path, module: &Path, root: &Path, port: u16) -> Result<Child, HarnessError> {
    let log = std::fs::File::create(root.join("redis.log"))?;
    Ok(Command::new(server)
        .args(["--bind", "127.0.0.1", "--protected-mode", "yes", "--port"])
        .arg(port.to_string())
        .args(["--save", "", "--appendonly", "no", "--dir"])
        .arg(root)
        .arg("--loadmodule")
        .arg(module)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .spawn()?)
}

fn module_list(port: u16) -> Result<String, HarnessError> {
    let mut stream = TcpStream::connect_timeout(&([127, 0, 0, 1], port).into(), IO_TIMEOUT)?;
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    stream.write_all(b"*2\r\n$6\r\nMODULE\r\n$4\r\nLIST\r\n")?;
    let mut reply = [0_u8; 4096];
    let read = stream.read(&mut reply)?;
    let bytes = reply
        .get(..read)
        .ok_or_else(|| HarnessError::invalid("invalid graph reply size"))?;
    Ok(String::from_utf8_lossy(bytes).into_owned())
}
