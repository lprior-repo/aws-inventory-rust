use super::{
    clock::within,
    support::{TEST_RESPONSE_CAP, TestResult, field},
};
use inventory_harness::proxy::{MAX_BODY_BYTES, MAX_HEAD_BYTES};
use std::{error::Error, io, net::SocketAddr};
use tokio::{
    io::{AsyncRead, AsyncReadExt, BufReader},
    net::TcpStream,
};

pub async fn head(reader: &mut BufReader<TcpStream>) -> Result<Vec<u8>, Box<dyn Error>> {
    within("TCP response head", head_inner(reader)).await
}

async fn head_inner(reader: &mut BufReader<TcpStream>) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    for _ in 0..MAX_HEAD_BYTES {
        bytes.push(reader.read_u8().await?);
        if bytes.ends_with(b"\r\n\r\n") {
            return Ok(bytes);
        }
    }
    Err("test response head limit".into())
}

pub async fn message(reader: &mut BufReader<TcpStream>) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = head(reader).await?;
    let size = field(&bytes, "content-length")?.parse::<usize>()?;
    if size > MAX_BODY_BYTES {
        return Err("test response body limit".into());
    }
    let mut body = vec![0; size];
    within("TCP response body", reader.read_exact(&mut body)).await?;
    bytes.extend_from_slice(&body);
    Ok(bytes)
}

pub async fn remaining(reader: impl AsyncRead + Unpin) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    within(
        "TCP response EOF",
        reader.take(TEST_RESPONSE_CAP).read_to_end(&mut bytes),
    )
    .await?;
    Ok(bytes)
}

pub fn held(client: &TcpStream) {
    assert_eq!(
        client.try_read(&mut [0]).err().map(|error| error.kind()),
        Some(std::io::ErrorKind::WouldBlock)
    );
}

const MAX_REBIND_PROBES: usize = 1024;
pub async fn rebound(address: SocketAddr) -> TestResult {
    within("scheduled refusal automatically rebinds", async {
        for _ in 0..MAX_REBIND_PROBES {
            match TcpStream::connect(address).await {
                Ok(_probe) => return Ok(()),
                Err(error) if error.kind() == io::ErrorKind::ConnectionRefused => {
                    tokio::task::yield_now().await;
                }
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "automatic refusal rebind",
        ))
    })
    .await
}

pub async fn refused(address: SocketAddr) -> TestResult {
    within("TCP connect is refused during the refusal window", async {
        let outcome = TcpStream::connect(address).await;
        assert_eq!(
            outcome.err().map(|error| error.kind()),
            Some(io::ErrorKind::ConnectionRefused)
        );
        Ok::<(), Box<dyn Error>>(())
    })
    .await
}
