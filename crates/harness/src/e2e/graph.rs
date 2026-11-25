use crate::{HarnessError, emulator::ChildOwner};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

const IO_TIMEOUT: Duration = Duration::from_secs(2);
const REDIS_OK: [u8; 5] = *b"+OK\r\n";
const FLUSHALL: &[u8] = b"*1\r\n$8\r\nFLUSHALL\r\n";

pub struct GraphServer {
    child: ChildOwner,
    address: SocketAddr,
    auth: String,
    url: String,
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
        let address = TcpListener::bind("127.0.0.1:0")?.local_addr()?;
        let password = instance_password()?;
        let auth = auth_request(&password);
        let mut config = tempfile::NamedTempFile::new()?;
        writeln!(config, "requirepass {password}")?;
        let graph = Self {
            child: spawn(&server, &module, root, address.port(), config.path())?,
            address,
            auth,
            url: format!("redis://:{password}@{address}"),
        };
        graph.await_ready()?;
        config.close()?;
        Ok(graph)
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn stop(mut self) -> Result<(), HarnessError> {
        self.child.terminate().map(drop)
    }

    pub(crate) async fn clear(&self) -> Result<(), HarnessError> {
        ensure_graph_running(&self.child)?;
        clear_graph(self.address, &self.auth, &self.child).await
    }

    fn await_ready(&self) -> Result<(), HarnessError> {
        (0..300)
            .find_map(|_| self.poll())
            .ok_or_else(|| HarnessError::invalid("FalkorDB readiness exceeded 300 attempts"))?
    }

    fn poll(&self) -> Option<Result<(), HarnessError>> {
        match self.child.exited() {
            Err(error) => Some(Err(error)),
            Ok(Some(status)) => Some(Err(HarnessError::invalid(format!(
                "FalkorDB exited with {status:?}; see redis.log"
            )))),
            Ok(None)
                if module_list(self.address, &self.auth)
                    .is_ok_and(|reply| reply.contains("$5\r\ngraph\r\n")) =>
            {
                Some(Ok(()))
            }
            Ok(None) => {
                std::thread::sleep(Duration::from_millis(100));
                None
            }
        }
    }
}

fn spawn(
    server: &Path,
    module: &Path,
    root: &Path,
    port: u16,
    config: &Path,
) -> Result<ChildOwner, HarnessError> {
    let log = std::fs::File::create(root.join("redis.log"))?;
    ChildOwner::spawn(
        Command::new(server)
            .arg(config)
            .args(["--bind", "127.0.0.1", "--protected-mode", "yes", "--port"])
            .arg(port.to_string())
            .args([
                "--daemonize",
                "no",
                "--save",
                "",
                "--appendonly",
                "no",
                "--dir",
            ])
            .arg(root)
            .arg("--loadmodule")
            .arg(module)
            .stdin(Stdio::null())
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log)),
    )
    .map_err(HarnessError::from)
}

fn module_list(address: SocketAddr, auth: &str) -> Result<String, HarnessError> {
    let mut stream = TcpStream::connect_timeout(&address, IO_TIMEOUT)?;
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    stream.write_all(auth.as_bytes())?;
    let mut response = [0_u8; 5];
    stream.read_exact(&mut response)?;
    confirm_auth(response)?;
    stream.write_all(b"*2\r\n$6\r\nMODULE\r\n$4\r\nLIST\r\n")?;
    let mut reply = [0_u8; 4096];
    let read = stream.read(&mut reply)?;
    let bytes = reply
        .get(..read)
        .ok_or_else(|| HarnessError::invalid("invalid graph reply size"))?;
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

fn instance_password() -> Result<String, HarnessError> {
    let mut bytes = [0_u8; 32];
    let filled = rustix::rand::getrandom(&mut bytes, rustix::rand::GetRandomFlags::empty())
        .map_err(std::io::Error::from)?;
    if filled != bytes.len() {
        return Err(HarnessError::invalid("incomplete Redis password entropy"));
    }
    Ok(hex::encode(bytes))
}

fn auth_request(password: &str) -> String {
    format!("*2\r\n$4\r\nAUTH\r\n${}\r\n{password}\r\n", password.len())
}

fn confirm_auth(response: [u8; 5]) -> Result<(), HarnessError> {
    if response != REDIS_OK {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "graph server did not authenticate the owned instance",
        )
        .into());
    }
    Ok(())
}

fn ensure_graph_running(child: &ChildOwner) -> Result<(), HarnessError> {
    match child.exited()? {
        Some(_) => Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "owned graph child has exited",
        )
        .into()),
        None => Ok(()),
    }
}

async fn clear_graph(
    address: SocketAddr,
    auth: &str,
    child: &ChildOwner,
) -> Result<(), HarnessError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut stream = tokio::net::TcpStream::connect(address).await?;
        stream.write_all(auth.as_bytes()).await?;
        let mut response = [0_u8; 5];
        stream.read_exact(&mut response).await?;
        confirm_auth(response)?;
        ensure_graph_running(child)?;
        stream.write_all(FLUSHALL).await?;
        stream.read_exact(&mut response).await?;
        if response != REDIS_OK {
            return Err(HarnessError::invalid("graph reset failed"));
        }
        Ok(())
    })
    .await
    .map_err(|_| HarnessError::invalid("graph reset exceeded 5 seconds"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn live_child() -> Result<ChildOwner, HarnessError> {
        ChildOwner::spawn(
            Command::new("/bin/cat")
                .stdin(Stdio::piped())
                .stdout(Stdio::null()),
        )
        .map_err(HarnessError::from)
    }

    #[tokio::test]
    async fn given_replacement_graph_when_authentication_fails_then_no_mutation_is_sent()
    -> TestResult {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr()?;
        let auth = auth_request(&instance_password()?);
        let child = live_child()?;
        let server = async {
            let (mut stream, _) = listener.accept().await?;
            let mut first = [0; 18];
            stream.read_exact(&mut first).await?;
            if first.as_slice() == FLUSHALL {
                stream.write_all(&REDIS_OK).await?;
                return Ok::<_, std::io::Error>(first.len());
            }
            let mut rest = vec![0; auth.len().saturating_sub(first.len())];
            stream.read_exact(&mut rest).await?;
            assert_eq!(
                [first.as_slice(), rest.as_slice()].concat(),
                auth.as_bytes()
            );
            stream.write_all(b"-ERR ").await?;
            stream.read_to_end(&mut Vec::new()).await
        };

        let (reset, received) = tokio::time::timeout(Duration::from_secs(10), async {
            tokio::join!(clear_graph(address, &auth, &child), server)
        })
        .await?;

        assert_eq!(received?, 0);
        assert!(matches!(reset, Err(HarnessError::Io(error))
            if error.kind() == std::io::ErrorKind::PermissionDenied));
        Ok(())
    }

    #[tokio::test]
    async fn given_owned_graph_when_authenticated_then_flushall_uses_the_same_connection()
    -> TestResult {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr()?;
        let auth = auth_request(&instance_password()?);
        let child = live_child()?;
        let server = async {
            let (mut stream, _) = listener.accept().await?;
            let mut received = vec![0; auth.len()];
            stream.read_exact(&mut received).await?;
            assert_eq!(received, auth.as_bytes());
            stream.write_all(&REDIS_OK).await?;
            let mut request = [0; 18];
            stream.read_exact(&mut request).await?;
            assert_eq!(&request, b"*1\r\n$8\r\nFLUSHALL\r\n");
            stream.write_all(&REDIS_OK).await
        };

        let (reset, served) = tokio::time::timeout(Duration::from_secs(10), async {
            tokio::join!(clear_graph(address, &auth, &child), server)
        })
        .await?;
        served?;
        reset?;
        Ok(())
    }

    #[tokio::test]
    async fn given_graph_child_exits_during_authentication_when_resetting_then_no_mutation_is_sent()
    -> TestResult {
        use rustix::process::{WaitId, WaitIdOptions};

        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr()?;
        let auth = auth_request(&instance_password()?);
        let mut child = live_child()?;
        let pid = child.pid();
        let input = child
            .child_mut()
            .stdin
            .take()
            .ok_or("missing child stdin")?;
        let server = async {
            let (mut stream, _) = listener.accept().await?;
            let mut received = vec![0; auth.len()];
            stream.read_exact(&mut received).await?;
            drop(input);
            let status = tokio::task::spawn_blocking(move || {
                rustix::process::waitid(
                    WaitId::Pid(pid),
                    WaitIdOptions::EXITED | WaitIdOptions::NOWAIT,
                )
            })
            .await??;
            assert!(matches!(status, Some(exit) if exit.exit_status() == Some(0)));
            stream.write_all(&REDIS_OK).await?;
            Ok::<_, Box<dyn std::error::Error>>(stream.read_to_end(&mut Vec::new()).await?)
        };

        let (reset, received) = tokio::time::timeout(Duration::from_secs(10), async {
            tokio::join!(clear_graph(address, &auth, &child), server)
        })
        .await?;

        assert_eq!(received?, 0);
        assert!(matches!(reset, Err(HarnessError::Io(error))
            if error.kind() == std::io::ErrorKind::BrokenPipe));
        Ok(())
    }
}
