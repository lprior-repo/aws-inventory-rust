use crate::{
    candidate::Candidate,
    error::{Error, Result, number},
    sample::Sample,
    sink::Mode,
};
use std::{
    hint::black_box,
    time::{Duration, Instant},
};
pub fn run(c: Candidate, sample: &Sample, size: &str, mode: Mode) -> Result<()> {
    let input = if c.xml() { &sample.xml } else { &sample.json };
    (0..3).try_for_each(|_| {
        c.decode(black_box(input), mode).map(|v| {
            black_box(v);
        })
    })?;
    let mut samples: Vec<f64> = (0..15)
        .map(|_| time(c, input, mode))
        .collect::<Result<_>>()?;
    samples.sort_by(f64::total_cmp);
    let median = *samples.get(7).ok_or(Error::Bound)?;
    let min = *samples.first().ok_or(Error::Bound)?;
    let max = *samples.last().ok_or(Error::Bound)?;
    let mut outcome = Ok(());
    let alloc = allocation_counter::measure(|| {
        outcome = c.decode(black_box(input), mode).map(|s| {
            black_box(s);
        });
    });
    outcome?;
    let items = number(sample.items)?;
    let mib = number(input.len())? / 1_048_576.0;
    println!(
        "| {} | {size} | {mode:?} | {} | {} | {:.2} | {:.2}–{:.2} | {:.0} | {:.0} | {:.2} | {:.2} | {} |",
        c.name(),
        input.len(),
        sample.items,
        mib / median,
        mib / max,
        mib / min,
        items / median,
        median * 1e6,
        f64::from(u32::try_from(alloc.count_total).map_err(|_| Error::Bound)?) / items,
        f64::from(u32::try_from(alloc.bytes_total).map_err(|_| Error::Bound)?) / items,
        alloc.bytes_max
    );
    Ok(())
}
fn time(c: Candidate, input: &[u8], mode: Mode) -> Result<f64> {
    let start = Instant::now();
    let mut calls = 0u32;
    std::iter::from_fn(|| {
        if calls >= 4096 || (calls > 0 && start.elapsed() >= Duration::from_millis(10)) {
            return None;
        }
        calls = calls.saturating_add(1);
        Some(c.decode(black_box(input), mode).map(|value| {
            black_box(value);
        }))
    })
    .try_for_each(|r| r)?;
    Ok(start.elapsed().as_secs_f64() / f64::from(calls))
}
pub fn equivalence(candidates: &[Candidate], sample: &Sample) -> Result<()> {
    let expected = Candidate::Jiter.decode(&sample.json, Mode::Records)?;
    if expected.items != sample.items {
        return Err(Error::Shape("wrong oracle item count"));
    }
    candidates.iter().try_for_each(|c| {
        let bytes = if c.xml() { &sample.xml } else { &sample.json };
        let actual = c.decode(bytes, Mode::Records)?;
        if actual.items != sample.items || actual.records != expected.records {
            return Err(Error::Syntax(format!(
                "{} record output mismatch",
                c.name()
            )));
        }
        Ok(())
    })
}
pub fn profile(candidate: Option<Candidate>) -> Result<()> {
    let candidate = candidate.ok_or(Error::Shape("profile candidate missing"))?;
    let sample = crate::sample::generate(1500, 3000)?;
    let input = if candidate.xml() {
        &sample.xml
    } else {
        &sample.json
    };
    (0..200).try_for_each(|_| {
        candidate
            .decode(black_box(input), Mode::Records)
            .map(|output| {
                black_box(output);
            })
    })
}
