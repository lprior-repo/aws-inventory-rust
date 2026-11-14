use catalog_tool::overlay::{Identity, Paginator, Rule};
pub mod overlay_support;
use overlay_support::{TestResult, name, parse, reject};

fn strings(n: usize) -> String {
    (0..n)
        .map(|i| format!("'A{i}'"))
        .collect::<Vec<_>>()
        .join(",")
}
#[test]
fn all_sixty_four_entry_arrays_accept_the_bound_and_reject_sixty_five() -> TestResult {
    ["exclude_from_hash", "skip", "queryable", "fixed.Values"]
        .into_iter()
        .try_for_each(|key| -> TestResult {
            [0, 1, 64].into_iter().try_for_each(|n| -> TestResult {
                let o = parse(&format!("[operation.ListThings]\n{key}=[{}]", strings(n)))?;
                let op = o
                    .operations()
                    .get(&name("ListThings")?)
                    .ok_or("missing operation")?;
                let count = match key {
                    "exclude_from_hash" => op.exclude_from_hash.len(),
                    "skip" => op.skip.len(),
                    "queryable" => op.queryable.len(),
                    _ => match op.fixed.values().next() {
                        Some(catalog_tool::overlay::FixedValue::Strings(v)) => v.len(),
                        _ => return Err("missing strings".into()),
                    },
                };
                assert_eq!(count, n);
                Ok(())
            })?;
            reject(
                &format!("{key}=[{}]", strings(65)),
                key,
                "65",
                Rule::Count { min: 0, max: 64 },
            );
            reject(&format!("{key}=['Id','Id']"), key, "Id", Rule::Duplicate);
            Ok(())
        })
}
#[test]
fn identity_keys_accept_one_and_eight_and_reject_nine() -> TestResult {
    [1, 8].into_iter().try_for_each(|n| -> TestResult {
        let o = parse(&format!(
            "[operation.ListThings.identity]\nkey=[{}]",
            strings(n)
        ))?;
        let count = o
            .operations()
            .get(&name("ListThings")?)
            .and_then(|op| op.identity.as_ref())
            .map(|i| match i {
                Identity::Key(v) => v.len(),
                _ => 0,
            });
        assert_eq!(count, Some(n));
        Ok(())
    })?;
    reject(
        &format!("identity={{key=[{}]}}", strings(9)),
        "identity.key",
        "9",
        Rule::Count { min: 1, max: 8 },
    );
    Ok(())
}
fn pairs(n: usize) -> String {
    (0..n)
        .map(|i| format!("{{input='A{i}',output='B{i}'}}"))
        .collect::<Vec<_>>()
        .join(",")
}
#[test]
fn token_pairs_accept_one_and_eight_and_reject_nine_and_duplicates() -> TestResult {
    [1, 8].into_iter().try_for_each(|n| -> TestResult {
        let o = parse(&format!(
            "[operation.ListThings.paginator]\nkind='tokens'\npairs=[{}]",
            pairs(n)
        ))?;
        let p = o
            .operations()
            .get(&name("ListThings")?)
            .and_then(|op| op.paginator.as_ref());
        match p {
            Some(Paginator::Tokens { pairs, truncated }) => {
                assert_eq!((pairs.len(), truncated), (n, &None))
            }
            _ => return Err("wrong paginator".into()),
        }
        Ok(())
    })?;
    reject(
        &format!("paginator={{kind='tokens',pairs=[{}]}}", pairs(9)),
        "paginator.pairs",
        "9",
        Rule::Count { min: 1, max: 8 },
    );
    reject(
        "paginator={kind='tokens',pairs=[{input='A',output='B'},{input='A',output='B'}]}",
        "paginator.pairs",
        "table",
        Rule::Duplicate,
    );
    Ok(())
}
