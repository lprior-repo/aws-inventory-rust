use super::*;

#[derive(Clone, Copy)]
pub(super) enum Fault {
    DropAt(u64),
    ErrorAt(u64, &'static str),
    AlwaysRetry,
    Hang,
}

pub(super) struct State {
    pub(super) queries: AtomicU64,
    pub(super) connections: AtomicU64,
    pub(super) committed_before_drop: AtomicU64,
    pub(super) observed: Mutex<Vec<String>>,
    pub(super) query_started: Notify,
    pub(super) fault: Fault,
}

pub(super) struct Endpoint {
    pub(super) url: String,
    pub(super) state: Arc<State>,
    pub(super) stop: CancellationToken,
    pub(super) task: Option<JoinHandle<io::Result<()>>>,
}

impl Endpoint {
    pub(super) async fn start(fault: Fault, backend: Option<String>) -> Result<Self, io::Error> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let url = format!("redis://{}", listener.local_addr()?);
        let state = Arc::new(State {
            queries: AtomicU64::new(0),
            connections: AtomicU64::new(0),
            committed_before_drop: AtomicU64::new(0),
            observed: Mutex::new(Vec::new()),
            query_started: Notify::new(),
            fault,
        });
        let stop = CancellationToken::new();
        let owner = stop.clone();
        let shared = Arc::clone(&state);
        let task = tokio::spawn(async move {
            let mut clients = JoinSet::new();
            for _ in 0..64 {
                let accepted = tokio::select! {
                    biased;
                    () = owner.cancelled() => break,
                    result = listener.accept() => result,
                };
                let (socket, _) = accepted?;
                shared.connections.fetch_add(1, Ordering::Relaxed);
                let state = Arc::clone(&shared);
                let cancellation = owner.clone();
                let target = backend.clone();
                clients.spawn(async move {
                    tokio::select! {
                        biased;
                        () = cancellation.cancelled() => Ok(()),
                        result = serve(socket, state, target) => result,
                    }
                });
            }
            clients.abort_all();
            while let Some(result) = clients.join_next().await {
                match result {
                    Ok(Ok(())) => {}
                    Ok(Err(error))
                        if matches!(
                            error.kind(),
                            io::ErrorKind::BrokenPipe
                                | io::ErrorKind::UnexpectedEof
                                | io::ErrorKind::ConnectionReset
                        ) => {}
                    Err(error) if error.is_cancelled() => {}
                    Ok(Err(error)) => return Err(error),
                    Err(error) => return Err(io::Error::other(error)),
                }
            }
            Ok(())
        });
        Ok(Self {
            url,
            state,
            stop,
            task: Some(task),
        })
    }

    pub(super) async fn query_started(&self) -> TestResult {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let notified = self.state.query_started.notified();
                if self.state.queries.load(Ordering::Relaxed) > 0 {
                    break;
                }
                notified.await;
            }
        })
        .await
        .map_err(|_| "graph retry fixture query start exceeded its 5-second deadline")?;
        Ok(())
    }

    pub(super) async fn close(mut self) -> TestResult {
        self.stop.cancel();
        if let Some(task) = self.task.as_mut() {
            tokio::time::timeout(Duration::from_secs(5), task)
                .await
                .map_err(|_| "graph retry fixture shutdown exceeded its 5-second deadline")???;
        }
        Ok(())
    }
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        self.stop.cancel();
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}
pub(super) async fn serve(
    socket: TcpStream,
    state: Arc<State>,
    backend: Option<String>,
) -> io::Result<()> {
    let mut upstream = if let Some(url) = backend {
        Some(
            redis::Client::open(url)
                .map_err(io::Error::other)?
                .get_multiplexed_async_connection()
                .await
                .map_err(io::Error::other)?,
        )
    } else {
        None
    };
    let mut reader = BufReader::new(socket);
    for _ in 0..128 {
        let Some(arguments) = command(&mut reader).await? else {
            return Ok(());
        };
        if arguments.first().map(Vec::as_slice) != Some(inventory_graph::cypher::COMMAND.as_bytes())
        {
            reader.get_mut().write_all(b"+OK\r\n").await?;
            continue;
        }
        let number = state
            .queries
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        let query = arguments
            .get(2)
            .ok_or_else(|| io::Error::other("missing query"))?;
        {
            let mut observed = state
                .observed
                .lock()
                .map_err(|_| io::Error::other("poisoned test state"))?;
            if observed.len() >= 64 {
                return Err(io::Error::other("test query count bound"));
            }
            observed.push(String::from_utf8(query.clone()).map_err(io::Error::other)?);
        }
        state.query_started.notify_waiters();
        let mut response = Vec::new();
        if let Some(connection) = &mut upstream {
            let mut command = redis::cmd(inventory_graph::cypher::COMMAND);
            for argument in arguments.iter().skip(1) {
                command.arg(argument);
            }
            let result: redis::RedisResult<redis::Value> = command.query_async(connection).await;
            match result {
                Ok(value) => {
                    encode(&value, &mut response)?;
                    if matches!(state.fault, Fault::DropAt(at) if at == number) {
                        state.committed_before_drop.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Err(error) => response.extend_from_slice(
                    format!(
                        "-{} {}\r\n",
                        error.code().unwrap_or("ERR"),
                        error.detail().unwrap_or("rejected")
                    )
                    .as_bytes(),
                ),
            }
        } else {
            response.extend_from_slice(b"+OK\r\n");
        }
        match state.fault {
            Fault::DropAt(at) if at == number => return Ok(()),
            Fault::ErrorAt(at, error) if at == number => {
                response = format!("-{error}\r\n").into_bytes()
            }
            Fault::AlwaysRetry => response = b"-TRYAGAIN temporary\r\n".to_vec(),
            Fault::Hang => std::future::pending::<()>().await,
            _ => {}
        }
        reader.get_mut().write_all(&response).await?;
    }
    Err(io::Error::other("test command-count bound"))
}
