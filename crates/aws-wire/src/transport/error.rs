use std::{error::Error, io};

pub(super) type Source = Box<dyn Error + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("transport deadline elapsed")]
    Timeout,

    #[error("connection failed: {0}")]
    Connect(#[source] Source),

    #[error("TLS failed: {0}")]
    Tls(#[source] Source),

    #[error("response body exceeds {limit} bytes")]
    BodyTooLarge { limit: usize },

    #[error("I/O failed: {0}")]
    Io(#[source] Source),

    #[error("HTTP protocol failed: {0}")]
    Protocol(#[source] Source),

    #[error("invalid request: {0}")]
    InvalidRequest(&'static str),
}

pub(super) fn classify(error: Source) -> TransportError {
    let source: &(dyn Error + 'static) = error.as_ref();
    let chain = std::iter::successors(Some(source), |e| next_source(*e));
    if chain.clone().take(16).any(|source| {
        matches!(
            source.downcast_ref::<TransportError>(),
            Some(TransportError::Timeout)
        )
    }) {
        return TransportError::Timeout;
    }
    let kinds = chain.take(16).fold((false, false, false), inspect);
    match kinds {
        (true, _, _) => TransportError::Tls(error),
        (_, true, _) => TransportError::Connect(error),
        (_, _, true) => TransportError::Io(error),
        _ => TransportError::Protocol(error),
    }
}

fn next_source<'a>(error: &'a (dyn Error + 'static)) -> Option<&'a (dyn Error + 'static)> {
    if let Some(inner) = error
        .downcast_ref::<io::Error>()
        .and_then(io::Error::get_ref)
    {
        return Some(inner);
    }
    if let Some(inner) = error
        .downcast_ref::<h2::Error>()
        .and_then(h2::Error::get_io)
    {
        return Some(inner);
    }
    error.source()
}

fn inspect(state: (bool, bool, bool), error: &(dyn Error + 'static)) -> (bool, bool, bool) {
    let socket = error.downcast_ref::<io::Error>();
    let tls = error.is::<rustls::Error>()
        || socket
            .and_then(io::Error::get_ref)
            .is_some_and(|inner| inner.is::<rustls::Error>());
    let connect = is_connect(error) || socket.is_some_and(is_reset);
    (
        state.0 || tls,
        state.1 || connect,
        state.2 || socket.is_some(),
    )
}

fn is_connect(error: &(dyn Error + 'static)) -> bool {
    error
        .downcast_ref::<hyper_util::client::legacy::Error>()
        .is_some_and(|e| e.is_connect())
        || error
            .downcast_ref::<hyper::Error>()
            .is_some_and(|e| e.is_closed() || e.is_incomplete_message() || e.is_canceled())
        || error.downcast_ref::<h2::Error>().is_some_and(retired_h2)
}

fn retired_h2(error: &h2::Error) -> bool {
    error.is_remote()
        && ((error.is_go_away() && error.reason() == Some(h2::Reason::NO_ERROR))
            || (error.is_reset() && error.reason() == Some(h2::Reason::REFUSED_STREAM)))
}

fn is_reset(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::UnexpectedEof
    )
}
