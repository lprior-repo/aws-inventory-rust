//! Account hierarchy layout over borrowed, interned account arrays.
//!
//! Groups are stored bottom-up: resource leaves, kinds, services, regions, accounts.
//! Every non-leaf's children occupy a contiguous range in `groups`. Resource leaves
//! use the original node index as their label, preserving `node_xy` indexing even
//! when the input nodes of a kind are interleaved with other kinds.

#[path = "hierarchy/pack.rs"]
mod pack;
#[cfg(test)]
#[path = "hierarchy/tests.rs"]
mod tests;

use std::fmt;

/// Width and height of one resource cell. Coordinates are cell top-left corners.
pub const CELL_SIZE: f32 = 16.0;
const PADDING: f32 = 16.0;
/// Matches the default whole-account load limit; also bounds grid conversions.
pub const MAX_NODES: usize = 4_000_000;

/// Only the D4.2 account arrays needed for grouping; no identities or edges are copied.
#[derive(Clone, Copy, Debug)]
pub struct HierarchyInput<'a> {
    pub kinds: &'a [Box<str>],
    pub node_account: &'a [u16],
    pub node_region: &'a [u16],
    pub node_kind: &'a [u32],
}

/// Flat layout in original node order, with bottom-up, contiguous child ranges.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HierarchyLayout {
    pub node_xy: Vec<[f32; 2]>,
    pub groups: Vec<Group>,
}

/// Rectangle `[x, y, width, height]` in world coordinates.
///
/// Levels are account=0, region=1, service=2, kind=3, resource=4. Labels are
/// account/region intern indices, a representative kind index for a service
/// (the smallest child kind index; resolve its second `::` component), kind
/// indices, or original node indices respectively. Leaves have no children.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Group {
    pub level: u8,
    pub rect: [f32; 4],
    pub first_child: u32,
    pub child_count: u32,
    pub node_count: u32,
    pub label: u32,
}

/// Invalid flat-array input or a layout exceeding bounded numeric/storage limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutError {
    LengthMismatch,
    TooManyNodes,
    InvalidKind(u32),
    MissingService(u32),
    Capacity,
    CoordinateLimit,
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthMismatch => f.write_str("hierarchy node array lengths differ"),
            Self::TooManyNodes => f.write_str("hierarchy exceeds 4000000 nodes"),
            Self::InvalidKind(index) => write!(f, "hierarchy kind index {index} is out of bounds"),
            Self::MissingService(index) => {
                write!(f, "hierarchy kind {index} has no service component")
            }
            Self::Capacity => f.write_str("hierarchy array capacity exceeded"),
            Self::CoordinateLimit => f.write_str("hierarchy coordinate limit exceeded"),
        }
    }
}

impl std::error::Error for LayoutError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key<'a> {
    account: u16,
    region: u16,
    service: &'a str,
    kind: u32,
}

#[derive(Clone, Copy)]
struct Node<'a> {
    key: Key<'a>,
    index: u32,
}

#[derive(Clone, Copy)]
struct Entry<'a> {
    key: Key<'a>,
    group: Group,
}

#[derive(Clone, Copy)]
enum Parent {
    Service,
    Region,
    Account,
}

impl Parent {
    fn level(self) -> u8 {
        match self {
            Self::Service => 2,
            Self::Region => 1,
            Self::Account => 0,
        }
    }

    fn same(self, a: Key<'_>, b: Key<'_>) -> bool {
        a.account == b.account
            && match self {
                Self::Service => a.region == b.region && a.service == b.service,
                Self::Region => a.region == b.region,
                Self::Account => true,
            }
    }
}

/// Lay out account, region, service, kind and resource rectangles deterministically.
///
/// Uses a square resource grid and descending-area shelf packing with aspect target
/// 1.5. All ties have deterministic ordering. Work is bounded by the input node count
/// (at most `MAX_NODES`); buffers are contiguous and no per-resource strings are owned.
///
/// # Errors
/// Returns an error for mismatched arrays, invalid kind/service references, allocation
/// failure, more than `MAX_NODES`, or coordinates outside the exact cell-grid range.
pub fn layout(input: HierarchyInput<'_>) -> Result<HierarchyLayout, LayoutError> {
    let nodes = sorted_nodes(input)?;
    let mut groups = buffer(nodes.len())?;
    let kinds = kind_groups(&nodes, &mut groups)?;
    let services = parent_groups(kinds, Parent::Service, &mut groups)?;
    let regions = parent_groups(services, Parent::Region, &mut groups)?;
    let mut accounts = parent_groups(regions, Parent::Account, &mut groups)?;
    pack::rectangles(&mut accounts, 0.0)?;
    reserve(&mut groups, accounts.len())?;
    groups.extend(accounts.into_iter().map(|entry| entry.group));
    translate_groups(&mut groups)?;
    let mut node_xy = buffer(nodes.len())?;
    node_xy.resize(nodes.len(), [0.0; 2]);
    groups
        .iter()
        .filter(|g| g.level == 4)
        .try_for_each(|leaf| {
            let [x, y, _, _] = leaf.rect;
            let slot = node_xy
                .get_mut(index(leaf.label)?)
                .ok_or(LayoutError::Capacity)?;
            *slot = [x, y];
            Ok::<(), LayoutError>(())
        })?;
    Ok(HierarchyLayout { node_xy, groups })
}

fn sorted_nodes(input: HierarchyInput<'_>) -> Result<Vec<Node<'_>>, LayoutError> {
    let count = input.node_kind.len();
    if count != input.node_account.len() || count != input.node_region.len() {
        return Err(LayoutError::LengthMismatch);
    }
    if count > MAX_NODES {
        return Err(LayoutError::TooManyNodes);
    }
    if count == 0 {
        return Ok(Vec::new());
    }
    let mut services = buffer(input.kinds.len())?;
    input.kinds.iter().enumerate().try_for_each(|(i, name)| {
        let service = name
            .split("::")
            .nth(1)
            .filter(|part| !part.is_empty())
            .ok_or(LayoutError::MissingService(count32(i)?))?;
        services.push(service);
        Ok::<(), LayoutError>(())
    })?;
    let mut nodes = buffer(count)?;
    input
        .node_account
        .iter()
        .zip(input.node_region)
        .zip(input.node_kind)
        .enumerate()
        .try_for_each(|(i, ((account, region), kind))| {
            let service = services
                .get(index(*kind)?)
                .copied()
                .ok_or(LayoutError::InvalidKind(*kind))?;
            nodes.push(Node {
                key: Key {
                    account: *account,
                    region: *region,
                    service,
                    kind: *kind,
                },
                index: count32(i)?,
            });
            Ok::<(), LayoutError>(())
        })?;
    nodes.sort_unstable_by(|a, b| a.key.cmp(&b.key).then_with(|| a.index.cmp(&b.index)));
    Ok(nodes)
}

fn kind_groups<'a>(
    nodes: &[Node<'a>],
    groups: &mut Vec<Group>,
) -> Result<Vec<Entry<'a>>, LayoutError> {
    let count = nodes.chunk_by(|a, b| a.key == b.key).count();
    let mut kinds = buffer(count)?;
    nodes
        .chunk_by(|a, b| a.key == b.key)
        .try_for_each(|chunk| {
            let node = chunk.first().ok_or(LayoutError::Capacity)?;
            let first_child = count32(groups.len())?;
            let (columns, rows) = pack::grid(chunk.len())?;
            chunk.iter().enumerate().try_for_each(|(i, node)| {
                let [x, y] = pack::cell(i, columns)?;
                groups.push(Group {
                    level: 4,
                    rect: [x, y, CELL_SIZE, CELL_SIZE],
                    first_child: 0,
                    child_count: 0,
                    node_count: 1,
                    label: node.index,
                });
                Ok::<(), LayoutError>(())
            })?;
            let node_count = count32(chunk.len())?;
            kinds.push(Entry {
                key: node.key,
                group: Group {
                    level: 3,
                    rect: [
                        0.0,
                        0.0,
                        f32::from(columns) * CELL_SIZE + 2.0 * PADDING,
                        f32::from(rows) * CELL_SIZE + 2.0 * PADDING,
                    ],
                    first_child,
                    child_count: node_count,
                    node_count,
                    label: node.key.kind,
                },
            });
            Ok::<(), LayoutError>(())
        })?;
    Ok(kinds)
}

fn parent_groups<'a>(
    mut children: Vec<Entry<'a>>,
    parent: Parent,
    groups: &mut Vec<Group>,
) -> Result<Vec<Entry<'a>>, LayoutError> {
    let count = children.chunk_by(|a, b| parent.same(a.key, b.key)).count();
    let mut parents = buffer(count)?;
    reserve(groups, children.len())?;
    children
        .chunk_by_mut(|a, b| parent.same(a.key, b.key))
        .try_for_each(|chunk| {
            let [width, height] = pack::rectangles(chunk, PADDING)?;
            let first = chunk.first().ok_or(LayoutError::Capacity)?;
            let label = match parent {
                Parent::Service => chunk
                    .iter()
                    .map(|entry| entry.group.label)
                    .min()
                    .ok_or(LayoutError::Capacity)?,
                Parent::Region => u32::from(first.key.region),
                Parent::Account => u32::from(first.key.account),
            };
            let node_count = chunk.iter().try_fold(0_u32, |count, entry| {
                count
                    .checked_add(entry.group.node_count)
                    .ok_or(LayoutError::Capacity)
            })?;
            parents.push(Entry {
                key: first.key,
                group: Group {
                    level: parent.level(),
                    rect: [0.0, 0.0, width, height],
                    first_child: count32(groups.len())?,
                    child_count: count32(chunk.len())?,
                    node_count,
                    label,
                },
            });
            groups.extend(chunk.iter().map(|entry| entry.group));
            Ok::<(), LayoutError>(())
        })?;
    Ok(parents)
}

fn translate_groups(groups: &mut [Group]) -> Result<(), LayoutError> {
    // Parents follow their children, so reverse iteration propagates each origin once.
    (0..groups.len()).rev().try_for_each(|i| {
        let (before, tail) = groups.split_at_mut(i);
        let parent = tail.first().ok_or(LayoutError::Capacity)?;
        let start = index(parent.first_child)?;
        let end = start
            .checked_add(index(parent.child_count)?)
            .ok_or(LayoutError::Capacity)?;
        let [x, y, _, _] = parent.rect;
        before
            .get_mut(start..end)
            .ok_or(LayoutError::Capacity)?
            .iter_mut()
            .try_for_each(|child| {
                let [cx, cy, width, height] = child.rect;
                child.rect = [cx + x, cy + y, width, height];
                pack::check_extent(cx + x + width, cy + y + height)
            })
    })
}

fn count32(count: usize) -> Result<u32, LayoutError> {
    u32::try_from(count).map_err(|_| LayoutError::Capacity)
}

fn index(value: u32) -> Result<usize, LayoutError> {
    usize::try_from(value).map_err(|_| LayoutError::Capacity)
}

fn buffer<T>(capacity: usize) -> Result<Vec<T>, LayoutError> {
    let mut values = Vec::new();
    reserve(&mut values, capacity)?;
    Ok(values)
}

fn reserve<T>(values: &mut Vec<T>, additional: usize) -> Result<(), LayoutError> {
    values
        .try_reserve_exact(additional)
        .map_err(|_| LayoutError::Capacity)
}
