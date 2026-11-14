use super::{
    TestError, TestResult,
    wire::{self, Io, SendResult},
};
use std::pin::Pin;
use tokio::io::AsyncReadExt;

pub async fn request_headers<F>(send: &mut Pin<Box<F>>, stream: &mut Io) -> TestResult
where
    F: Future<Output = SendResult>,
{
    let mut preface = [0; 24];
    wire::until(send, stream.read_exact(&mut preface)).await??;
    assert_eq!(&preface, b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");
    wire::write(stream, b"\0\0\0\x04\0\0\0\0\0").await?;
    for _ in 0..32 {
        let (kind, id) = wire::until(send, read(stream)).await??;
        if kind == 1 {
            assert_eq!(id, 1);
            return Ok(());
        }
    }
    Err("client did not send HTTP/2 request HEADERS".into())
}

async fn read(stream: &mut Io) -> Result<(u8, u32), TestError> {
    let mut header = [0; 9];
    stream.read_exact(&mut header).await?;
    let [a, b, c, kind, _flags, d, e, f, g] = header;
    let length = usize::try_from(u32::from_be_bytes([0, a, b, c]))?;
    if length > 16_384 {
        return Err("fixture frame exceeds bound".into());
    }
    let mut payload = vec![0; length];
    stream.read_exact(&mut payload).await?;
    Ok((kind, u32::from_be_bytes([d, e, f, g]) & 0x7fff_ffff))
}
