use super::session::Context;
use super::{
    Decision, Fault, MAX_BODY_BYTES, ProxyError, deadline, errors, http,
    protocol::{BodyPolicy, Connection},
    response,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

pub(crate) async fn canned(
    client: &mut BufReader<TcpStream>,
    context: &Context,
    response: &response::Response,
    request: &http::Request,
    decision: Decision,
) -> Result<(), ProxyError> {
    context.state.response_ready()?;
    tokio::time::sleep(decision.latency).await;
    deadline(client.get_mut().write_all(&response.head)).await?;
    if request.body_policy == BodyPolicy::Normal {
        deadline(client.get_mut().write_all(&response.body)).await?;
    }
    context.state.response_written()
}

pub(crate) async fn forward(
    client: &mut BufReader<TcpStream>,
    context: &Context,
    response: &response::Response,
    decision: Decision,
) -> Result<Connection, ProxyError> {
    context.state.response_ready()?;
    tokio::time::sleep(decision.latency).await;
    deadline(client.get_mut().write_all(&response.head)).await?;
    let body = match decision.fault {
        Some(Fault::Truncation) => response::prefix(&response.body, decision.entropy),
        _ => &response.body,
    };
    deadline(client.get_mut().write_all(body)).await?;
    context.state.response_written()?;
    Ok(match decision.fault {
        Some(Fault::Truncation) => Connection::Close,
        _ => response.connection,
    })
}

pub(crate) async fn reject(
    client: &mut BufReader<TcpStream>,
    context: &Context,
    failure: http::RequestError,
) -> Result<(), ProxyError> {
    context.state.response_ready()?;
    if let Some(scope) = &failure.scope {
        tokio::time::sleep(context.config.delay(scope)).await;
    }
    let (status, code) = http::rejection(&failure.error);
    send_error(client, status, code).await?;
    context.state.response_written()?;
    drain_rejected_peer(client).await?;
    Err(failure.error)
}

pub(crate) async fn bad_gateway(
    client: &mut BufReader<TcpStream>,
    context: &Context,
    decision: Decision,
) -> Result<Connection, ProxyError> {
    context.state.response_ready()?;
    tokio::time::sleep(decision.latency).await;
    send_error(client, 502, "upstream-framing").await?;
    context.state.response_written()?;
    Err(ProxyError::Framing)
}

async fn send_error(
    client: &mut BufReader<TcpStream>,
    status: u16,
    code: &'static str,
) -> Result<(), ProxyError> {
    deadline(client.get_mut().write_all(&errors::rejection(status, code))).await?;
    deadline(client.get_mut().shutdown()).await
}

// Send FIN before draining so unread oversized input cannot replace rejection
// bytes with RST. The enclosing session's cancellation drops both halves.
async fn drain_rejected_peer(client: &mut BufReader<TcpStream>) -> Result<(), ProxyError> {
    const BUFFER_BYTES: usize = 4_096;
    const MAX_READS: usize = (MAX_BODY_BYTES / BUFFER_BYTES)
        .saturating_add(super::MAX_HEAD_BYTES / BUFFER_BYTES)
        .saturating_add(1);
    let mut buffer = [0u8; BUFFER_BYTES];
    for _ in 0..MAX_READS {
        if deadline(client.read(&mut buffer)).await? == 0 {
            return Ok(());
        }
    }
    Err(ProxyError::Limit("rejected peer data"))
}
