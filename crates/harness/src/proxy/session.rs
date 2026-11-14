use super::{
    Decision, Fault, MAX_REQUESTS, ProxyError, deadline, errors, http,
    lifecycle::Command,
    protocol::{Connection, TransportDisposition},
    reply, response,
    state::Shared,
    transport::{Socket, Target},
    validated::ValidatedConfig,
};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc,
};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub(crate) struct Context {
    pub config: Arc<ValidatedConfig>,
    pub target: Arc<Target>,
    pub state: Shared,
    pub commands: mpsc::Sender<Command>,
    pub cancel: CancellationToken,
}
impl Context {
    pub(crate) fn new(
        config: ValidatedConfig,
        target: Arc<Target>,
        state: Shared,
        commands: mpsc::Sender<Command>,
        cancel: &CancellationToken,
    ) -> Self {
        Self {
            config: Arc::new(config),
            target,
            state,
            commands,
            cancel: cancel.child_token(),
        }
    }
}
pub(crate) enum Outcome {
    Completed,
    Cancelled,
    Failed(ProxyError),
}

struct PlannedRequest {
    request: http::Request,
    ordinal: u64,
    decision: Decision,
}

pub(crate) async fn run(socket: TcpStream, context: Context) -> Outcome {
    let cancel = context.cancel.child_token();
    let outcome = tokio::select! {
        biased;
        () = cancel.cancelled() => Outcome::Cancelled,
        result = serve(socket, &context) => match result {
            Ok(()) => Outcome::Completed,
            Err(error) => Outcome::Failed(error),
        }
    };
    match context.state.finished() {
        Ok(()) => outcome,
        Err(error) => Outcome::Failed(error),
    }
}

async fn serve(socket: TcpStream, context: &Context) -> Result<(), ProxyError> {
    let mut client = BufReader::new(socket);
    let mut upstream = None;
    for _ in 0..MAX_REQUESTS {
        let planned = match prepare(&mut client, context).await {
            Ok(Some(planned)) => planned,
            Ok(None) => return Ok(()),
            Err(error) => return reply::reject(&mut client, context, error).await,
        };
        if respond(&mut client, &mut upstream, context, planned).await? == Connection::Close {
            return Ok(());
        }
    }
    Err(ProxyError::Limit("connection request count"))
}

async fn prepare(
    client: &mut BufReader<TcpStream>,
    context: &Context,
) -> http::RequestResult<Option<PlannedRequest>> {
    let Some(request) = http::request(client, &context.config).await? else {
        return Ok(None);
    };
    let (ordinal, decision) = match context.state.decide(&context.config, &request.scope) {
        Ok(plan) => plan,
        Err(error) => return Err(http::RequestError::scoped(error, request.scope)),
    };
    Ok(Some(PlannedRequest {
        request,
        ordinal,
        decision,
    }))
}

async fn respond(
    client: &mut BufReader<TcpStream>,
    upstream: &mut Option<BufReader<Socket>>,
    context: &Context,
    planned: PlannedRequest,
) -> Result<Connection, ProxyError> {
    let PlannedRequest {
        request,
        ordinal,
        decision,
    } = planned;
    let transport = transport_fault(client, context, decision.fault).await?;
    if matches!(transport, TransportDisposition::Handled) {
        return Ok(Connection::Close);
    }
    if let Some(response) = errors::canned(decision, context.config.style(&request.scope), ordinal)?
    {
        reply::canned(client, context, &response, &request, decision).await?;
        return Ok(request.connection);
    }
    let close = forward(client, upstream, context, &request, decision).await?;
    Ok(close.combine(request.connection))
}

async fn transport_fault(
    client: &mut BufReader<TcpStream>,
    context: &Context,
    fault: Option<Fault>,
) -> Result<TransportDisposition, ProxyError> {
    match fault {
        Some(Fault::Reset) => {
            client.get_ref().set_zero_linger()?;
            Ok(TransportDisposition::Handled)
        }
        Some(Fault::Stall) => {
            stall(client, context).await?;
            Ok(TransportDisposition::Handled)
        }
        Some(Fault::RefusedConnection) => {
            let action = super::lifecycle::Control::RefuseFor(context.config.refusal_duration());
            super::lifecycle::control(&context.commands, action).await?;
            Ok(TransportDisposition::Handled)
        }
        _ => Ok(TransportDisposition::Forward),
    }
}

async fn forward(
    client: &mut BufReader<TcpStream>,
    upstream: &mut Option<BufReader<Socket>>,
    context: &Context,
    request: &http::Request,
    decision: Decision,
) -> Result<Connection, ProxyError> {
    let response = match fetch_response(upstream, context, request, decision).await {
        Ok(response) => response,
        Err(ProxyError::Framing) => return reply::bad_gateway(client, context, decision).await,
        Err(error) => return Err(error),
    };
    let close = reply::forward(client, context, &response, decision).await?;
    if response.connection == Connection::Close {
        *upstream = None;
    }
    Ok(close)
}

async fn fetch_response(
    upstream: &mut Option<BufReader<Socket>>,
    context: &Context,
    request: &http::Request,
    decision: Decision,
) -> Result<response::Response, ProxyError> {
    if upstream.is_none() {
        *upstream = Some(context.target.connect().await?);
    }
    let socket = upstream.as_mut().ok_or(ProxyError::Stopped)?;
    deadline(socket.get_mut().write_all(&request.bytes)).await?;
    deadline(socket.get_mut().flush()).await?;
    context.state.forwarded()?;
    response::read(socket, request.body_policy, decision).await
}

async fn stall(client: &mut BufReader<TcpStream>, context: &Context) -> Result<(), ProxyError> {
    const STALL_BUFFER: usize = 4_096;
    let mut buffer = [0u8; STALL_BUFFER];
    // EOF/error or outer cancellation bounds this lifetime, never cumulative bytes.
    loop {
        let count = client.read(&mut buffer).await?;
        if count == 0 {
            return Ok(());
        }
        context.state.stalled_bytes(count)?;
    }
}
