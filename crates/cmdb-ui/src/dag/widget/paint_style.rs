//! Palette and text-fitting helpers shared by the DAG paint passes.

use crate::dag::layout::{LayoutEdge, LayoutRect};

pub(super) const EDGE_COLOR: u32 = 0x6b768c;
pub(super) const EDGE_BACK: u32 = 0xc25b5b;
pub(super) const OVERLAY_COLOR: u32 = 0x5f9ea0;
pub(super) const ACCENT: u32 = 0xf0b429;

pub(super) fn edge_box(a: LayoutRect, b: LayoutRect) -> LayoutRect {
    let left = a.x.min(b.x);
    let top = a.y.min(b.y);
    let right = a.right().max(b.right());
    let bottom = (a.y + a.height).max(b.y + b.height);
    LayoutRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

/// Cuts `text` to at most `max_chars` without allocating.
pub(super) fn clamp_chars(text: &str, max_chars: usize) -> &str {
    if max_chars == 0 {
        return "";
    }
    match text.char_indices().nth(max_chars) {
        Some((index, _)) => text.get(..index).unwrap_or(text),
        None => text,
    }
}

pub(super) fn max_chars(width: f64, font: f32) -> usize {
    let capacity = width / (f64::from(font) * 0.62);
    if capacity >= 22.0 {
        22
    } else if capacity >= 16.0 {
        16
    } else if capacity >= 12.0 {
        12
    } else {
        8
    }
}

pub(super) fn edge_style(edge: LayoutEdge, selection: Option<usize>) -> (u32, f32) {
    let selected = selection.is_some_and(|s| s == edge.from || s == edge.to);
    if edge.back {
        (EDGE_BACK, 2.0)
    } else if selected {
        (ACCENT, 2.8)
    } else if edge.overlay {
        (OVERLAY_COLOR, 1.6)
    } else {
        (EDGE_COLOR, 1.8)
    }
}
