use crate::errors::{CompilerError, SourceLocation};
use crate::lexer::{Token, tokenize_with_locations};
use crate::parser::parse_with_locations;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead, BufReader, BufWriter, Write};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct LspPosition {
    line: u32,
    character: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct LspRange {
    start: LspPosition,
    end: LspPosition,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct LspDiagnostic {
    range: LspRange,
    severity: u32,
    source: String,
    message: String,
}

#[derive(Debug, Clone)]
struct IndexedSymbol {
    name: String,
    kind: u32,
    completion_kind: u32,
    detail: String,
    documentation: String,
    range: LspRange,
}

#[derive(Debug, Clone)]
struct DocumentState {
    text: String,
    symbols: Vec<IndexedSymbol>,
}

pub fn run_language_server() -> Result<(), String> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());
    let mut documents = HashMap::<String, DocumentState>::new();
    let mut shutdown_requested = false;

    loop {
        let message = match read_message(&mut reader) {
            Ok(Some(msg)) => msg,
            Ok(None) => break,
            Err(err) => return Err(format!("LSP read error: {}", err)),
        };

        let method = message.get("method").and_then(Value::as_str);
        let id = message.get("id").cloned();

        if let Some(method) = method {
            match method {
                "initialize" => {
                    if let Some(id) = id {
                        let result = json!({
                            "capabilities": {
                                "textDocumentSync": 1,
                                "completionProvider": {
                                    "resolveProvider": false
                                },
                                "hoverProvider": true,
                                "definitionProvider": true,
                                "documentSymbolProvider": true
                            }
                        });
                        write_response(&mut writer, id, result)?;
                    }
                }
                "initialized" => {}
                "shutdown" => {
                    shutdown_requested = true;
                    if let Some(id) = id {
                        write_response(&mut writer, id, Value::Null)?;
                    }
                }
                "exit" => {
                    if shutdown_requested {
                        return Ok(());
                    }
                    return Err("received exit before shutdown".to_string());
                }
                "textDocument/didOpen" => {
                    let uri = uri_from_pointer(&message, "/params/textDocument/uri");
                    let text = message
                        .pointer("/params/textDocument/text")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();

                    let state = index_document(&uri, &text);
                    let diagnostics = syntax_diagnostics(&text, Some(uri.clone()));
                    publish_diagnostics(&mut writer, &uri, diagnostics)?;
                    documents.insert(uri, state);
                }
                "textDocument/didChange" => {
                    let uri = uri_from_pointer(&message, "/params/textDocument/uri");

                    if let Some(text) = message
                        .pointer("/params/contentChanges")
                        .and_then(Value::as_array)
                        .and_then(|changes| changes.last())
                        .and_then(|entry| entry.get("text"))
                        .and_then(Value::as_str)
                    {
                        let state = index_document(&uri, text);
                        let diagnostics = syntax_diagnostics(text, Some(uri.clone()));
                        publish_diagnostics(&mut writer, &uri, diagnostics)?;
                        documents.insert(uri, state);
                    }
                }
                "textDocument/didClose" => {
                    let uri = uri_from_pointer(&message, "/params/textDocument/uri");
                    documents.remove(&uri);
                    publish_diagnostics(&mut writer, &uri, Vec::new())?;
                }
                "textDocument/completion" => {
                    if let Some(id) = id {
                        let uri = uri_from_pointer(&message, "/params/textDocument/uri");
                        let position = position_from_message(&message);
                        let items = completion_items(&uri, position.as_ref(), &documents);
                        write_response(
                            &mut writer,
                            id,
                            json!({
                                "isIncomplete": false,
                                "items": items
                            }),
                        )?;
                    }
                }
                "textDocument/hover" => {
                    if let Some(id) = id {
                        let uri = uri_from_pointer(&message, "/params/textDocument/uri");
                        let result = if let Some(position) = position_from_message(&message) {
                            hover_result(&uri, &position, &documents)
                        } else {
                            None
                        };
                        write_response(&mut writer, id, result.unwrap_or(Value::Null))?;
                    }
                }
                "textDocument/definition" => {
                    if let Some(id) = id {
                        let uri = uri_from_pointer(&message, "/params/textDocument/uri");
                        let result = if let Some(position) = position_from_message(&message) {
                            definition_result(&uri, &position, &documents)
                        } else {
                            None
                        };
                        write_response(&mut writer, id, result.unwrap_or(Value::Null))?;
                    }
                }
                "textDocument/documentSymbol" => {
                    if let Some(id) = id {
                        let uri = uri_from_pointer(&message, "/params/textDocument/uri");
                        let result = document_symbols_result(&uri, &documents);
                        write_response(&mut writer, id, result)?;
                    }
                }
                _ => {
                    if let Some(id) = id {
                        write_response(
                            &mut writer,
                            id,
                            json!({
                                "error": {
                                    "code": -32601,
                                    "message": format!("Method not found: {}", method)
                                }
                            }),
                        )?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn uri_from_pointer(message: &Value, pointer: &str) -> String {
    message
        .pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn position_from_message(message: &Value) -> Option<LspPosition> {
    let line = message.pointer("/params/position/line")?.as_u64()? as u32;
    let character = message.pointer("/params/position/character")?.as_u64()? as u32;
    Some(LspPosition { line, character })
}

fn index_document(uri: &str, text: &str) -> DocumentState {
    DocumentState {
        text: text.to_string(),
        symbols: index_symbols(text, Some(uri.to_string())),
    }
}

fn index_symbols(source: &str, filename: Option<String>) -> Vec<IndexedSymbol> {
    let tokens = tokenize_with_locations(source, filename);
    let mut symbols = Vec::new();
    let mut i = 0usize;

    while i < tokens.len() {
        match &tokens[i].token {
            Token::Fn => {
                if let Some((name, location)) = identifier_after(&tokens, i + 1) {
                    symbols.push(make_symbol(
                        name,
                        12,
                        3,
                        format!("fn {}", name),
                        "Function declaration.",
                        location,
                    ));
                }
            }
            Token::Struct => {
                if let Some((name, location)) = identifier_after(&tokens, i + 1) {
                    symbols.push(make_symbol(
                        name,
                        23,
                        22,
                        format!("struct {}", name),
                        "Struct declaration.",
                        location,
                    ));
                }
            }
            Token::Enum => {
                if let Some((name, location)) = identifier_after(&tokens, i + 1) {
                    symbols.push(make_symbol(
                        name,
                        10,
                        13,
                        format!("enum {}", name),
                        "Enum declaration.",
                        location,
                    ));
                }
            }
            Token::Trait => {
                if let Some((name, location)) = identifier_after(&tokens, i + 1) {
                    symbols.push(make_symbol(
                        name,
                        11,
                        8,
                        format!("trait {}", name),
                        "Trait declaration.",
                        location,
                    ));
                }
            }
            Token::Mod => {
                if let Some((name, location)) = identifier_after(&tokens, i + 1) {
                    symbols.push(make_symbol(
                        name,
                        2,
                        9,
                        format!("mod {}", name),
                        "Module declaration.",
                        location,
                    ));
                }
            }
            Token::Let => {
                let mut j = i + 1;
                if matches!(tokens.get(j).map(|t| &t.token), Some(Token::Mut)) {
                    j += 1;
                }
                if let Some((name, location)) = identifier_after(&tokens, j) {
                    symbols.push(make_symbol(
                        name,
                        13,
                        6,
                        format!("let {}", name),
                        "Variable binding.",
                        location,
                    ));
                }
            }
            _ => {}
        }

        i += 1;
    }

    symbols
}

fn identifier_after(
    tokens: &[crate::lexer::LocatedToken],
    start: usize,
) -> Option<(&str, &SourceLocation)> {
    match tokens.get(start).map(|t| &t.token) {
        Some(Token::Identifier(name)) => Some((name.as_str(), &tokens[start].location)),
        _ => None,
    }
}

fn make_symbol(
    name: &str,
    kind: u32,
    completion_kind: u32,
    detail: String,
    documentation: &str,
    location: &SourceLocation,
) -> IndexedSymbol {
    IndexedSymbol {
        name: name.to_string(),
        kind,
        completion_kind,
        detail,
        documentation: documentation.to_string(),
        range: range_from_location(location, name.len()),
    }
}

fn syntax_diagnostics(source: &str, filename: Option<String>) -> Vec<LspDiagnostic> {
    let tokens = tokenize_with_locations(source, filename);
    match parse_with_locations(tokens) {
        Ok(_) => Vec::new(),
        Err(err) => diagnostics_from_error(&err),
    }
}

fn diagnostics_from_error(error: &CompilerError) -> Vec<LspDiagnostic> {
    match error {
        CompilerError::MultiError { errors } => errors
            .iter()
            .flat_map(diagnostics_from_error)
            .collect::<Vec<_>>(),
        single => vec![diagnostic_for_single_error(single)],
    }
}

fn diagnostic_for_single_error(error: &CompilerError) -> LspDiagnostic {
    let location = error
        .location()
        .cloned()
        .unwrap_or_else(SourceLocation::unknown);
    let line = location.line.saturating_sub(1) as u32;
    let column = location.column.saturating_sub(1) as u32;

    LspDiagnostic {
        range: LspRange {
            start: LspPosition {
                line,
                character: column,
            },
            end: LspPosition {
                line,
                character: column.saturating_add(1),
            },
        },
        severity: 1,
        source: "aero-parser".to_string(),
        message: error.to_string(),
    }
}

fn completion_items(
    uri: &str,
    position: Option<&LspPosition>,
    documents: &HashMap<String, DocumentState>,
) -> Vec<Value> {
    let prefix = documents
        .get(uri)
        .and_then(|doc| position.and_then(|pos| identifier_prefix(&doc.text, pos)))
        .unwrap_or_default();

    let mut items = Vec::new();
    let mut seen = HashSet::new();

    for keyword in [
        "let", "mut", "fn", "return", "if", "else", "while", "for", "in", "loop", "break",
        "continue", "struct", "enum", "trait", "impl", "match", "mod", "use", "pub", "where",
    ] {
        push_completion_item(&mut items, &mut seen, keyword, 14, "keyword", &prefix);
    }

    push_completion_item(
        &mut items,
        &mut seen,
        "println!",
        3,
        "builtin macro",
        &prefix,
    );
    push_completion_item(&mut items, &mut seen, "print!", 3, "builtin macro", &prefix);
    push_completion_item(&mut items, &mut seen, "vec!", 15, "builtin macro", &prefix);

    for state in documents.values() {
        for symbol in &state.symbols {
            push_completion_item(
                &mut items,
                &mut seen,
                &symbol.name,
                symbol.completion_kind,
                &symbol.detail,
                &prefix,
            );
        }
    }

    items.sort_by_key(|item| {
        item.get("label")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    });
    items.truncate(200);
    items
}

fn push_completion_item(
    items: &mut Vec<Value>,
    seen: &mut HashSet<String>,
    label: &str,
    kind: u32,
    detail: &str,
    prefix: &str,
) {
    if !prefix.is_empty() && !label.starts_with(prefix) {
        return;
    }

    if !seen.insert(label.to_string()) {
        return;
    }

    items.push(json!({
        "label": label,
        "kind": kind,
        "detail": detail
    }));
}

fn hover_result(
    uri: &str,
    position: &LspPosition,
    documents: &HashMap<String, DocumentState>,
) -> Option<Value> {
    let document = documents.get(uri)?;
    let word = word_at_position(&document.text, position)?;

    if let Some((_, symbol)) = find_symbol_by_name(&word, uri, documents) {
        return Some(json!({
            "contents": {
                "kind": "markdown",
                "value": format!("```aero\n{}\n```\n\n{}", symbol.detail, symbol.documentation)
            },
            "range": symbol.range
        }));
    }

    if let Some(builtin) = builtin_hover(&word) {
        return Some(json!({
            "contents": {
                "kind": "markdown",
                "value": builtin
            }
        }));
    }

    None
}

fn definition_result(
    uri: &str,
    position: &LspPosition,
    documents: &HashMap<String, DocumentState>,
) -> Option<Value> {
    let document = documents.get(uri)?;
    let word = word_at_position(&document.text, position)?;

    if let Some((def_uri, symbol)) = find_symbol_by_name(&word, uri, documents) {
        return Some(json!([
            {
                "uri": def_uri,
                "range": symbol.range
            }
        ]));
    }

    None
}

fn document_symbols_result(uri: &str, documents: &HashMap<String, DocumentState>) -> Value {
    let Some(document) = documents.get(uri) else {
        return json!([]);
    };

    let symbols = document
        .symbols
        .iter()
        .map(|symbol| {
            json!({
                "name": symbol.name,
                "kind": symbol.kind,
                "detail": symbol.detail,
                "range": symbol.range,
                "selectionRange": symbol.range
            })
        })
        .collect::<Vec<_>>();

    json!(symbols)
}

fn find_symbol_by_name(
    name: &str,
    preferred_uri: &str,
    documents: &HashMap<String, DocumentState>,
) -> Option<(String, IndexedSymbol)> {
    if let Some(local) = documents
        .get(preferred_uri)
        .and_then(|state| best_symbol_match(name, &state.symbols))
    {
        return Some((preferred_uri.to_string(), local.clone()));
    }

    let mut candidates = Vec::new();
    for (uri, state) in documents {
        if uri == preferred_uri {
            continue;
        }
        if let Some(symbol) = best_symbol_match(name, &state.symbols) {
            candidates.push((uri.clone(), symbol.clone()));
        }
    }

    candidates.sort_by_key(|(_, symbol)| symbol_priority(symbol));
    candidates.into_iter().next()
}

fn best_symbol_match<'a>(name: &str, symbols: &'a [IndexedSymbol]) -> Option<&'a IndexedSymbol> {
    symbols
        .iter()
        .filter(|symbol| symbol.name == name)
        .min_by_key(|symbol| symbol_priority(symbol))
}

fn symbol_priority(symbol: &IndexedSymbol) -> u8 {
    match symbol.kind {
        12 => 0,
        23 => 1,
        10 => 2,
        11 => 3,
        2 => 4,
        13 => 5,
        _ => 6,
    }
}

fn builtin_hover(name: &str) -> Option<&'static str> {
    match name {
        "println" => Some("`println!` prints formatted output with a trailing newline."),
        "print" => Some("`print!` prints formatted output without a trailing newline."),
        "vec" => Some("`vec![]` constructs a dynamic array value."),
        _ => None,
    }
}

fn word_at_position(text: &str, position: &LspPosition) -> Option<String> {
    let line = text.lines().nth(position.line as usize)?;
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return None;
    }

    let mut idx = (position.character as usize).min(chars.len().saturating_sub(1));
    if !is_identifier_char(chars[idx]) {
        if idx > 0 && is_identifier_char(chars[idx - 1]) {
            idx -= 1;
        } else {
            return None;
        }
    }

    let mut start = idx;
    while start > 0 && is_identifier_char(chars[start - 1]) {
        start -= 1;
    }

    let mut end = idx + 1;
    while end < chars.len() && is_identifier_char(chars[end]) {
        end += 1;
    }

    let ident: String = chars[start..end].iter().collect();
    if ident.is_empty() { None } else { Some(ident) }
}

fn identifier_prefix(text: &str, position: &LspPosition) -> Option<String> {
    let line = text.lines().nth(position.line as usize)?;
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return Some(String::new());
    }

    let mut idx = (position.character as usize).min(chars.len());
    while idx > 0 && !is_identifier_char(chars[idx - 1]) {
        if idx == 0 {
            break;
        }
        if idx == chars.len() {
            break;
        }
        idx -= 1;
    }

    let mut start = idx;
    while start > 0 && is_identifier_char(chars[start - 1]) {
        start -= 1;
    }

    let prefix: String = chars[start..idx].iter().collect();
    Some(prefix)
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn range_from_location(location: &SourceLocation, len: usize) -> LspRange {
    let line = location.line.saturating_sub(1) as u32;
    let start_col = location.column.saturating_sub(1) as u32;
    let length = len.max(1) as u32;

    LspRange {
        start: LspPosition {
            line,
            character: start_col,
        },
        end: LspPosition {
            line,
            character: start_col.saturating_add(length),
        },
    }
}

fn publish_diagnostics(
    writer: &mut dyn Write,
    uri: &str,
    diagnostics: Vec<LspDiagnostic>,
) -> Result<(), String> {
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": diagnostics
        }
    });
    write_message(writer, &notification)
}

fn write_response(writer: &mut dyn Write, id: Value, result: Value) -> Result<(), String> {
    if result.get("error").is_some() {
        write_message(
            writer,
            &json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": result["error"].clone()
            }),
        )
    } else {
        write_message(
            writer,
            &json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            }),
        )
    }
}

fn write_message(writer: &mut dyn Write, payload: &Value) -> Result<(), String> {
    let body = serde_json::to_vec(payload).map_err(|err| format!("serialize error: {}", err))?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    writer
        .write_all(header.as_bytes())
        .map_err(|err| format!("write header error: {}", err))?;
    writer
        .write_all(&body)
        .map_err(|err| format!("write body error: {}", err))?;
    writer
        .flush()
        .map_err(|err| format!("flush error: {}", err))
}

fn read_message(reader: &mut dyn BufRead) -> Result<Option<Value>, String> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let bytes = reader
            .read_line(&mut line)
            .map_err(|err| format!("failed to read header: {}", err))?;
        if bytes == 0 {
            return Ok(None);
        }

        if line == "\r\n" || line == "\n" {
            break;
        }

        if let Some((name, value)) = line.split_once(':')
            && name.trim().eq_ignore_ascii_case("Content-Length")
        {
            let parsed = value
                .trim()
                .parse::<usize>()
                .map_err(|err| format!("invalid Content-Length: {}", err))?;
            content_length = Some(parsed);
        }
    }

    let length = content_length.ok_or_else(|| "missing Content-Length header".to_string())?;
    let mut body = vec![0_u8; length];
    reader
        .read_exact(&mut body)
        .map_err(|err| format!("failed to read body: {}", err))?;

    let message: Value =
        serde_json::from_slice(&body).map_err(|err| format!("invalid JSON payload: {}", err))?;
    Ok(Some(message))
}

trait ErrorLocation {
    fn location(&self) -> Option<&SourceLocation>;
}

impl ErrorLocation for CompilerError {
    fn location(&self) -> Option<&SourceLocation> {
        match self {
            CompilerError::UnexpectedCharacter { location, .. }
            | CompilerError::UnterminatedString { location }
            | CompilerError::InvalidNumber { location, .. }
            | CompilerError::UnexpectedToken { location, .. }
            | CompilerError::UnexpectedEndOfInput { location, .. }
            | CompilerError::InvalidSyntax { location, .. }
            | CompilerError::FunctionRedefinition { location, .. }
            | CompilerError::UndefinedFunction { location, .. }
            | CompilerError::ArityMismatch { location, .. }
            | CompilerError::ParameterTypeMismatch { location, .. }
            | CompilerError::ReturnTypeMismatch { location, .. }
            | CompilerError::BreakOutsideLoop { location }
            | CompilerError::ContinueOutsideLoop { location }
            | CompilerError::UnreachableCode { location }
            | CompilerError::InvalidConditionType { location, .. }
            | CompilerError::UndefinedVariable { location, .. }
            | CompilerError::VariableRedefinition { location, .. }
            | CompilerError::ImmutableAssignment { location, .. }
            | CompilerError::UninitializedVariable { location, .. }
            | CompilerError::TypeMismatch { location, .. }
            | CompilerError::IncompatibleTypes { location, .. }
            | CompilerError::InvalidTypeAnnotation { location, .. }
            | CompilerError::InvalidFormatString { location, .. }
            | CompilerError::FormatArgumentMismatch { location, .. }
            | CompilerError::InvalidFormatSpecifier { location, .. }
            | CompilerError::InvalidOperation { location, .. }
            | CompilerError::ScopeError { location, .. } => Some(location),
            CompilerError::MultiError { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_from_single_error_map_line_and_column_to_zero_based() {
        let error = CompilerError::UnexpectedToken {
            expected: "identifier".to_string(),
            found: "Semicolon".to_string(),
            location: SourceLocation::new(3, 5),
        };
        let diagnostics = diagnostics_from_error(&error);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range.start.line, 2);
        assert_eq!(diagnostics[0].range.start.character, 4);
        assert_eq!(diagnostics[0].range.end.character, 5);
    }

    #[test]
    fn diagnostics_flatten_multi_error() {
        let first = CompilerError::UnexpectedToken {
            expected: "identifier".to_string(),
            found: "Semicolon".to_string(),
            location: SourceLocation::new(1, 1),
        };
        let second = CompilerError::InvalidSyntax {
            message: "bad statement".to_string(),
            location: SourceLocation::new(2, 3),
        };
        let error = CompilerError::MultiError {
            errors: vec![first, second],
        };
        let diagnostics = diagnostics_from_error(&error);
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn syntax_diagnostics_is_empty_for_valid_program() {
        let source = "let x = 1;";
        let diagnostics = syntax_diagnostics(source, None);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn symbol_index_collects_common_declarations() {
        let source = "mod math; struct Vec2 { x: i32 } enum Color { Red } trait Draw { fn draw(self: Self); } fn add(x: i32, y: i32) -> i32 { let sum = x + y; return sum; }";
        let symbols = index_symbols(source, Some("file:///tmp/main.aero".to_string()));
        let names = symbols.into_iter().map(|s| s.name).collect::<Vec<_>>();
        assert!(names.contains(&"math".to_string()));
        assert!(names.contains(&"Vec2".to_string()));
        assert!(names.contains(&"Color".to_string()));
        assert!(names.contains(&"Draw".to_string()));
        assert!(names.contains(&"add".to_string()));
        assert!(names.contains(&"sum".to_string()));
    }

    #[test]
    fn completion_includes_keywords_and_document_symbols() {
        let uri = "file:///tmp/main.aero".to_string();
        let mut docs = HashMap::new();
        docs.insert(
            uri.clone(),
            index_document(&uri, "fn add(x: i32) -> i32 { x }\nlet count = 1;\nco"),
        );

        let items = completion_items(
            &uri,
            Some(&LspPosition {
                line: 2,
                character: 2,
            }),
            &docs,
        );

        let labels = items
            .iter()
            .filter_map(|item| item.get("label").and_then(Value::as_str))
            .collect::<Vec<_>>();

        assert!(labels.contains(&"continue"));
        assert!(labels.contains(&"count"));
    }

    #[test]
    fn hover_and_definition_resolve_function_symbol() {
        let uri = "file:///tmp/main.aero".to_string();
        let mut docs = HashMap::new();
        let source = "fn add(x: i32, y: i32) -> i32 { x + y; }\nfn main() { add(1, 2); }";
        docs.insert(uri.clone(), index_document(&uri, source));

        let usage = LspPosition {
            line: 1,
            character: 14,
        };

        let hover = hover_result(&uri, &usage, &docs).expect("hover should resolve");
        let hover_text = hover
            .pointer("/contents/value")
            .and_then(Value::as_str)
            .expect("hover markdown");
        assert!(hover_text.contains("fn add"));

        let definition = definition_result(&uri, &usage, &docs).expect("definition should resolve");
        let def_line = definition
            .pointer("/0/range/start/line")
            .and_then(Value::as_u64)
            .expect("definition line");
        assert_eq!(def_line, 0);
    }

    #[test]
    fn document_symbols_returns_flat_symbol_array() {
        let uri = "file:///tmp/main.aero".to_string();
        let mut docs = HashMap::new();
        docs.insert(uri.clone(), index_document(&uri, "fn add() {}\nlet x = 1;"));

        let result = document_symbols_result(&uri, &docs);
        let arr = result.as_array().expect("document symbols array");
        assert!(arr.len() >= 2);
    }
}
