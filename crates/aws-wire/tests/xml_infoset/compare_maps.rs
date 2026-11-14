use super::{Node, compare, local_name, text_matches};
use inventory_core::catalog::{Service, Shape};

pub(super) fn compare_map(
    left: &Node,
    right: &Node,
    service: Service<'_>,
    shape: Shape<'_>,
    depth: usize,
    entry_name: &str,
) -> bool {
    let l = left.children.iter().collect::<Vec<_>>();
    let r = right.children.iter().collect::<Vec<_>>();
    compare_map_nodes(&l, &r, service, shape, depth, entry_name)
}

pub(super) fn compare_map_nodes(
    left: &[&Node],
    right: &[&Node],
    service: Service<'_>,
    shape: Shape<'_>,
    depth: usize,
    entry_name: &str,
) -> bool {
    let (Ok(Some(key_target)), Ok(Some(value_target))) = (shape.map_key(), shape.map_value())
    else {
        return false;
    };
    let (Ok(key_wire), Ok(value_wire)) = (shape.map_key_wire(), shape.map_value_wire()) else {
        return false;
    };
    let key_name = match key_wire.xml_name() {
        Ok(Some(name)) => name,
        Ok(None) => "key",
        Err(_) => return false,
    };
    let value_name = match value_wire.xml_name() {
        Ok(Some(name)) => name,
        Ok(None) => "value",
        Err(_) => return false,
    };
    if left.len() != right.len() {
        return false;
    }
    left.iter().all(|entry| {
        let Some((key, _)) = entry_parts(entry, entry_name, key_name, value_name) else {
            return false;
        };
        let left_matches = left
            .iter()
            .filter_map(|other| entry_parts(other, entry_name, key_name, value_name))
            .filter(|(candidate, _)| candidate.text == key.text)
            .count();
        let matches = right
            .iter()
            .filter_map(|other| {
                entry_parts(other, entry_name, key_name, value_name).map(|pair| (*other, pair))
            })
            .filter(|(_, (candidate, _))| candidate.text == key.text)
            .collect::<Vec<_>>();
        if left_matches != 1 {
            return false;
        }
        match matches.as_slice() {
            [(other, (other_key, other_value))] => {
                entry.name == other.name
                    && entry.attributes == other.attributes
                    && text_matches(&entry.text, &other.text, true)
                    && compare(key, other_key, service, key_target, depth + 1)
                    && entry_parts(entry, entry_name, key_name, value_name).is_some_and(
                        |(_, value)| compare(value, other_value, service, value_target, depth + 1),
                    )
            }
            _ => false,
        }
    }) && right.iter().all(|entry| {
        let Some((key, _)) = entry_parts(entry, entry_name, key_name, value_name) else {
            return false;
        };
        right
            .iter()
            .filter_map(|other| entry_parts(other, entry_name, key_name, value_name))
            .filter(|(candidate, _)| candidate.text == key.text)
            .count()
            == 1
    })
}

fn entry_parts<'a>(
    entry: &'a Node,
    entry_name: &str,
    key_name: &str,
    value_name: &str,
) -> Option<(&'a Node, &'a Node)> {
    if local_name(&entry.name) != entry_name || entry.children.len() != 2 {
        return None;
    }
    Some((
        entry
            .children
            .iter()
            .find(|node| local_name(&node.name) == key_name)?,
        entry
            .children
            .iter()
            .find(|node| local_name(&node.name) == value_name)?,
    ))
}
