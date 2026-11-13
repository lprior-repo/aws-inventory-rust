//! Buffered event writer that injects duplicate and adjacent-reorder defects.
//!
//! Duplicates are emitted as an adjacent pair so the oracle verifier can recognise
//! and discard them without retaining every resource id. With `--reorder-pct p`, a
//! `p` percent share of events is held back and written after the following event
//! (one adjacent swap per held event); units keep their internal adjacency, so
//! duplicate lines stay adjacent.

use std::error::Error;
use std::io::Write;

use inventory_core::CollectionEvent;

use crate::options::Options;
use crate::rng::Rng;

const MUTATION_SALT: u64 = 0x2545_F491_4F6C_DD1D;

pub struct EventSink<W: Write> {
    writer: W,
    mutation: Rng,
    duplicate_pct: u64,
    reorder_pct: u64,
    pending: Option<String>,
    pub duplicates: usize,
    pub reorders: usize,
}

impl<W: Write> EventSink<W> {
    pub fn new(writer: W, options: &Options) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            writer,
            mutation: Rng::new(options.seed ^ MUTATION_SALT),
            duplicate_pct: options.duplicate_pct,
            reorder_pct: options.reorder_pct,
            pending: None,
            duplicates: 0,
            reorders: 0,
        })
    }

    pub fn emit(&mut self, event: &CollectionEvent) -> Result<(), Box<dyn Error>> {
        let line = serde_json::to_string(event)?;
        let duplicate = self.mutation.below(100) < self.duplicate_pct;
        let mut unit = line;
        if duplicate {
            self.duplicates = self.duplicates.checked_add(1).ok_or("duplicate overflow")?;
            let copy = unit.clone();
            unit.push('\n');
            unit.push_str(&copy);
        }
        unit.push('\n');

        let hold = self.mutation.below(100) < self.reorder_pct;
        if let Some(held) = self.pending.take() {
            self.write_unit(&unit)?;
            self.write_unit(&held)?;
            self.reorders = self.reorders.checked_add(1).ok_or("reorder overflow")?;
        } else if hold {
            self.pending = Some(unit);
        } else {
            self.write_unit(&unit)?;
        }
        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(held) = self.pending.take() {
            self.write_unit(&held)?;
        }
        self.writer.flush()?;
        Ok(())
    }

    fn write_unit(&mut self, unit: &str) -> Result<(), Box<dyn Error>> {
        self.writer.write_all(unit.as_bytes())?;
        Ok(())
    }
}
