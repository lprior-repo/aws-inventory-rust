use super::{CELL_SIZE, Group, HierarchyInput, HierarchyLayout, LayoutError, layout};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn hierarchy_empty_graph() -> TestResult {
    let result = layout(HierarchyInput {
        kinds: &[],
        node_account: &[],
        node_region: &[],
        node_kind: &[],
    })?;
    assert_eq!(
        result,
        HierarchyLayout {
            node_xy: vec![],
            groups: vec![]
        }
    );
    Ok(())
}

#[test]
fn hierarchy_one_node_has_all_five_levels() -> TestResult {
    let kinds = [Box::from("AWS::EC2::VPC")];
    let result = layout(HierarchyInput {
        kinds: &kinds,
        node_account: &[7],
        node_region: &[9],
        node_kind: &[0],
    })?;
    assert_eq!(result.node_xy, vec![[64.0, 64.0]]);
    assert_eq!(
        result.groups.iter().map(|g| g.level).collect::<Vec<_>>(),
        vec![4, 3, 2, 1, 0]
    );
    assert_eq!(
        result.groups.iter().map(|g| g.label).collect::<Vec<_>>(),
        vec![0, 0, 0, 9, 7]
    );
    assert_eq!(
        result.groups.iter().map(|g| g.rect).collect::<Vec<_>>(),
        vec![
            [64.0, 64.0, 16.0, 16.0],
            [48.0, 48.0, 48.0, 48.0],
            [32.0, 32.0, 80.0, 80.0],
            [16.0, 16.0, 112.0, 112.0],
            [0.0, 0.0, 144.0, 144.0],
        ]
    );
    check_containment(&result)?;
    Ok(())
}

#[test]
fn hierarchy_groups_services_and_preserves_node_indices() -> TestResult {
    let kinds = [
        Box::from("AWS::EC2::VPC"),
        Box::from("AWS::S3::Bucket"),
        Box::from("AWS::EC2::Subnet"),
    ];
    let input = HierarchyInput {
        kinds: &kinds,
        node_account: &[1, 0, 0, 0, 0, 0],
        node_region: &[0, 1, 0, 0, 0, 0],
        node_kind: &[0, 0, 0, 2, 1, 0],
    };
    let result = layout(input)?;
    let counts = (0..=4)
        .map(|level| result.groups.iter().filter(|g| g.level == level).count())
        .collect::<Vec<_>>();
    assert_eq!(counts, vec![2, 3, 4, 5, 6]);
    let service = result
        .groups
        .iter()
        .find(|g| g.level == 2 && g.node_count == 3)
        .ok_or("missing EC2 service")?;
    assert_eq!((service.label, service.child_count), (0, 2));
    check_containment(&result)?;
    check_node_identities(&result, input)?;
    check_cells(&result)?;
    Ok(())
}

#[test]
fn hierarchy_square_grid_is_fixed_size() -> TestResult {
    let kinds = [Box::from("AWS::EC2::VPC")];
    let result = layout(HierarchyInput {
        kinds: &kinds,
        node_account: &[0; 5],
        node_region: &[0; 5],
        node_kind: &[0; 5],
    })?;
    assert_eq!(
        result.node_xy,
        vec![
            [64.0, 64.0],
            [80.0, 64.0],
            [96.0, 64.0],
            [64.0, 80.0],
            [80.0, 80.0]
        ]
    );
    let kind = result
        .groups
        .iter()
        .find(|group| group.level == 3)
        .ok_or("missing kind")?;
    assert_eq!(kind.rect, [48.0, 48.0, 80.0, 64.0]);
    check_containment(&result)?;
    check_cells(&result)?;
    Ok(())
}

#[test]
fn hierarchy_is_bitwise_deterministic() -> TestResult {
    let kinds = [Box::from("AWS::S3::Bucket"), Box::from("AWS::EC2::VPC")];
    let input = HierarchyInput {
        kinds: &kinds,
        node_account: &[1, 0, 1, 0],
        node_region: &[0, 1, 0, 1],
        node_kind: &[1, 0, 0, 1],
    };
    let first = layout(input)?;
    let second = layout(input)?;
    assert_eq!(first, second);
    assert_eq!(float_bits(&first), float_bits(&second));
    let account = first
        .groups
        .iter()
        .find(|group| group.level == 0 && group.label == 0)
        .ok_or("missing account zero")?;
    let start = usize::try_from(account.first_child)?;
    let end = start
        .checked_add(usize::try_from(account.child_count)?)
        .ok_or("child overflow")?;
    let region = first
        .groups
        .get(start..end)
        .ok_or("missing regions")?
        .iter()
        .find(|group| group.level == 1 && group.label == 1)
        .ok_or("missing region one")?;
    let start = usize::try_from(region.first_child)?;
    let end = start
        .checked_add(usize::try_from(region.child_count)?)
        .ok_or("child overflow")?;
    let services = first.groups.get(start..end).ok_or("missing services")?;
    assert_eq!(
        services
            .iter()
            .map(|group| (group.level, group.label))
            .collect::<Vec<_>>(),
        vec![(2, 1), (2, 0)]
    );
    Ok(())
}

#[test]
fn hierarchy_rejects_invalid_flat_arrays() -> TestResult {
    let kinds = [Box::from("AWS::EC2::VPC")];
    let input = HierarchyInput {
        kinds: &kinds,
        node_account: &[0],
        node_region: &[],
        node_kind: &[0],
    };
    assert_eq!(
        layout(input).err().ok_or("expected an error")?,
        LayoutError::LengthMismatch
    );
    let input = HierarchyInput {
        node_region: &[0],
        node_kind: &[1],
        ..input
    };
    let error = layout(input).err().ok_or("expected an error")?;
    assert_eq!(error.to_string(), "hierarchy kind index 1 is out of bounds");
    let bad = [Box::from("AWS::::VPC")];
    let input = HierarchyInput {
        kinds: &bad,
        node_kind: &[0],
        ..input
    };
    assert_eq!(
        layout(input).err().ok_or("expected an error")?.to_string(),
        "hierarchy kind 0 has no service component"
    );
    Ok(())
}

#[test]
fn hierarchy_250000_nodes() -> TestResult {
    let kinds = [
        Box::from("AWS::EC2::VPC"),
        Box::from("AWS::EC2::Subnet"),
        Box::from("AWS::S3::Bucket"),
    ];
    let accounts = (0_u32..250_000)
        .map(|i| u16::try_from(i % 3))
        .collect::<Result<Vec<_>, _>>()?;
    let regions = (0_u32..250_000)
        .map(|i| u16::try_from((i / 3) % 4))
        .collect::<Result<Vec<_>, _>>()?;
    let node_kinds = (0_u32..250_000).map(|i| (i / 12) % 3).collect::<Vec<_>>();
    let input = HierarchyInput {
        kinds: &kinds,
        node_account: &accounts,
        node_region: &regions,
        node_kind: &node_kinds,
    };
    let result = layout(input)?;
    assert_eq!(result.node_xy.len(), 250_000);
    assert_eq!(
        result
            .groups
            .iter()
            .filter(|g| g.level == 0)
            .map(|g| g.node_count)
            .sum::<u32>(),
        250_000
    );
    assert_eq!(result.groups.len(), 250_075);
    check_containment(&result)?;
    check_node_identities(&result, input)?;
    check_cells(&result)?;
    Ok(())
}

fn check_containment(layout: &HierarchyLayout) -> TestResult {
    layout.groups.iter().try_for_each(|parent| -> TestResult {
        if parent.level == 4 {
            let [x, y] = *layout
                .node_xy
                .get(usize::try_from(parent.label)?)
                .ok_or("missing node")?;
            assert_eq!(parent.rect, [x, y, CELL_SIZE, CELL_SIZE]);
            assert_eq!((parent.child_count, parent.node_count), (0, 1));
            return Ok(());
        }
        let start = usize::try_from(parent.first_child)?;
        let end = start
            .checked_add(usize::try_from(parent.child_count)?)
            .ok_or("child overflow")?;
        let children = layout.groups.get(start..end).ok_or("invalid child range")?;
        assert_eq!(
            children.iter().map(|g| g.node_count).sum::<u32>(),
            parent.node_count
        );
        children.iter().for_each(|child| {
            assert_eq!(child.level, parent.level.saturating_add(1));
            let [px, py, pw, ph] = parent.rect;
            let [cx, cy, cw, ch] = child.rect;
            assert!(cx >= px && cy >= py && cx + cw <= px + pw && cy + ch <= py + ph);
        });
        Ok(())
    })
}

fn check_cells(layout: &HierarchyLayout) -> TestResult {
    let mut cells = layout.node_xy.to_vec();
    cells.sort_unstable_by(|[ax, ay], [bx, by]| ay.total_cmp(by).then_with(|| ax.total_cmp(bx)));
    cells.windows(2).for_each(|pair| {
        if let [a, b] = pair {
            let [ax, ay] = *a;
            let [bx, by] = *b;
            assert!(by - ay >= CELL_SIZE || bx - ax >= CELL_SIZE);
        }
    });
    assert_eq!(
        cells
            .iter()
            .filter(|[x, y]| x % CELL_SIZE != 0.0 || y % CELL_SIZE != 0.0)
            .count(),
        0
    );
    Ok(())
}

fn float_bits(layout: &HierarchyLayout) -> Vec<u32> {
    layout
        .node_xy
        .iter()
        .flat_map(|xy| xy.iter())
        .chain(layout.groups.iter().flat_map(|g| g.rect.iter()))
        .map(|v| v.to_bits())
        .collect()
}

#[test]
fn hierarchy_sibling_rectangles_do_not_overlap() -> TestResult {
    let kinds = [
        Box::from("AWS::EC2::VPC"),
        Box::from("AWS::EC2::Subnet"),
        Box::from("AWS::S3::Bucket"),
    ];
    let node_kind = (0_u32..90)
        .map(|i| {
            if i < 60 {
                0
            } else if i < 80 {
                1
            } else {
                2
            }
        })
        .collect::<Vec<_>>();
    let result = layout(HierarchyInput {
        kinds: &kinds,
        node_account: &[0; 90],
        node_region: &[0; 90],
        node_kind: &node_kind,
    })?;
    result
        .groups
        .iter()
        .filter(|g| g.level < 3)
        .try_for_each(|parent| -> TestResult {
            let start = usize::try_from(parent.first_child)?;
            let end = start
                .checked_add(usize::try_from(parent.child_count)?)
                .ok_or("child overflow")?;
            let children = result.groups.get(start..end).ok_or("missing children")?;
            children.iter().enumerate().for_each(|(i, a)| {
                children
                    .iter()
                    .skip(i.saturating_add(1))
                    .for_each(|b| assert!(!overlaps(a, b)));
            });
            Ok(())
        })?;
    check_containment(&result)?;
    check_cells(&result)?;
    Ok(())
}

fn overlaps(a: &Group, b: &Group) -> bool {
    let [ax, ay, aw, ah] = a.rect;
    let [bx, by, bw, bh] = b.rect;
    ax < bx + bw && bx < ax + aw && ay < by + bh && by < ay + ah
}

#[test]
fn hierarchy_packs_large_kinds_first_and_wraps_rows() -> TestResult {
    let kinds = [Box::from("AWS::EC2::Subnet"), Box::from("AWS::EC2::VPC")];
    let node_kind = (0_u32..80)
        .map(|i| if i < 20 { 0 } else { 1 })
        .collect::<Vec<_>>();
    let result = layout(HierarchyInput {
        kinds: &kinds,
        node_account: &[0; 80],
        node_region: &[0; 80],
        node_kind: &node_kind,
    })?;
    let service = result
        .groups
        .iter()
        .find(|g| g.level == 2)
        .ok_or("missing service")?;
    assert_eq!(
        (service.label, service.node_count, service.child_count),
        (0, 80, 2)
    );
    let first = usize::try_from(service.first_child)?;
    let large = result.groups.get(first).ok_or("missing large kind")?;
    let small = result
        .groups
        .get(first.checked_add(1).ok_or("child overflow")?)
        .ok_or("missing small kind")?;
    let [sx, sy, sw, sh] = service.rect;
    let [lx, ly, lw, lh] = large.rect;
    let [tx, ty, tw, th] = small.rect;
    assert_eq!((large.label, small.label), (1, 0));
    assert_eq!([sw, sh], [192.0, 304.0]);
    assert_eq!([lx - sx, ly - sy, lw, lh], [16.0, 16.0, 160.0, 160.0]);
    assert_eq!([tx - sx, ty - sy, tw, th], [16.0, 192.0, 112.0, 96.0]);
    check_containment(&result)?;
    check_cells(&result)?;
    Ok(())
}

#[test]
fn hierarchy_rejects_excessive_node_count() -> TestResult {
    let count = super::MAX_NODES
        .checked_add(1)
        .ok_or("node count overflow")?;
    let accounts = vec![0; count];
    let node_kind = vec![0; count];
    let kinds = [Box::from("AWS::EC2::VPC")];
    let error = layout(HierarchyInput {
        kinds: &kinds,
        node_account: &accounts,
        node_region: &accounts,
        node_kind: &node_kind,
    })
    .err()
    .ok_or("expected an error")?;
    assert_eq!(error, LayoutError::TooManyNodes);
    assert_eq!(error.to_string(), "hierarchy exceeds 4000000 nodes");
    Ok(())
}

#[test]
fn hierarchy_rejects_each_misaligned_node_array() -> TestResult {
    let kinds = [Box::from("AWS::EC2::VPC")];
    let cases: [(&[u16], &[u16], &[u32]); 6] = [
        (&[0, 0], &[0], &[0]),
        (&[], &[0], &[0]),
        (&[0], &[0, 0], &[0]),
        (&[0], &[], &[0]),
        (&[0], &[0], &[0, 0]),
        (&[0], &[0], &[]),
    ];
    cases
        .into_iter()
        .try_for_each(|(node_account, node_region, node_kind)| -> TestResult {
            let error = layout(HierarchyInput {
                kinds: &kinds,
                node_account,
                node_region,
                node_kind,
            })
            .err()
            .ok_or("expected a length error")?;
            assert_eq!(error, LayoutError::LengthMismatch);
            assert_eq!(error.to_string(), "hierarchy node array lengths differ");
            Ok(())
        })
}

#[test]
fn hierarchy_aspect_target_keeps_two_singletons_on_the_first_row() -> TestResult {
    let kinds = [
        Box::from("AWS::EC2::VPC"),
        Box::from("AWS::EC2::Subnet"),
        Box::from("AWS::EC2::Instance"),
    ];
    let input = HierarchyInput {
        kinds: &kinds,
        node_account: &[0; 3],
        node_region: &[0; 3],
        node_kind: &[0, 1, 2],
    };
    let result = layout(input)?;
    let service = result
        .groups
        .iter()
        .find(|group| group.level == 2)
        .ok_or("missing service")?;
    let [sx, sy, width, height] = service.rect;
    let offsets = result
        .groups
        .iter()
        .filter(|group| group.level == 3)
        .map(|group| {
            let [x, y, width, height] = group.rect;
            (group.label, [x - sx, y - sy, width, height])
        })
        .collect::<Vec<_>>();
    assert_eq!([width, height], [144.0, 144.0]);
    assert_eq!(
        offsets,
        vec![
            (0, [16.0, 16.0, 48.0, 48.0]),
            (1, [80.0, 16.0, 48.0, 48.0]),
            (2, [16.0, 80.0, 48.0, 48.0]),
        ]
    );
    check_node_identities(&result, input)?;
    Ok(())
}

#[test]
fn hierarchy_coordinate_guard_checks_both_axes_and_nonfinite_values() -> TestResult {
    let limit = 16_777_216.0;
    assert_eq!(super::pack::check_extent(limit, limit), Ok(()));
    let invalid = [
        [limit + CELL_SIZE, 0.0],
        [0.0, limit + CELL_SIZE],
        [f32::NAN, 0.0],
        [0.0, f32::NAN],
        [f32::INFINITY, 0.0],
        [0.0, f32::INFINITY],
        [f32::NEG_INFINITY, 0.0],
        [0.0, f32::NEG_INFINITY],
    ];
    invalid
        .into_iter()
        .try_for_each(|[width, height]| -> TestResult {
            let error = super::pack::check_extent(width, height)
                .err()
                .ok_or("expected a coordinate error")?;
            assert_eq!(error, LayoutError::CoordinateLimit);
            assert_eq!(error.to_string(), "hierarchy coordinate limit exceeded");
            Ok(())
        })
}

#[test]
fn hierarchy_capacity_errors_are_explicit() -> TestResult {
    let mut buffer = Vec::<u8>::new();
    let results = [
        super::pack::grid(0).map(|_| ()),
        super::pack::cell(0, 0).map(|_| ()),
        super::reserve(&mut buffer, usize::MAX),
    ];
    results.into_iter().try_for_each(|result| -> TestResult {
        let error = result.err().ok_or("expected a capacity error")?;
        assert_eq!(error, LayoutError::Capacity);
        assert_eq!(error.to_string(), "hierarchy array capacity exceeded");
        Ok(())
    })
}

fn check_node_identities(layout: &HierarchyLayout, input: HierarchyInput<'_>) -> TestResult {
    let mut parents = vec![None; layout.groups.len()];
    layout
        .groups
        .iter()
        .enumerate()
        .filter(|(_, group)| group.level < 4)
        .try_for_each(|(i, group)| -> TestResult {
            let start = usize::try_from(group.first_child)?;
            let end = start
                .checked_add(usize::try_from(group.child_count)?)
                .ok_or("child overflow")?;
            parents
                .get_mut(start..end)
                .ok_or("missing children")?
                .iter_mut()
                .for_each(|slot| {
                    assert_eq!(*slot, None, "each child has exactly one parent");
                    *slot = Some(i);
                });
            Ok(())
        })?;
    layout
        .groups
        .iter()
        .enumerate()
        .filter(|(_, group)| group.level == 4)
        .try_for_each(|(i, leaf)| check_node_ancestors(layout, input, &parents, i, leaf.label))
}

fn check_node_ancestors(
    layout: &HierarchyLayout,
    input: HierarchyInput<'_>,
    parents: &[Option<usize>],
    leaf_index: usize,
    label: u32,
) -> TestResult {
    let node = usize::try_from(label)?;
    let kind = *input.node_kind.get(node).ok_or("missing input kind")?;
    let region = u32::from(*input.node_region.get(node).ok_or("missing input region")?);
    let account = u32::from(
        *input
            .node_account
            .get(node)
            .ok_or("missing input account")?,
    );
    let [x, y] = *layout.node_xy.get(node).ok_or("missing coordinates")?;
    [(3, kind), (2, kind), (1, region), (0, account)]
        .into_iter()
        .try_fold(
            leaf_index,
            |child, (level, expected)| -> Result<usize, Box<dyn std::error::Error>> {
                let parent = parents
                    .get(child)
                    .copied()
                    .flatten()
                    .ok_or("missing parent")?;
                let group = layout.groups.get(parent).ok_or("missing ancestor")?;
                assert_eq!(group.level, level);
                if level == 2 {
                    let name = input
                        .kinds
                        .get(usize::try_from(group.label)?)
                        .ok_or("missing service label")?;
                    let expected = input
                        .kinds
                        .get(usize::try_from(expected)?)
                        .ok_or("missing kind name")?;
                    assert_eq!(name.split("::").nth(1), expected.split("::").nth(1));
                } else {
                    assert_eq!(group.label, expected);
                }
                let [gx, gy, width, height] = group.rect;
                assert!(
                    x >= gx
                        && y >= gy
                        && x + CELL_SIZE <= gx + width
                        && y + CELL_SIZE <= gy + height
                );
                Ok(parent)
            },
        )?;
    Ok(())
}

#[test]
fn hierarchy_aspect_target_wraps_the_third_of_five_singletons() -> TestResult {
    let kinds = [
        Box::from("AWS::EC2::A"),
        Box::from("AWS::EC2::B"),
        Box::from("AWS::EC2::C"),
        Box::from("AWS::EC2::D"),
        Box::from("AWS::EC2::E"),
    ];
    let input = HierarchyInput {
        kinds: &kinds,
        node_account: &[0; 5],
        node_region: &[0; 5],
        node_kind: &[0, 1, 2, 3, 4],
    };
    let result = layout(input)?;
    let service = result
        .groups
        .iter()
        .find(|group| group.level == 2)
        .ok_or("missing service")?;
    let [sx, sy, width, height] = service.rect;
    let offsets = result
        .groups
        .iter()
        .filter(|group| group.level == 3)
        .map(|group| {
            let [x, y, width, height] = group.rect;
            (group.label, [x - sx, y - sy, width, height])
        })
        .collect::<Vec<_>>();
    assert_eq!([width, height], [144.0, 208.0]);
    assert_eq!(
        offsets,
        vec![
            (0, [16.0, 16.0, 48.0, 48.0]),
            (1, [80.0, 16.0, 48.0, 48.0]),
            (2, [16.0, 80.0, 48.0, 48.0]),
            (3, [80.0, 80.0, 48.0, 48.0]),
            (4, [16.0, 144.0, 48.0, 48.0]),
        ]
    );
    check_node_identities(&result, input)?;
    Ok(())
}

#[test]
fn hierarchy_multi_account_and_region_rectangles_do_not_overlap() -> TestResult {
    let kinds = [Box::from("AWS::EC2::VPC")];
    let input = HierarchyInput {
        kinds: &kinds,
        node_account: &[0, 0, 1, 1],
        node_region: &[0, 1, 0, 1],
        node_kind: &[0; 4],
    };
    let result = layout(input)?;
    assert_eq!(result.groups.len(), 18);
    (0..4).for_each(|level| {
        let peers = result
            .groups
            .iter()
            .filter(|group| group.level == level)
            .collect::<Vec<_>>();
        peers.iter().enumerate().for_each(|(i, a)| {
            peers
                .iter()
                .skip(i.saturating_add(1))
                .for_each(|b| assert!(!overlaps(a, b)));
        });
    });
    check_node_identities(&result, input)?;
    check_containment(&result)?;
    Ok(())
}

#[test]
fn hierarchy_accepts_exact_maximum_node_count() -> TestResult {
    let count = super::MAX_NODES;
    let accounts = vec![0; count];
    let node_kind = vec![0; count];
    let kinds = [Box::from("AWS::EC2::VPC")];
    let started = std::time::Instant::now();
    let result = layout(HierarchyInput {
        kinds: &kinds,
        node_account: &accounts,
        node_region: &accounts,
        node_kind: &node_kind,
    })?;
    eprintln!(
        "hierarchy MAX_NODES={count} layout_seconds={:.6}",
        started.elapsed().as_secs_f64()
    );
    assert_eq!(result.node_xy.len(), count);
    assert_eq!(
        result.groups.len(),
        count.checked_add(4).ok_or("group count overflow")?
    );
    assert_eq!(result.node_xy.first(), Some(&[64.0, 64.0]));
    assert_eq!(result.node_xy.last(), Some(&[32_048.0, 32_048.0]));
    let kind = result
        .groups
        .iter()
        .find(|group| group.level == 3)
        .ok_or("missing kind")?;
    assert_eq!(kind.rect, [48.0, 48.0, 32_032.0, 32_032.0]);
    Ok(())
}

#[test]
fn hierarchy_aspect_target_fits_three_square_kinds_at_one_point_five() -> TestResult {
    let kinds = [
        Box::from("AWS::EC2::A"),
        Box::from("AWS::EC2::B"),
        Box::from("AWS::EC2::C"),
        Box::from("AWS::EC2::D"),
        Box::from("AWS::EC2::E"),
        Box::from("AWS::EC2::F"),
    ];
    let node_kind = (0_u32..294).map(|node| node / 49).collect::<Vec<_>>();
    let result = layout(HierarchyInput {
        kinds: &kinds,
        node_account: &[0; 294],
        node_region: &[0; 294],
        node_kind: &node_kind,
    })?;
    let service = result
        .groups
        .iter()
        .find(|group| group.level == 2)
        .ok_or("missing service")?;
    let [sx, sy, width, height] = service.rect;
    let offsets = result
        .groups
        .iter()
        .filter(|group| group.level == 3)
        .map(|group| {
            let [x, y, width, height] = group.rect;
            (group.label, [x - sx, y - sy, width, height])
        })
        .collect::<Vec<_>>();
    assert_eq!([width, height], [496.0, 336.0]);
    assert_eq!(
        offsets,
        vec![
            (0, [16.0, 16.0, 144.0, 144.0]),
            (1, [176.0, 16.0, 144.0, 144.0]),
            (2, [336.0, 16.0, 144.0, 144.0]),
            (3, [16.0, 176.0, 144.0, 144.0]),
            (4, [176.0, 176.0, 144.0, 144.0]),
            (5, [336.0, 176.0, 144.0, 144.0]),
        ]
    );
    check_containment(&result)?;
    check_cells(&result)?;
    Ok(())
}

#[test]
fn hierarchy_aspect_target_brackets_lower_at_one_point_four_five() -> TestResult {
    let kinds = [
        Box::from("AWS::EC2::A"),
        Box::from("AWS::EC2::B"),
        Box::from("AWS::EC2::C"),
        Box::from("AWS::EC2::D"),
        Box::from("AWS::EC2::E"),
        Box::from("AWS::EC2::F"),
    ];
    let node_kind = (0_u32..1734).map(|node| node / 289).collect::<Vec<_>>();
    let result = layout(HierarchyInput {
        kinds: &kinds,
        node_account: &[0; 1734],
        node_region: &[0; 1734],
        node_kind: &node_kind,
    })?;
    let service = result
        .groups
        .iter()
        .find(|group| group.level == 2)
        .ok_or("missing service")?;
    let [sx, sy, width, height] = service.rect;
    let offsets = result
        .groups
        .iter()
        .filter(|group| group.level == 3)
        .map(|group| {
            let [x, y, width, height] = group.rect;
            (group.label, [x - sx, y - sy, width, height])
        })
        .collect::<Vec<_>>();
    assert_eq!([width, height], [976.0, 656.0]);
    assert_eq!(
        offsets,
        vec![
            (0, [16.0, 16.0, 304.0, 304.0]),
            (1, [336.0, 16.0, 304.0, 304.0]),
            (2, [656.0, 16.0, 304.0, 304.0]),
            (3, [16.0, 336.0, 304.0, 304.0]),
            (4, [336.0, 336.0, 304.0, 304.0]),
            (5, [656.0, 336.0, 304.0, 304.0]),
        ]
    );
    check_containment(&result)?;
    check_cells(&result)?;
    Ok(())
}

#[test]
fn hierarchy_aspect_target_brackets_upper_at_one_point_five_one() -> TestResult {
    let kinds = [
        Box::from("AWS::EC2::A"),
        Box::from("AWS::EC2::B"),
        Box::from("AWS::EC2::C"),
        Box::from("AWS::EC2::D"),
        Box::from("AWS::EC2::E"),
        Box::from("AWS::EC2::F"),
        Box::from("AWS::EC2::G"),
        Box::from("AWS::EC2::H"),
        Box::from("AWS::EC2::I"),
        Box::from("AWS::EC2::J"),
    ];
    let node_kind = (0_u32..250).map(|node| node / 25).collect::<Vec<_>>();
    let result = layout(HierarchyInput {
        kinds: &kinds,
        node_account: &[0; 250],
        node_region: &[0; 250],
        node_kind: &node_kind,
    })?;
    let service = result
        .groups
        .iter()
        .find(|group| group.level == 2)
        .ok_or("missing service")?;
    let [sx, sy, width, height] = service.rect;
    let offsets = result
        .groups
        .iter()
        .filter(|group| group.level == 3)
        .map(|group| {
            let [x, y, width, height] = group.rect;
            (group.label, [x - sx, y - sy, width, height])
        })
        .collect::<Vec<_>>();
    assert_eq!([width, height], [400.0, 528.0]);
    assert_eq!(
        offsets,
        vec![
            (0, [16.0, 16.0, 112.0, 112.0]),
            (1, [144.0, 16.0, 112.0, 112.0]),
            (2, [272.0, 16.0, 112.0, 112.0]),
            (3, [16.0, 144.0, 112.0, 112.0]),
            (4, [144.0, 144.0, 112.0, 112.0]),
            (5, [272.0, 144.0, 112.0, 112.0]),
            (6, [16.0, 272.0, 112.0, 112.0]),
            (7, [144.0, 272.0, 112.0, 112.0]),
            (8, [272.0, 272.0, 112.0, 112.0]),
            (9, [16.0, 400.0, 112.0, 112.0]),
        ]
    );
    check_containment(&result)?;
    check_cells(&result)?;
    Ok(())
}
