use cmdb_ui::hierarchy::{HierarchyInput, MAX_NODES, layout};
use std::time::Instant;

const USAGE: &str = "usage: cmdb-bench layout <target-nodes:1..4000000>";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LayoutArgs {
    target: usize,
}

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> Result<LayoutArgs, String> {
    let text = args.next().ok_or_else(|| USAGE.to_owned())?;
    let target = text.parse::<usize>().map_err(|_| USAGE.to_owned())?;
    if target == 0 || target > MAX_NODES || args.next().is_some() {
        return Err(USAGE.to_owned());
    }
    Ok(LayoutArgs { target })
}

pub(crate) fn run(args: &LayoutArgs) -> Result<(), String> {
    let fixture = fixture(args.target)?;
    let input = HierarchyInput {
        kinds: &fixture.kinds,
        node_account: &fixture.accounts,
        node_region: &fixture.regions,
        node_kind: &fixture.node_kinds,
    };
    println!(
        "synthetic seed=none nodes={} accounts=4 regions=12 services=2 kinds=3 account=(i/36)%4 region=(i/3)%12 kind=i%3",
        args.target
    );
    let started = Instant::now();
    let result = layout(input).map_err(|error| error.to_string())?;
    let elapsed = started.elapsed();
    println!(
        "p13 nodes={} groups={} seconds={:.6}",
        result.node_xy.len(),
        result.groups.len(),
        elapsed.as_secs_f64()
    );
    Ok(())
}

struct Fixture {
    kinds: [Box<str>; 3],
    accounts: Vec<u16>,
    regions: Vec<u16>,
    node_kinds: Vec<u32>,
}

fn fixture(target: usize) -> Result<Fixture, String> {
    let mut accounts = buffer(target)?;
    let mut regions = buffer(target)?;
    let mut node_kinds = buffer(target)?;
    (0..target).try_for_each(|i| -> Result<(), String> {
        accounts.push(u16::try_from((i / 36) % 4).map_err(|e| e.to_string())?);
        regions.push(u16::try_from((i / 3) % 12).map_err(|e| e.to_string())?);
        node_kinds.push(u32::try_from(i % 3).map_err(|e| e.to_string())?);
        Ok(())
    })?;
    Ok(Fixture {
        kinds: [
            Box::from("AWS::EC2::VPC"),
            Box::from("AWS::EC2::Subnet"),
            Box::from("AWS::S3::Bucket"),
        ],
        accounts,
        regions,
        node_kinds,
    })
}

fn buffer<T>(capacity: usize) -> Result<Vec<T>, String> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|error| error.to_string())?;
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::{LayoutArgs, MAX_NODES, USAGE, fixture, parse};
    use cmdb_ui::hierarchy::{HierarchyInput, layout};

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn hierarchy_benchmark_target() -> TestResult {
        assert_eq!(
            parse(&mut ["250000".to_owned()].into_iter())?,
            LayoutArgs { target: 250_000 }
        );
        assert_eq!(
            parse(&mut [MAX_NODES.to_string()].into_iter())?,
            LayoutArgs { target: MAX_NODES }
        );
        Ok(())
    }

    #[test]
    fn hierarchy_benchmark_rejects_invalid_targets() -> TestResult {
        let cases = [
            vec![],
            vec!["0"],
            vec!["4000001"],
            vec!["no"],
            vec!["1", "extra"],
        ];
        cases.into_iter().try_for_each(|case| -> TestResult {
            let error = parse(&mut case.into_iter().map(str::to_owned))
                .err()
                .ok_or("expected an error")?;
            assert_eq!(error, USAGE);
            Ok(())
        })
    }

    #[test]
    fn hierarchy_benchmark_uses_the_public_layout() -> TestResult {
        let fixture = fixture(144)?;
        let result = layout(HierarchyInput {
            kinds: &fixture.kinds,
            node_account: &fixture.accounts,
            node_region: &fixture.regions,
            node_kind: &fixture.node_kinds,
        })?;
        assert_eq!(result.node_xy.len(), 144);
        assert_eq!(result.groups.len(), 436);
        assert_eq!(
            result
                .groups
                .iter()
                .filter(|g| g.level == 0)
                .map(|g| g.node_count)
                .collect::<Vec<_>>(),
            vec![36; 4]
        );
        Ok(())
    }
}
