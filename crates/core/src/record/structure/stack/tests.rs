use super::*;
use crate::record::structure::Reachability;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn bounded_stack_borrows_the_callers_inline_array() -> TestResult {
    // Given stack storage, bounded construction retains that array instead of owning heap storage.
    let mut storage = [None; MAX_DEPTH];
    let pointer = storage.as_ptr();
    let frames = Frames::inline(&mut storage, MAX_DEPTH)?;
    let Frames::Inline { slots, .. } = frames else {
        return Err("bounded stack selected dynamic storage".into());
    };
    assert!(std::ptr::eq(pointer, slots.as_ptr()));
    Ok(())
}

#[test]
fn inline_stack_accepts_32_frames_rejects_33_and_preserves_lifo() -> TestResult {
    // Given exactly 32 slots, pushes are inclusive at the bound and pops reverse their order.
    let mut storage = [None; MAX_DEPTH];
    let mut frames = Frames::inline(&mut storage, MAX_DEPTH)?;
    let count = u64::try_from(MAX_DEPTH)?;
    (0..count).try_for_each(|n| frames.push(Frame::Items(n, Reachability::Opaque)))?;
    assert_eq!(frames.push(Frame::Array), Err(RecordError::DepthLimit));
    for expected in (0..count).rev() {
        assert!(matches!(frames.pop(), Some(Frame::Items(found, _)) if found == expected));
    }
    assert!(frames.pop().is_none());
    assert!(frames.last().is_none());
    assert!(frames.last_mut().is_none());
    Ok(())
}

#[test]
fn inline_stack_updates_and_reuses_its_top_slot() -> TestResult {
    // Given a one-slot stack, mutation and pop leave the slot available for another push.
    let mut storage = [None; MAX_DEPTH];
    let mut frames = Frames::inline(&mut storage, 1)?;
    frames.push(Frame::Array)?;
    *frames.last_mut().ok_or("top")? = Frame::IndefMapKey;
    assert!(matches!(frames.last(), Some(Frame::IndefMapKey)));
    assert!(matches!(frames.pop(), Some(Frame::IndefMapKey)));
    frames.push(Frame::Array)?;
    assert_eq!(frames.push(Frame::Array), Err(RecordError::DepthLimit));
    Ok(())
}

#[test]
fn inline_stack_rejects_capacity_above_32_and_pushes_past_zero() -> TestResult {
    // Given unsupported capacity or no permitted slots, no push can reach outside the array.
    let mut storage = [None; MAX_DEPTH];
    assert!(matches!(
        Frames::inline(&mut storage, 33),
        Err(RecordError::DepthLimit)
    ));
    let mut frames = Frames::inline(&mut storage, 0)?;
    assert_eq!(frames.push(Frame::Array), Err(RecordError::DepthLimit));
    assert!(frames.pop().is_none());
    Ok(())
}

#[test]
fn generic_stack_remains_dynamic_and_accepts_more_than_32_frames() -> TestResult {
    // Given the generic path, deep unknown values retain their previous dynamically sized stack.
    let mut frames = Frames::dynamic();
    assert!(matches!(&frames, Frames::Dynamic(_)));
    (0..64).try_for_each(|_| frames.push(Frame::Array))?;
    assert!(matches!(frames.last(), Some(Frame::Array)));
    *frames.last_mut().ok_or("top")? = Frame::IndefMapKey;
    assert!(matches!(frames.pop(), Some(Frame::IndefMapKey)));
    Ok(())
}
