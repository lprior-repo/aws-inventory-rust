use super::{Outcome, Row, Totals, render};
use crate::probe::ProbeError;
use inventory_core::catalog::runtime::RuntimeProtocol;

#[test]
fn given_no_enumerations_when_rendered_then_every_outcome_total_is_zero() -> Result<(), ProbeError>
{
    assert_eq!(
        render(Vec::new())?,
        "total items 0\ntotal empty 0\ntotal not-applicable 0\ntotal unsupported 0\ntotal failed 0\ntotal skipped-child 0\n"
    );
    Ok(())
}

#[test]
fn given_maximum_outcome_total_when_incremented_then_overflow_is_reported() {
    let mut totals = Totals {
        items: u64::MAX,
        ..Totals::default()
    };
    assert!(matches!(
        totals.add(&Outcome::Items(1)),
        Err(ProbeError::TotalOverflow)
    ));
    assert_eq!(totals.items, u64::MAX);
}

#[test]
fn given_skipped_children_with_and_without_detail_when_rendered_then_both_count()
-> Result<(), ProbeError> {
    let rows = vec![
        Row {
            service: "fixture",
            operation: "ListTags",
            protocol: RuntimeProtocol::RestJson1,
            outcome: Outcome::SkippedChild {
                reason: "tag endpoint uses dedicated resource tag binding",
                detail: None,
            },
        },
        Row {
            service: "fixture",
            operation: "ListChildren",
            protocol: RuntimeProtocol::RestJson1,
            outcome: Outcome::SkippedChild {
                reason: "no sampled parent item satisfies all bindings and row eligibility",
                detail: Some("no caller identity".into()),
            },
        },
    ];
    assert_eq!(
        render(rows)?,
        concat!(
            "fixture ListChildren restJson1 skipped child no sampled parent item satisfies all bindings and row eligibility: no caller identity\n",
            "fixture ListTags restJson1 skipped child tag endpoint uses dedicated resource tag binding\n",
            "total items 0\ntotal empty 0\ntotal not-applicable 0\ntotal unsupported 0\ntotal failed 0\ntotal skipped-child 2\n",
            "protocol restJson1 items 0 empty 0 not-applicable 0 unsupported 0 failed 0 skipped-child 2\n",
        )
    );
    Ok(())
}

#[test]
fn given_not_applicable_outcome_when_rendered_then_it_has_an_independent_total()
-> Result<(), ProbeError> {
    let rows = vec![Row {
        service: "sesv2",
        operation: "ListImportJobs",
        protocol: RuntimeProtocol::RestJson1,
        outcome: Outcome::NotApplicable("account is in the sandbox".into()),
    }];
    assert_eq!(
        render(rows)?,
        concat!(
            "sesv2 ListImportJobs restJson1 not-applicable account is in the sandbox\n",
            "total items 0\ntotal empty 0\ntotal not-applicable 1\ntotal unsupported 0\ntotal failed 0\ntotal skipped-child 0\n",
            "protocol restJson1 items 0 empty 0 not-applicable 1 unsupported 0 failed 0 skipped-child 0\n",
        )
    );
    Ok(())
}
