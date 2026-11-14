use crate::{
    candidate::Candidate,
    error::{Error, Result},
    sample,
    sink::Mode,
};
use std::{
    hint::black_box,
    panic::{AssertUnwindSafe, catch_unwind},
    time::Instant,
};
#[derive(Default)]
struct Counts {
    accepted: usize,
    errors: usize,
    panics: usize,
    slow: usize,
}
fn observe(c: Candidate, input: &[u8], counts: &mut Counts) {
    let start = Instant::now();
    match catch_unwind(AssertUnwindSafe(|| {
        c.decode(black_box(input), Mode::Records)
    })) {
        Ok(Ok(_)) => counts.accepted = counts.accepted.saturating_add(1),
        Ok(Err(_)) => counts.errors = counts.errors.saturating_add(1),
        Err(_) => counts.panics = counts.panics.saturating_add(1),
    }
    if start.elapsed().as_secs() >= 2 {
        counts.slow = counts.slow.saturating_add(1);
    }
}
pub fn run(candidates: &[Candidate]) -> Result<()> {
    println!(
        "\n| Candidate | Case | Accepted | Typed errors | Panics | ≥2s |\n|---|---|---:|---:|---:|---:|"
    );
    let small = sample::generate(2, 450)?;
    candidates.iter().try_for_each(|&c| {
        let input = if c.xml() { &small.xml } else { &small.json };
        let mut counts = Counts::default();
        (0..input.len()).try_for_each(|offset| {
            observe(c, input.get(..offset).ok_or(Error::Bound)?, &mut counts);
            Ok::<_, Error>(())
        })?;
        row(c, "every truncated prefix", &counts);
        if matches!(c, Candidate::Jiter | Candidate::Quick)
            && (counts.accepted != 0 || counts.panics != 0 || counts.slow != 0)
        {
            return Err(Error::Shape("recommended candidate failed truncation gate"));
        }
        cases(c.xml())
            .iter()
            .try_for_each(|(name, bytes)| check_case(c, name, bytes))?;
        Ok(())
    })
}
fn row(c: Candidate, case: &str, n: &Counts) {
    println!(
        "| {} | {case} | {} | {} | {} | {} |",
        c.name(),
        n.accepted,
        n.errors,
        n.panics,
        n.slow
    );
}
fn check_case(c: Candidate, name: &str, bytes: &[u8]) -> Result<()> {
    let mut counts = Counts::default();
    observe(c, bytes, &mut counts);
    row(c, name, &counts);
    let accept = matches!(
        name,
        "1 MiB string" | "unknown members" | "depth 64 skipped document"
    );
    if matches!(c, Candidate::Jiter | Candidate::Quick)
        && (counts.panics != 0 || counts.slow != 0 || (counts.accepted == 1) != accept)
    {
        return Err(Error::Shape(
            "recommended candidate failed hostile-input gate",
        ));
    }
    Ok(())
}
fn cases(xml: bool) -> Vec<(&'static str, Vec<u8>)> {
    let wrap = |body: &str| {
        if xml {
            format!(
                "<DescribeInstancesResponse><reservationSet><item><instancesSet><item>{body}</item></instancesSet></item></reservationSet></DescribeInstancesResponse>"
            )
        } else {
            format!("{{\"Items\":[{{{body}}}]}}")
        }
    };
    let description = |s: &str| {
        if xml {
            format!("<description>{s}</description>")
        } else {
            format!("\"Description\":\"{s}\"")
        }
    };
    let deep = if xml {
        format!(
            "<Unknown>{}x{}</Unknown>",
            "<n>".repeat(64),
            "</n>".repeat(64)
        )
    } else {
        format!("\"Unknown\":{}0{}", "[".repeat(64), "]".repeat(64))
    };
    let duplicate = if xml {
        "<instanceId>a</instanceId><instanceId>b</instanceId>"
    } else {
        "\"InstanceId\":\"a\",\"InstanceId\":\"b\""
    };
    let unknown = if xml {
        "<Unknown><n>&amp;é</n></Unknown><instanceId>ok</instanceId>"
    } else {
        "\"Unknown\":{\"n\":[null,true,1,\"é\"]},\"InstanceId\":\"ok\""
    };
    let mut invalid = wrap(&description("x")).into_bytes();
    if let Some(byte) = invalid.iter_mut().find(|b| **b == b'x') {
        *byte = 255;
    }
    let bad = if xml {
        "<description>unterminated"
    } else {
        "{\"Items\":[{\"Description\":\"unterminated"
    };
    vec![
        ("invalid UTF-8", invalid),
        ("unterminated string/tag", bad.as_bytes().to_vec()),
        (
            "1 MiB string",
            wrap(&description(&"x".repeat(1_048_576))).into_bytes(),
        ),
        ("duplicate known key", wrap(duplicate).into_bytes()),
        ("unknown members", wrap(unknown).into_bytes()),
        ("depth 64 skipped document", wrap(&deep).into_bytes()),
        (
            "mismatched/trailing syntax",
            if xml {
                b"<DescribeInstancesResponse></wrong>".to_vec()
            } else {
                b"{\"Items\":[]} false".to_vec()
            },
        ),
    ]
}
