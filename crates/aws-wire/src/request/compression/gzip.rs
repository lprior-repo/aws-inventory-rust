use crate::request::{EncodeError, common::BoundedBody};
use flate2::{Compression, GzBuilder};
use std::io::{self, Write};

struct BoundedSink<'a> {
    body: BoundedBody,
    failure: &'a mut Option<EncodeError>,
}

impl Write for BoundedSink<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self.body.write(bytes) {
            Ok(()) => Ok(bytes.len()),
            Err(error) => {
                *self.failure = Some(error);
                Err(io::Error::other("bounded gzip output failed"))
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) fn encode(body: &[u8], limit: usize) -> Result<Vec<u8>, EncodeError> {
    if body.len() > limit {
        return Err(EncodeError::Limit("body"));
    }
    let mut failure = None;
    let sink = BoundedSink {
        body: BoundedBody::new(limit),
        failure: &mut failure,
    };
    let mut encoder = GzBuilder::new()
        .mtime(0)
        .operating_system(255)
        .write(sink, Compression::default());
    if let Err(error) = encoder.write_all(body) {
        drop(encoder);
        return Err(match failure.take() {
            Some(error) => error,
            None => EncodeError::Compression(error),
        });
    }
    if let Err(error) = encoder.try_finish() {
        drop(encoder);
        return Err(match failure.take() {
            Some(error) => error,
            None => EncodeError::Compression(error),
        });
    }
    let BoundedSink {
        body: compressed,
        failure: _,
    } = encoder.finish().map_err(EncodeError::Compression)?;
    match failure.take() {
        Some(error) => Err(error),
        None => Ok(compressed.into_vec()),
    }
}
