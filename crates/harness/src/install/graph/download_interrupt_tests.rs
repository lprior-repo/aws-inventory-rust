use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct ScriptedStream {
    interrupts: usize,
    pause: Duration,
    payload: &'static [u8],
    bounds: Vec<Duration>,
    reads: usize,
}

impl ScriptedStream {
    fn new(interrupts: usize, pause: Duration, payload: &'static [u8]) -> Self {
        Self {
            interrupts,
            pause,
            payload,
            bounds: Vec::new(),
            reads: 0,
        }
    }
}

impl Read for ScriptedStream {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.reads = self.reads.saturating_add(1);
        if self.interrupts == 0 {
            return self.payload.read(buffer);
        }
        self.interrupts = self.interrupts.saturating_sub(1);
        std::thread::sleep(self.pause);
        Err(std::io::Error::from(std::io::ErrorKind::Interrupted))
    }
}

impl BoundedRead for ScriptedStream {
    fn bound(&mut self, duration: Duration) -> std::io::Result<()> {
        self.bounds.push(duration);
        Ok(())
    }
}

fn budget(lifetime: Duration) -> DownloadBudget {
    DownloadBudget {
        started: Instant::now(),
        max_bytes: 1_024,
        lifetime,
    }
}

#[test]
fn given_interrupted_reads_when_deadline_input_reads_then_data_arrives_with_shrinking_bounds()
-> TestResult {
    let mut stream = ScriptedStream::new(2, Duration::from_millis(25), b"wanted");
    let budget = budget(Duration::from_secs(30));
    let mut buffer = [0_u8; 16];
    let count = DeadlineInput {
        input: &mut stream,
        budget: &budget,
    }
    .read(&mut buffer)?;
    assert_eq!(buffer.get(..count), Some(&b"wanted"[..]));
    assert_eq!(stream.reads, 3);
    assert_eq!(stream.bounds.len(), 3);
    assert!(stream.bounds.windows(2).all(|pair| match pair {
        [first, second] => second < first,
        _ => false,
    }));
    assert!(
        stream
            .bounds
            .iter()
            .all(|bound| *bound < Duration::from_secs(30))
    );
    Ok(())
}

#[test]
fn given_endless_interrupts_when_deadline_expires_then_read_times_out() -> TestResult {
    let mut stream = ScriptedStream::new(usize::MAX, Duration::from_millis(10), b"never");
    let budget = budget(Duration::from_millis(80));
    let mut buffer = [0_u8; 16];
    let result = DeadlineInput {
        input: &mut stream,
        budget: &budget,
    }
    .read(&mut buffer);
    let error = result.err().ok_or("interrupted reads never succeed")?;
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(stream.reads >= 2);
    assert_eq!(stream.bounds.len(), stream.reads);
    Ok(())
}

#[test]
fn given_interrupted_download_stream_when_body_is_copied_then_digest_matches() -> TestResult {
    let mut stream = ScriptedStream::new(3, Duration::from_millis(1), b"wanted");
    let budget = budget(Duration::from_secs(30));
    let digest = copy_body(
        &mut DeadlineInput {
            input: &mut stream,
            budget: &budget,
        },
        &mut std::io::sink(),
        1_024,
    )?;
    assert_eq!(digest, crate::provenance::sha256_hex(b"wanted"));
    Ok(())
}
