use super::{CELL_SIZE, Entry, LayoutError, PADDING, count32};

const ASPECT_TARGET: f32 = 1.5;
const MAX_COORDINATE: f32 = 16_777_216.0;

pub(super) fn grid(count: usize) -> Result<(u16, u16), LayoutError> {
    let count = count32(count)?;
    if count == 0 {
        return Err(LayoutError::Capacity);
    }
    let root = count.isqrt();
    let columns = if root.saturating_mul(root) == count {
        root
    } else {
        root.saturating_add(1)
    };
    let rows = count
        .checked_add(columns.saturating_sub(1))
        .and_then(|n| n.checked_div(columns))
        .ok_or(LayoutError::Capacity)?;
    Ok((
        u16::try_from(columns).map_err(|_| LayoutError::Capacity)?,
        u16::try_from(rows).map_err(|_| LayoutError::Capacity)?,
    ))
}

pub(super) fn cell(index: usize, columns: u16) -> Result<[f32; 2], LayoutError> {
    let index = count32(index)?;
    let columns = u32::from(columns);
    let x = index.checked_rem(columns).ok_or(LayoutError::Capacity)?;
    let y = index.checked_div(columns).ok_or(LayoutError::Capacity)?;
    let x = f32::from(u16::try_from(x).map_err(|_| LayoutError::Capacity)?);
    let y = f32::from(u16::try_from(y).map_err(|_| LayoutError::Capacity)?);
    Ok([PADDING + x * CELL_SIZE, PADDING + y * CELL_SIZE])
}

pub(super) fn rectangles(children: &mut [Entry<'_>], inset: f32) -> Result<[f32; 2], LayoutError> {
    if children.is_empty() {
        return Ok([0.0; 2]);
    }
    children.sort_unstable_by(|a, b| {
        let [_, _, aw, ah] = a.group.rect;
        let [_, _, bw, bh] = b.group.rect;
        (bw * bh)
            .total_cmp(&(aw * ah))
            .then_with(|| bw.total_cmp(&aw))
            .then_with(|| bh.total_cmp(&ah))
            .then_with(|| a.key.cmp(&b.key))
    });
    let (area, widest) = children
        .iter()
        .fold((0.0_f32, 0.0_f32), |(area, widest), child| {
            let [_, _, width, height] = child.group.rect;
            (
                area + (width + PADDING) * (height + PADDING),
                widest.max(width),
            )
        });
    let target = (area * ASPECT_TARGET).sqrt().max(widest);
    let (_, y, row_height, width) = children.iter_mut().try_fold(
        (inset, inset, 0.0_f32, 0.0_f32),
        |(x, y, row_height, used_width), child| {
            let [_, _, width, height] = child.group.rect;
            let (x, y, row_height) = if x > inset && x + width > inset + target {
                (inset, y + row_height + PADDING, 0.0)
            } else {
                (x, y, row_height)
            };
            child.group.rect = [x, y, width, height];
            check_extent(x + width + inset, y + height + inset)?;
            Ok::<_, LayoutError>((
                x + width + PADDING,
                y,
                row_height.max(height),
                used_width.max(x + width),
            ))
        },
    )?;
    Ok([width + inset, y + row_height + inset])
}

pub(super) fn check_extent(width: f32, height: f32) -> Result<(), LayoutError> {
    if !width.is_finite()
        || !height.is_finite()
        || width > MAX_COORDINATE
        || height > MAX_COORDINATE
    {
        return Err(LayoutError::CoordinateLimit);
    }
    Ok(())
}
