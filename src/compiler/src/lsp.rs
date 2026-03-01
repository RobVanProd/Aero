use crate::errors::{CompilerError, SourceLocation};
use crate::lexer::tokenize_with_locations;
use crate::parser::parse_with_locations;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashMap;
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

pub fn run_language_server() -> Result<(), String> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());
    let mut documents = HashMap::<String, String>::new();
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
                                "textDocumentSync": 1
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
                    let uri = message
                        .pointer("/params/textDocument/uri")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let text = message
                        .pointer("/params/textDocument/text")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();

                    documents.insert(uri.clone(), text.clone());
                    let diagnostics = syntax_diagnostics(&text, Some(uri.clone()));
                    publish_diagnostics(&mut writer, &uri, diagnostics)?;
                }
                "textDocument/didChange" => {
                    let uri = message
                        .pointer("/params/textDocument/uri")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();

                    if let Some(text) = message
                        .pointer("/params/contentChanges")
                        .and_then(Value::as_array)
                        .and_then(|changes| changes.last())
                        .and_then(|entry| entry.get("text"))
                        .and_then(Value::as_str)
                    {
                        documents.insert(uri.clone(), text.to_string());
                    }

                    if let Some(current_text) = documents.get(&uri) {
                        let diagnostics = syntax_diagnostics(current_text, Some(uri.clone()));
                        publish_diagnostics(&mut writer, &uri, diagnostics)?;
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
}
