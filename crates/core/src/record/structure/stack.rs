//! Bounded validation borrows caller-owned inline storage; generic validation retains its Vec.
use super::{Frame, RecordError};
use crate::record::MAX_DEPTH;

#[cfg(test)]
mod tests;

pub(super) enum Frames<'a, 'scratch> {
    Inline {
        slots: &'scratch mut [Option<Frame<'a>>; MAX_DEPTH],
        len: usize,
        limit: usize,
    },
    Dynamic(Vec<Frame<'a>>),
}

impl<'a, 'scratch> Frames<'a, 'scratch> {
    pub(super) fn inline(
        slots: &'scratch mut [Option<Frame<'a>>; MAX_DEPTH],
        limit: usize,
    ) -> Result<Self, RecordError> {
        if limit > MAX_DEPTH {
            return Err(RecordError::DepthLimit);
        }
        Ok(Self::Inline {
            slots,
            len: 0,
            limit,
        })
    }

    pub(super) fn dynamic() -> Self {
        Self::Dynamic(Vec::new())
    }

    pub(super) fn last(&self) -> Option<&Frame<'a>> {
        match self {
            Self::Inline { slots, len, .. } => slots.get(len.checked_sub(1)?)?.as_ref(),
            Self::Dynamic(frames) => frames.last(),
        }
    }

    pub(super) fn last_mut(&mut self) -> Option<&mut Frame<'a>> {
        match self {
            Self::Inline { slots, len, .. } => slots.get_mut(len.checked_sub(1)?)?.as_mut(),
            Self::Dynamic(frames) => frames.last_mut(),
        }
    }

    pub(super) fn pop(&mut self) -> Option<Frame<'a>> {
        match self {
            Self::Inline { slots, len, .. } => pop_inline(slots, len),
            Self::Dynamic(frames) => frames.pop(),
        }
    }

    pub(super) fn push(&mut self, frame: Frame<'a>) -> Result<(), RecordError> {
        match self {
            Self::Inline { slots, len, limit } => {
                if *len >= *limit {
                    return Err(RecordError::DepthLimit);
                }
                push_inline(slots, len, frame)?;
            }
            Self::Dynamic(frames) => {
                frames.try_reserve(1).map_err(|_| RecordError::Allocation)?;
                frames.push(frame);
            }
        }
        Ok(())
    }
}

fn pop_inline<'a>(
    slots: &mut [Option<Frame<'a>>; MAX_DEPTH],
    len: &mut usize,
) -> Option<Frame<'a>> {
    let index = len.checked_sub(1)?;
    let frame = slots.get_mut(index)?.take()?;
    *len = index;
    Some(frame)
}

fn push_inline<'a>(
    slots: &mut [Option<Frame<'a>>; MAX_DEPTH],
    len: &mut usize,
    frame: Frame<'a>,
) -> Result<(), RecordError> {
    let next = len.checked_add(1).ok_or(RecordError::DepthLimit)?;
    *slots.get_mut(*len).ok_or(RecordError::DepthLimit)? = Some(frame);
    *len = next;
    Ok(())
}
