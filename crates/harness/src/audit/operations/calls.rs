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
    let scan = SendScan {
        tokens: &tokens,
        pairs: &pairs,
        masked: &masked,
        path,
        source,
    };
    let mut calls: Vec<Callsite> = Vec::new();
    let mut nonempty_sends: usize = 0;

    let mut index = 0;
    let limit = tokens.len().saturating_sub(2);
    while index < limit {
        match scan.step(index)? {
            Step::Stop => break,
            Step::Skip => {}
            Step::NonEmpty => nonempty_sends = nonempty_sends.saturating_add(1),
            Step::Call(call) => {
                calls.push(call);
                if calls.len() > MAX_CALLS {
                    return Err(HarnessError::invalid("SDK send call budget exceeded"));
                }
            }
        }
        index = index.saturating_add(1);
    }

    Ok((calls, nonempty_sends))
}

struct SendScan<'a> {
    tokens: &'a [lexical::Token],
    pairs: &'a HashMap<usize, usize>,
    masked: &'a str,
    path: &'a str,
    source: &'a str,
}

/// Outcome of inspecting the token triple that starts at one index.
enum Step {
    /// The token stream ended; the scan returns what it has collected.
    Stop,
    /// Not an empty `.send()` call.
    Skip,
    /// A `.send(...)` call with arguments.
    NonEmpty,
    Call(Callsite),
}

impl<'a> SendScan<'a> {
    fn step(&self, index: usize) -> Result<Step, HarnessError> {
        let send_index = index.saturating_add(2);
        let (Some(current), Some(next), Some(open)) = (
            self.tokens.get(index),
            self.tokens.get(index.saturating_add(1)),
            self.tokens.get(send_index),
        ) else {
            return Ok(Step::Stop);
        };
        if current.text != "." || next.text != "send" || open.text != "(" {
            return Ok(Step::Skip);
        }
        let open_pair = send_index.saturating_add(1);
        if *self.pairs.get(&send_index).unwrap_or(&send_index) != open_pair {
            return Ok(Step::NonEmpty);
        }

        let (last_method, cursor) = self.method_chain(index);
        // Python uses methods[-1] (last method found)
        let Some((operation, last_open)) = last_method else {
            return Err(source_error(
                self.path,
                self.source,
                current.start,
                "unrecognized empty send call chain",
            ));
        };
        let (Some(op_token), Some(cursor_token)) = (
            self.tokens.get(last_open.saturating_sub(1)),
            self.tokens.get(cursor),
        ) else {
            return Ok(Step::Stop);
        };
        let receiver = cursor_token.text.clone();

        let (function, function_start) = function_at(self.masked, current.start)?;
        let service = receiver_service(self.masked, &receiver, function_start, current.start)
            .map_err(|e| source_error(self.path, self.source, current.start, &e.to_string()))?;
        Ok(Step::Call(Callsite {
            id: format!("{}.{}", service, operation),
            path: self.path.to_string(),
            function,
            sdk_service: service,
            sdk_operation: operation.to_string(),
            receiver,
            operation_line: line_column(self.source, op_token.start).0,
            send_line: line_column(self.source, next.start).0,
        }))
    }

    /// Returns the last method found with its opening delimiter, and the receiver cursor.
    fn method_chain(&self, index: usize) -> (Option<(&'a str, usize)>, usize) {
        // Walk backwards through the method call chain
        // Python: cursor = index - 1, walk while token is ")"
        let mut cursor = index.saturating_sub(1);
        let mut last = None;
        while let Some(tok) = self.tokens.get(cursor) {
            if tok.text != ")" {
                break;
            }
            let open = self.pairs.get(&cursor).copied().unwrap_or(0);
            if open < 2 {
                break;
            }
            let (Some(method), Some(prev)) = (
                self.tokens.get(open.saturating_sub(1)),
                self.tokens.get(open.saturating_sub(2)),
            ) else {
                break;
            };
            if prev.text != "."
                || !method
                    .text
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                break;
            }
            last = Some((method.text.as_str(), open));
            cursor = open.saturating_sub(3);
        }
        (last, cursor)
    }
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
