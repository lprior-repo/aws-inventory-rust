use std::collections::HashMap;

use crate::HarnessError;

use super::lexical;
use super::mask;

const MAX_FILES: usize = 4_096;
const MAX_SOURCE_BYTES: usize = 32 * 1024 * 1024;
const MAX_CALLS: usize = 4_096;

pub type SourceInventoryResult =
    Result<(HashMap<String, String>, Vec<Callsite>, usize), HarnessError>;

pub struct Callsite {
    pub id: String,
    pub path: String,
    pub function: String,
    pub sdk_service: String,
    pub sdk_operation: String,
    pub receiver: String,
    pub operation_line: usize,
    pub send_line: usize,
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let before = &source[..offset];
    let line = before.matches('\n').count().saturating_add(1);
    let line_start = before.rfind('\n').map(|i| i.saturating_add(1)).unwrap_or(0);
    let column = offset.saturating_sub(line_start);
    (line, column)
}

fn source_error(path: &str, source: &str, offset: usize, message: &str) -> HarnessError {
    let (line, column) = line_column(source, offset);
    HarnessError::invalid(format!("{path}:{line}:{column}: {message}"))
}

fn function_at(masked: &str, offset: usize) -> Result<(String, usize), HarnessError> {
    let before = &masked[..offset];
    let mut last_pos = 0;
    let mut found = false;
    for (pos, _) in before.match_indices("fn ") {
        let name_start = pos.saturating_add(3);
        if name_start >= masked.len() {
            continue;
        }
        let rest = &masked[name_start..];
        if let Some(first) = rest.as_bytes().first() {
            let byte_value = *first;
            if byte_value.is_ascii_alphabetic() || byte_value == b'_' {
                last_pos = pos;
                found = true;
            }
        }
    }

    if !found {
        return Err(HarnessError::invalid(
            "SDK send is not inside a recognized function",
        ));
    }

    let name_start = last_pos.saturating_add(3);
    let rest = &masked[name_start..];
    let name_end = rest
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(rest.len());
    let function_name = &rest[..name_end];

    Ok((function_name.to_string(), last_pos))
}

fn receiver_service(
    masked: &str,
    receiver: &str,
    function_start: usize,
    offset: usize,
) -> Result<String, HarnessError> {
    let scope = &masked[function_start..offset];

    // Check for qualified client assignment: let sdk = aws_sdk_<service>::Client::new(...)
    let pattern = format!("{} = aws_sdk_", receiver);
    if let Some(pos) = scope.find(&pattern) {
        let after_start = pos.saturating_add(pattern.len());
        let after = &scope[after_start..];
        let service_end = after
            .find(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_')
            .unwrap_or(after.len());
        return Ok(after[..service_end].to_string());
    }

    // Check for type-annotated parameter: fn foo(sdk: &aws_sdk_<service>::Client)
    let pattern2 = format!("{}: &aws_sdk_", receiver);
    if let Some(pos) = scope.find(&pattern2) {
        let after_start = pos.saturating_add(pattern2.len());
        let after = &scope[after_start..];
        let service_end = after
            .find(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_')
            .unwrap_or(after.len());
        return Ok(after[..service_end].to_string());
    }

    // Check for import: use aws_sdk_<service>::Client
    let use_pattern = "use aws_sdk_";
    let mut imports: Vec<String> = Vec::new();
    for use_match in masked.match_indices(use_pattern) {
        let after_start = use_match.0.saturating_add(use_pattern.len());
        let after = &masked[after_start..];
        let service_end = after
            .find(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_')
            .unwrap_or(64);
        let service = &after[..service_end];
        imports.push(service.to_string());
    }

    // Deduplicate imports
    imports.sort();
    imports.dedup();

    match imports.len() {
        1 => Ok(imports.remove(0)),
        0 => Err(HarnessError::invalid(format!(
            "cannot uniquely resolve SDK client receiver {}",
            receiver
        ))),
        _ => Err(HarnessError::invalid(format!(
            "cannot uniquely resolve SDK client receiver {}",
            receiver
        ))),
    }
}

pub fn calls_in_source(path: &str, source: &str) -> Result<(Vec<Callsite>, usize), HarnessError> {
    calls_in_source_with_budget(path, source, lexical::MAX_TOKENS)
}

pub fn calls_in_source_with_budget(
    path: &str,
    source: &str,
    max_tokens: usize,
) -> Result<(Vec<Callsite>, usize), HarnessError> {
    let masked = mask::mask_noncode(path, source)?;
    let (tokens, pairs) = lexical::tokens_and_pairs_with_budget(path, &masked, max_tokens)?;
    let mut calls: Vec<Callsite> = Vec::new();
    let mut nonempty_sends: usize = 0;

    let mut index = 0;
    let limit = tokens.len().saturating_sub(2);
    while index < limit {
        let current = match tokens.get(index) {
            Some(t) => t,
            None => break,
        };
        let next_index = index.saturating_add(1);
        let send_index = index.saturating_add(2);
        let next = match tokens.get(next_index) {
            Some(t) => t,
            None => break,
        };
        let open = match tokens.get(send_index) {
            Some(t) => t,
            None => break,
        };
        if current.text == "." && next.text == "send" && open.text == "(" {
            let open_pair = send_index.saturating_add(1);
            if *pairs.get(&send_index).unwrap_or(&send_index) != open_pair {
                nonempty_sends = nonempty_sends.saturating_add(1);
                index = index.saturating_add(1);
                continue;
            }

            // Walk backwards through the method call chain
            // Python: cursor = index - 1, walk while token is ")"
            let mut cursor = index.saturating_sub(1);
            let mut methods: Vec<(String, usize)> = Vec::new();
            while let Some(tok) = tokens.get(cursor) {
                if tok.text != ")" {
                    break;
                }
                let open = pairs.get(&cursor).copied().unwrap_or(0);
                if open < 2 {
                    break;
                }
                let method_index = open.saturating_sub(1);
                let method_token = match tokens.get(method_index) {
                    Some(t) => t,
                    None => break,
                };
                let method_name = &method_token.text;
                let prev_index = open.saturating_sub(2);
                let prev_token = match tokens.get(prev_index) {
                    Some(t) => t,
                    None => break,
                };
                if prev_token.text != "." {
                    break;
                }
                if !method_name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    break;
                }
                methods.push((method_name.clone(), open));
                cursor = open.saturating_sub(3);
            }

            // Python uses methods[-1] (last method found)
            let (operation, last_open) = match methods.last() {
                Some((m, o)) => (m.clone(), *o),
                None => {
                    let tok = match tokens.get(index) {
                        Some(t) => t,
                        None => break,
                    };
                    return Err(source_error(
                        path,
                        source,
                        tok.start,
                        "unrecognized empty send call chain",
                    ));
                }
            };
            let op_index = last_open.saturating_sub(1);
            let op_token = match tokens.get(op_index) {
                Some(t) => t,
                None => break,
            };
            let operation_offset = op_token.start;
            let cursor_token = match tokens.get(cursor) {
                Some(t) => t,
                None => break,
            };
            let receiver = cursor_token.text.clone();

            let index_token = match tokens.get(index) {
                Some(t) => t,
                None => break,
            };
            let (function, function_start) = function_at(&masked, index_token.start)?;
            let service =
                receiver_service(&masked, &receiver, function_start, index_token.start)
                    .map_err(|e| source_error(path, source, index_token.start, &e.to_string()))?;
            let operation_line = line_column(source, operation_offset).0;
            let next_index = index.saturating_add(1);
            let next_token = match tokens.get(next_index) {
                Some(t) => t,
                None => break,
            };
            let send_line = line_column(source, next_token.start).0;

            calls.push(Callsite {
                id: format!("{}.{}", service, operation),
                path: path.to_string(),
                function,
                sdk_service: service,
                sdk_operation: operation,
                receiver,
                operation_line,
                send_line,
            });

            if calls.len() > MAX_CALLS {
                return Err(HarnessError::invalid("SDK send call budget exceeded"));
            }
        }
        index = index.saturating_add(1);
    }

    Ok((calls, nonempty_sends))
}

fn walk_directory(
    root: &std::path::Path,
    dir: &std::path::Path,
    sources: &mut HashMap<String, String>,
    total: &mut usize,
) -> Result<(), HarnessError> {
    let entries = dir.read_dir().map_err(|e| {
        HarnessError::invalid(format!("failed to read directory {}: {}", dir.display(), e))
    })?;

    for entry in entries {
        let entry =
            entry.map_err(|e| HarnessError::invalid(format!("directory entry error: {}", e)))?;
        let path = entry.path();

        if path.is_symlink() {
            return Err(HarnessError::invalid(format!(
                "AWS source inventory found a symlink: {}",
                path.display()
            )));
        }

        if path.is_dir() {
            walk_directory(root, &path, sources, total)?;
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let rel_str = rel.to_string_lossy().to_string();

            if sources.len() >= MAX_FILES {
                return Err(HarnessError::invalid("unsupported AWS source inventory"));
            }

            let content = std::fs::read_to_string(&path)
                .map_err(|e| HarnessError::invalid(format!("failed to read {}: {}", rel_str, e)))?;
            *total = total.saturating_add(content.len());
            if *total > MAX_SOURCE_BYTES {
                return Err(HarnessError::invalid("AWS source byte budget exceeded"));
            }
            sources.insert(rel_str, content);
        }
    }

    Ok(())
}

pub fn source_inventory(root: &std::path::Path) -> SourceInventoryResult {
    let mut sources: HashMap<String, String> = HashMap::new();
    let mut total: usize = 0;
    let aws_dir = root.join("crates/aws/src");

    if !aws_dir.exists() {
        return Ok((sources, Vec::new(), 0));
    }

    walk_directory(root, &aws_dir, &mut sources, &mut total)?;

    let mut calls: Vec<Callsite> = Vec::new();
    let mut nonempty: usize = 0;
    for (path, content) in &sources {
        let (found, skipped) = calls_in_source(path, content)?;
        calls.extend(found);
        nonempty = nonempty.saturating_add(skipped);
    }

    Ok((sources, calls, nonempty))
}
