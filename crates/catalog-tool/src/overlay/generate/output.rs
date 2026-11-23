use super::{Failure, service::GeneratedService};
use std::fmt::{self, Write};

#[derive(Default)]
pub(super) struct Report {
    text: Text,
    enumerate: usize,
    callable: usize,
    deferred: usize,
}

impl Report {
    pub(super) fn service(&mut self, name: &str, service: &GeneratedService) -> Result<(), Failure> {
        let deferred = service.reasons.values().try_fold(0_usize, |total, count| {
            total.checked_add(*count).ok_or("reason count overflow")
        })?;
        if service.callable.checked_add(deferred) != Some(service.enumerate) {
            return Err("generated service count mismatch".into());
        }
        add(&mut self.enumerate, service.enumerate)?;
        add(&mut self.callable, service.callable)?;
        add(&mut self.deferred, deferred)?;
        writeln!(self.text, "service {name} callable={} deferred={deferred}", service.callable)?;
        for (reason, count) in &service.reasons {
            writeln!(self.text, "reason {reason} {count}")?;
        }
        Ok(())
    }

    pub(super) fn finish(mut self) -> Result<String, Failure> {
        if self.callable.checked_add(self.deferred) != Some(self.enumerate) {
            return Err("generated total count mismatch".into());
        }
        writeln!(self.text, "totals enumerate_candidates={} callable={} deferred={}",
            self.enumerate, self.callable, self.deferred)?;
        Ok(self.text.0)
    }
}

#[derive(Default)]
struct Text(String);

impl Write for Text {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let length = self.0.len().checked_add(value.len()).ok_or(fmt::Error)?;
        if length > 4_194_304 {
            return Err(fmt::Error);
        }
        self.0.try_reserve(value.len()).map_err(|_| fmt::Error)?;
        self.0.push_str(value);
        Ok(())
    }
}

fn add(value: &mut usize, count: usize) -> Result<(), Failure> {
    *value = value.checked_add(count).ok_or("generated count overflow")?;
    Ok(())
}
