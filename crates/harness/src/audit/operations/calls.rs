use std::collections::HashMap;

use crate::HarnessError;

use super::lexical;

const MAX_FILES: usize = 4_096;
const MAX_SOURCE_BYTES: usize = 32 * 1024 * 1024;
const MAX_CALLS: usize = 4_096;
const SOURCE_PREFIX: &str = "crates/aws/src/";

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
    let line = before.matches('\n').count() + 1;
    let column = offset - before.rfind('\n').unwrap_or(0);
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
        let name_start = pos + 3;
        if name_start >= masked.len() {
            continue;
        }
        let rest = &masked[name_start..];
        if let Some(first) = rest.as_bytes().first() {
            if first.is_ascii_alphabetic() || *first == b'_' {
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

    let name_start = last_pos + 3;
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
        let after = &scope[pos + pattern.len()..];
        let service_end = after
            .find(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_')
            .unwrap_or(after.len());
        return Ok(after[..service_end].to_string());
    }

    // Check for type-annotated parameter: fn foo(sdk: &aws_sdk_<service>::Client)
    let pattern2 = format!("{}: &aws_sdk_", receiver);
    if let Some(pos) = scope.find(&pattern2) {
        let after = &scope[pos + pattern2.len()..];
        let service_end = after
            .find(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_')
            .unwrap_or(after.len());
        return Ok(after[..service_end].to_string());
    }

    // Check for import: use aws_sdk_<service>::Client
    let use_pattern = "use aws_sdk_";
    let mut imports: Vec<String> = Vec::new();
    for use_match in masked.match_indices(use_pattern) {
        let after = &masked[use_match.0 + use_pattern.len()..];
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

pub fn calls_in_source_with_budget(path: &str, source: &str, max_tokens: usize) -> Result<(Vec<Callsite>, usize), HarnessError> {
    let masked = lexical::mask_noncode(path, source)?;
    let (tokens, pairs) = lexical::tokens_and_pairs_with_budget(path, &masked, max_tokens)?;
    let mut calls: Vec<Callsite> = Vec::new();
    let mut nonempty_sends: usize = 0;

    let mut index = 0;
    while index < tokens.len() - 2 {
        if tokens[index].text == "." && tokens[index + 1].text == "send" && tokens[index + 2].text == "(" {
            if *pairs.get(&(index + 2)).unwrap_or(&index) != index + 3 {
                nonempty_sends += 1;
                index += 1;
                continue;
            }

            let mut cursor = index - 1;
            let mut methods: Vec<String> = Vec::new();
            let mut last_open = 0;
            loop {
                if cursor < 0 {
                    break;
                }
                let tok = &tokens[cursor];
                if tok.text != ")" {
                    break;
                }
                let open = *pairs.get(&cursor).unwrap_or(&0);
                if open < 2 {
                    break;
                }
                let method_name = &tokens[open - 1].text;
                if tokens[open - 2].text != "." {
                    break;
                }
                if !method_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    break;
                }
                last_open = open;
                methods.push(method_name.clone());
                let new_cursor = open - 3;
                eprintln!("DEBUG LOOP: collected {}, cursor {} -> {}", method_name, cursor, new_cursor);
                if new_cursor >= 0 && new_cursor < tokens.len() {
                    eprintln!("DEBUG LOOP: tokens[new_cursor] = {}", tokens[new_cursor].text);
                }
                cursor = new_cursor;
            }
            eprintln!("DEBUG: collected methods: {:?}", methods);
            eprintln!("DEBUG: cursor: {}", cursor);
            eprintln!("DEBUG: index: {}", index);
            eprintln!("DEBUG: index - 1: {}", index - 1);
            if index - 1 >= 0 {
                eprintln!("DEBUG: tokens[index - 1]: {}", tokens[index - 1].text);
            }

            if methods.is_empty() || cursor < 0 {
                return Err(source_error(
                    path,
                    source,
                    tokens[index].start,
                    "unrecognized empty send call chain",
                ));
            }

            let operation = methods.last().unwrap().clone();
            let operation_offset = tokens[last_open - 1].start;
            let receiver = tokens[cursor].text.clone();

            let (function, function_start) = function_at(&masked, tokens[index].start)?;
            let service = receiver_service(&masked, &receiver, function_start, tokens[index].start)
                .map_err(|e| source_error(path, source, tokens[index].start, &e.to_string()))?;
            let operation_line = line_column(source, operation_offset).0;
            let send_line = line_column(source, tokens[index + 1].start).0;

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
        index += 1;
    }

    Ok((calls, nonempty_sends))
}

pub fn source_inventory(root: &std::path::Path) -> Result<(HashMap<String, String>, Vec<Callsite>, usize), HarnessError> {
    let mut sources: HashMap<String, String> = HashMap::new();
    let mut total: usize = 0;
    let aws_dir = root.join("crates/aws/src");
    let entries = aws_dir
        .read_dir()
        .map_err(|e| HarnessError::invalid(format!("failed to read AWS source directory: {}", e)))?;

    for entry in entries {
        let entry = entry.map_err(|e| HarnessError::invalid(format!("directory entry error: {}", e)))?;
        if entry.file_name().to_string_lossy().ends_with(".rs") {
            let path_buf = entry.path();
            let rel = path_buf.strip_prefix(root).unwrap_or(&path_buf);
            let rel_str = rel.to_string_lossy().to_string();
            if sources.len() >= MAX_FILES {
                return Err(HarnessError::invalid(
                    "unsupported AWS source inventory",
                ));
            }
            let content = std::fs::read_to_string(&entry.path())
                .map_err(|e| HarnessError::invalid(format!("failed to read {}: {}", rel_str, e)))?;
            total += content.len();
            if total > MAX_SOURCE_BYTES {
                return Err(HarnessError::invalid(
                    "AWS source byte budget exceeded",
                ));
            }
            sources.insert(rel_str, content);
        }
    }

    let mut calls: Vec<Callsite> = Vec::new();
    let mut nonempty: usize = 0;
    for (path, content) in &sources {
        let (found, skipped) = calls_in_source(path, content)?;
        calls.extend(found);
        nonempty += skipped;
    }

    Ok((sources, calls, nonempty))
}