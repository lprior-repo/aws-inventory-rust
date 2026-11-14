#![forbid(unsafe_code)]
mod adversarial;
mod candidate;
mod entities;
mod error;
mod json_event;
mod json_jiter;
mod json_minimal;
mod json_serde;
mod json_smithy;
mod json_struson;
mod measure;
mod metadata;
mod minimal_lex;
mod sample;
mod scalar;
mod scenarios;
mod shape;
mod sink;
mod xml_bridge;
mod xml_events;
mod xml_smithy;
use candidate::{ALL, Candidate};
use error::Result;
use sink::Mode;
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let candidates: Vec<Candidate> = ALL
        .iter()
        .copied()
        .filter(|c| {
            args.first()
                .is_none_or(|a| a.starts_with("--") || a == c.name())
        })
        .collect();
    if args.iter().any(|a| a == "--metadata") {
        return metadata::run();
    }
    if args.iter().any(|a| a == "--adversarial") {
        return adversarial::run(&candidates);
    }
    if args.iter().any(|a| a == "--scenarios") {
        return scenarios::run();
    }
    if args.iter().any(|a| a == "--profile") {
        return measure::profile(candidates.first().copied());
    }
    scenarios::run()?;
    println!(
        "# Decoder benchmark\n15 samples × ≥10 ms (or 4096 calls), 3 warmups; min–max spread. Allocations include item Value tree, record encoder and retained page output. Peak is incremental live heap, input excluded.\n"
    );
    println!(
        "| Candidate | Size | Mode | Input bytes | Items | MiB/s median | MiB/s min–max | Items/s | Median µs/page | Allocs/item | Allocated B/item | Peak live heap B |\n|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    );
    [
        ("small", 2, 450),
        ("medium", 100, 1550),
        ("large", 1500, 3000),
    ]
    .into_iter()
    .try_for_each(|(size, items, padding)| {
        let sample = sample::generate(items, padding)?;
        measure::equivalence(&candidates, &sample)?;
        candidates.iter().try_for_each(|&c| {
            [Mode::Tokens, Mode::Records]
                .into_iter()
                .try_for_each(|mode| measure::run(c, &sample, size, mode))
        })
    })?;
    adversarial::run(&candidates)?;
    metadata::run()
}
