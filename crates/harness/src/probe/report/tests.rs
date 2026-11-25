use super::{Outcome, Totals, render};
use crate::probe::ProbeError;

#[test]
fn given_no_enumerations_when_rendered_then_every_outcome_total_is_zero() -> Result<(), ProbeError>
{
    assert_eq!(
        render(Vec::new())?,
        "total items 0\ntotal empty 0\ntotal unsupported 0\ntotal failed 0\ntotal skipped-child 0\n"
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
