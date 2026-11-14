use super::{H_GAP, LayoutRect, NODE_H, NODE_W, PADDING, V_GAP, f32_index};

pub(super) fn place(
    layers: &[Vec<usize>],
    node_count: usize,
) -> (Vec<Option<LayoutRect>>, f32, f32, Vec<usize>) {
    let max_rows = layers.iter().map(Vec::len).max().unwrap_or(0);
    let total_h = f32_index(max_rows) * (NODE_H + V_GAP) - V_GAP;
    let mut rects = vec![None; node_count];
    let mut order = Vec::with_capacity(node_count);
    for (layer_index, layer) in layers.iter().enumerate() {
        place_layer(layer, layer_index, total_h, &mut rects, &mut order);
    }
    let width = PADDING * 2.0 + f32_index(layers.len()) * (NODE_W + H_GAP) - H_GAP;
    let height = PADDING * 2.0 + total_h;
    (rects, width.max(0.0), height.max(0.0), order)
}

fn place_layer(
    layer: &[usize],
    layer_index: usize,
    total_h: f32,
    rects: &mut [Option<LayoutRect>],
    order: &mut Vec<usize>,
) {
    let layer_h = f32_index(layer.len()) * (NODE_H + V_GAP) - V_GAP;
    let y0 = PADDING + (total_h - layer_h) * 0.5;
    for (row, &node) in layer.iter().enumerate() {
        let rect = LayoutRect {
            x: PADDING + f32_index(layer_index) * (NODE_W + H_GAP),
            y: y0 + f32_index(row) * (NODE_H + V_GAP),
            width: NODE_W,
            height: NODE_H,
        };
        if let Some(slot) = rects.get_mut(node) {
            *slot = Some(rect);
        }
        order.push(node);
    }
}
