//! Frame-time recording for performance profiling (M6.3).
//!
//! When `CMDB_UI_FRAME_LOG` names a file, every drawn frame appends one line
//! `frame_index,elapsed_ms,node_count` to it. The file is truncated at start so a
//! run's trace is self-contained. A write failure disables recording and is
//! reported once on stderr; it never interrupts the UI.

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

pub struct FrameTimeRecorder {
    output: Option<File>,
    last_frame: Instant,
    frames: u64,
}

impl FrameTimeRecorder {
    /// Reads `CMDB_UI_FRAME_LOG`; recording is off when it is unset.
    pub fn from_env() -> Result<Self, String> {
        match std::env::var_os("CMDB_UI_FRAME_LOG") {
            Some(path) => Self::to_path(Path::new(&path)),
            None => Ok(Self::disabled()),
        }
    }

    pub fn disabled() -> Self {
        Self::with_output(None)
    }

    pub fn to_path(path: &Path) -> Result<Self, String> {
        let file =
            File::create(path).map_err(|error| format!("frame log {}: {error}", path.display()))?;
        Ok(Self::with_output(Some(file)))
    }

    fn with_output(output: Option<File>) -> Self {
        Self {
            output,
            last_frame: Instant::now(),
            frames: 0,
        }
    }

    /// Records one frame that drew `node_count` nodes; returns the elapsed milliseconds
    /// since the previous frame.
    pub fn frame(&mut self, node_count: usize) -> u64 {
        let elapsed = u64::try_from(self.last_frame.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.last_frame = Instant::now();
        self.frames = self.frames.saturating_add(1);
        if let Some(file) = self.output.as_mut() {
            let line = format!("{},{elapsed},{node_count}\n", self.frames);
            if let Err(error) = file.write_all(line.as_bytes()) {
                eprintln!("frame log write failed, recording stopped: {error}");
                self.output = None;
            }
        }
        elapsed
    }

    pub fn shutdown(&mut self) {
        if let Some(mut file) = self.output.take() {
            if let Err(error) = file.flush() {
                eprintln!("frame log flush failed: {error}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn given_log_path_when_frames_recorded_then_each_line_has_index_elapsed_and_nodes() -> TestResult
    {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("frames.csv");
        let mut recorder = FrameTimeRecorder::to_path(&path)?;
        recorder.frame(10);
        recorder.frame(25);
        recorder.shutdown();
        let content = std::fs::read_to_string(&path)?;
        let rows: Vec<Vec<&str>> = content
            .lines()
            .map(|line| line.split(',').collect())
            .collect();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows.iter().map(|row| row.len()).collect::<Vec<_>>(), [3, 3]);
        assert_eq!(
            rows.iter().map(|row| row[0]).collect::<Vec<_>>(),
            ["1", "2"]
        );
        assert_eq!(
            rows.iter().map(|row| row[2]).collect::<Vec<_>>(),
            ["10", "25"]
        );
        assert!(rows.iter().all(|row| row[1].parse::<u64>().is_ok()));
        Ok(())
    }

    #[test]
    fn given_existing_log_when_recorder_starts_then_old_trace_is_discarded() -> TestResult {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("frames.csv");
        std::fs::write(&path, "stale\n")?;
        let mut recorder = FrameTimeRecorder::to_path(&path)?;
        recorder.frame(1);
        recorder.shutdown();
        assert!(!std::fs::read_to_string(&path)?.contains("stale"));
        Ok(())
    }

    #[test]
    fn given_unwritable_log_path_when_opened_then_error_names_the_path() {
        let error = FrameTimeRecorder::to_path(Path::new("/nonexistent-dir/frames.csv"))
            .err()
            .unwrap_or_default();
        assert!(error.contains("/nonexistent-dir/frames.csv"), "{error}");
    }

    #[test]
    fn given_disabled_recorder_when_frame_recorded_then_nothing_is_written_and_time_is_returned() {
        let mut recorder = FrameTimeRecorder::disabled();
        let _elapsed = recorder.frame(5);
        assert!(recorder.output.is_none());
    }
}
