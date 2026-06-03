//! LSP server that provides ZenScript bracket-handler completions and hover
//! by reading the same `crafttweaker.log` dump format produced by the
//! `zenscript-bracket-completion` VSCode extension.
//!
//! The server is intentionally small. It speaks just enough of LSP 3.17 to
//! handle `initialize`, `initialized`, `textDocument/didOpen`,
//! `textDocument/didChange`, `textDocument/completion`, and
//! `textDocument/hover`, plus `shutdown` and `exit`. It does not talk to the
//! file system beyond the user-configured `crafttweaker.log` path and the
//! optional `additional_path` — Zed handles the rest of the editor surface.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;

use anyhow::Result;
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, RequestId, Response, ResponseError};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
    Hover, HoverContents, HoverParams, InitializeParams, MarkupContent, MarkupKind,
    Position, Range, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind,
};

use crate::parse;

/// User-configurable settings, serialised as LSP `initializationOptions` by
/// the Zed extension wrapper.
#[derive(Debug, Default, Clone)]
pub struct Settings {
    pub ct_log_path: Option<String>,
    pub additional_path: Option<String>,
    pub always_reload: bool,
    pub only_complete_brackets: bool,
    pub completion_suggest_all_items: bool,
    pub completion_suggest_with_start: bool,
}

/// Shared, mutable state for the LSP server.
struct State {
    settings: Settings,
    items: BTreeMap<String, String>,
    /// `URI -> latest full document text` cache, updated on every
    /// `textDocument/didChange` notification.
    documents: HashMap<String, String>,
    last_load_ms: i64,
}

impl State {
    fn new(settings: Settings) -> Self {
        let mut state = Self {
            settings,
            items: BTreeMap::new(),
            documents: HashMap::new(),
            last_load_ms: 0,
        };
        let _ = state.reload();
        state
    }

    fn reload(&mut self) -> Result<()> {
        if let Some(path) = &self.settings.ct_log_path {
            let contents = fs::read_to_string(path)?;
            if let Some(map) = parse::parse_ct_log(&contents) {
                self.items = map;
            } else {
                self.items.clear();
            }
            if let Some(extra) = &self.settings.additional_path {
                if let Ok(contents) = fs::read_to_string(extra) {
                    parse::merge_additional(&mut self.items, &contents);
                }
            }
            self.last_load_ms = millis_now();
        }
        Ok(())
    }

    fn maybe_reload(&mut self) {
        if !self.settings.always_reload {
            return;
        }
        let _ = self.reload();
    }
}

fn millis_now() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Entry point used by the `zsbc-lsp` binary.
pub fn run() -> Result<()> {
    eprintln!("[zsbc-lsp] starting");

    let (connection, io_threads) = Connection::stdio();

    let init_value = connection.initialize(serde_json::to_value(server_capabilities())?)?;
    let init_params: InitializeParams = serde_json::from_value(init_value)?;

    let settings = extract_settings(&init_params);
    let state = Arc::new(Mutex::new(State::new(settings)));

    let result = main_loop(connection, state);
    io_threads.join().expect("io thread join failed");
    result
}

fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::FULL,
        )),
        completion_provider: Some(lsp_types::CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec!["<".to_string(), ":".to_string(), ".".to_string()]),
            ..Default::default()
        }),
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        ..Default::default()
    }
}

fn extract_settings(_init: &InitializeParams) -> Settings {
    let raw = match &_init.initialization_options {
        Some(v) => v,
        None => return Settings::default(),
    };
    serde_json::from_value::<ZsbcSettings>(raw.clone())
        .map(Into::into)
        .unwrap_or_default()
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ZsbcSettings {
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    additional_path: Option<String>,
    #[serde(default)]
    always_reload: bool,
    #[serde(default)]
    only_complete_brackets: bool,
    #[serde(default)]
    completion_suggest_all_items: bool,
    #[serde(default)]
    completion_suggest_with_start: bool,
}

impl From<ZsbcSettings> for Settings {
    fn from(s: ZsbcSettings) -> Self {
        Self {
            ct_log_path: s.path,
            additional_path: s.additional_path,
            always_reload: s.always_reload,
            only_complete_brackets: s.only_complete_brackets,
            completion_suggest_all_items: s.completion_suggest_all_items,
            completion_suggest_with_start: s.completion_suggest_with_start,
        }
    }
}

fn main_loop(connection: Connection, state: Arc<Mutex<State>>) -> Result<()> {
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                if let Err(e) = handle_request(&connection, req, &state) {
                    eprintln!("[zsbc-lsp] request error: {e:?}");
                }
            }
            Message::Notification(not) => {
                if let Err(e) = handle_notification(&connection, not, &state) {
                    eprintln!("[zsbc-lsp] notification error: {e:?}");
                }
            }
            Message::Response(_) => {}
        }
    }
    Ok(())
}

fn handle_request(
    connection: &Connection,
    req: Request,
    state: &Arc<Mutex<State>>,
) -> Result<()> {
    let req_id = req.id.clone();
    let method = req.method.clone();
    match method.as_str() {
        "textDocument/completion" => {
            let params: CompletionParams = serde_json::from_value(req.params)?;
            let items = state.lock().unwrap().completions(&params);
            let resp = CompletionResponse::Array(items);
            send_response(connection, req_id, Some(serde_json::to_value(resp)?), None);
        }
        "textDocument/hover" => {
            let params: HoverParams = serde_json::from_value(req.params)?;
            let hover = state.lock().unwrap().hover(&params);
            send_response(connection, req_id, Some(serde_json::to_value(hover)?), None);
        }
        _ => {
            send_response(
                connection,
                req_id,
                None,
                Some(ResponseError {
                    code: ErrorCode::MethodNotFound as i32,
                    message: format!("unknown method {method}"),
                    data: None,
                }),
            );
        }
    }
    Ok(())
}

fn send_response(
    connection: &Connection,
    id: RequestId,
    result: Option<serde_json::Value>,
    error: Option<ResponseError>,
) {
    let resp = Response { id, result, error };
    if let Err(e) = connection.sender.send(Message::Response(resp)) {
        eprintln!("[zsbc-lsp] failed to send response: {e}");
    }
}

fn handle_notification(
    _connection: &Connection,
    not: Notification,
    state: &Arc<Mutex<State>>,
) -> Result<()> {
    match not.method.as_str() {
        "initialized" => {
            // Nothing to do — the server's settings have already been
            // decoded in `extract_settings`.
        }
        "textDocument/didOpen" => {
            #[derive(serde::Deserialize)]
            struct DidOpenParams {
                text_document: lsp_types::TextDocumentItem,
            }
            let params: DidOpenParams = serde_json::from_value(not.params)?;
            let mut s = state.lock().unwrap();
            s.documents.insert(
                params.text_document.uri.to_string(),
                params.text_document.text,
            );
        }
        "textDocument/didChange" => {
            #[derive(serde::Deserialize)]
            struct DidChangeParams {
                text_document: lsp_types::VersionedTextDocumentIdentifier,
                content_changes: Vec<lsp_types::TextDocumentContentChangeEvent>,
            }
            let params: DidChangeParams = serde_json::from_value(not.params)?;
            let mut s = state.lock().unwrap();
            let key = params.text_document.uri.to_string();
            if let Some(change) = params.content_changes.last() {
                // FULL sync: each change event contains the entire buffer.
                s.documents.insert(key, change.text.clone());
            }
        }
        "textDocument/didClose" => {
            #[derive(serde::Deserialize)]
            struct DidCloseParams {
                text_document: lsp_types::TextDocumentIdentifier,
            }
            let params: DidCloseParams = serde_json::from_value(not.params)?;
            state
                .lock()
                .unwrap()
                .documents
                .remove(&params.text_document.uri.to_string());
        }
        "zsbc/reload" => {
            state.lock().unwrap().reload()?;
        }
        "exit" => std::process::exit(0),
        _ => {
            // Ignore everything else.
        }
    }
    Ok(())
}

impl State {
    fn completions(&mut self, params: &CompletionParams) -> Vec<CompletionItem> {
        self.maybe_reload();
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let doc = match self.documents.get(&uri.to_string()) {
            Some(d) => d.clone(),
            None => return Vec::new(),
        };
        let line = match doc.lines().nth(pos.line as usize) {
            Some(l) => l.to_string(),
            None => return Vec::new(),
        };

        let (needle, replace_range) = match self.settings.only_complete_brackets {
            true => match bracket_word_range(&line, pos.character as usize) {
                Some((_, start, end)) => {
                    let raw: String = line[start..end].to_string();
                    (raw.trim_start_matches('<').to_string(), (start, end))
                }
                None => return Vec::new(),
            },
            false => match loose_word_range(&line, pos.character as usize) {
                Some((_, start, end)) => {
                    let raw: String = line[start..end].to_string();
                    (raw, (start, end))
                }
                None => return Vec::new(),
            },
        };

        let items: Vec<CompletionItem> = self
            .items
            .iter()
            .filter(|(key, _)| {
                if self.settings.completion_suggest_all_items {
                    return true;
                }
                if self.settings.completion_suggest_with_start {
                    return key.starts_with(&needle);
                }
                key.contains(&needle)
            })
            .take(200)
            .map(|(key, value)| CompletionItem {
                label: key.clone(),
                detail: Some(value.clone()),
                kind: Some(CompletionItemKind::VALUE),
                text_edit: Some(lsp_types::CompletionTextEdit::Edit(
                    lsp_types::TextEdit {
                        range: Range {
                            start: Position {
                                line: pos.line,
                                character: replace_range.0 as u32,
                            },
                            end: Position {
                                line: pos.line,
                                character: replace_range.1 as u32,
                            },
                        },
                        new_text: key.clone(),
                    },
                )),
                ..Default::default()
            })
            .collect();

        items
    }

    fn hover(&mut self, params: &HoverParams) -> Option<Hover> {
        self.maybe_reload();
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let doc = self.documents.get(&uri.to_string())?;
        let line = doc.lines().nth(pos.line as usize)?;
        let (key, range) = bracket_hover_range(line, pos.character as usize)?;
        let value = self.items.get(&key)?;
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("```\n{key}\n```\n\n{value}"),
            }),
            range: Some(range),
        })
    }
}

fn bracket_word_range(line: &str, col: usize) -> Option<(String, usize, usize)> {
    let bytes = line.as_bytes();
    let mut start = col;
    while start > 0 && is_word_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = col;
    while end < bytes.len() && is_word_byte(bytes[end]) {
        end += 1;
    }
    if start == end {
        return None;
    }
    Some((line[start..end].to_string(), start, end))
}

fn loose_word_range(line: &str, col: usize) -> Option<(String, usize, usize)> {
    bracket_word_range(line, col)
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b':' | b'.' | b'<')
}

fn bracket_hover_range(line: &str, col: usize) -> Option<(String, Range)> {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    if chars.is_empty() {
        return None;
    }
    let pos = chars.iter().position(|(i, _)| *i >= col).unwrap_or(chars.len());
    let mut start = pos;
    while start > 0 && chars[start - 1].1 != '<' {
        start -= 1;
    }
    if start >= chars.len() || chars[start].1 != '<' {
        return None;
    }
    let mut end = pos;
    if end < chars.len() && chars[end].1 == '>' {
        end += 1;
    } else {
        while end < chars.len() && chars[end].1 != '>' {
            end += 1;
        }
        if end < chars.len() {
            end += 1;
        }
    }
    if start >= end {
        return None;
    }
    let raw: String = chars[start..end].iter().map(|(_, c)| *c).collect();
    let stripped = raw.replace(":0", "");
    let start_byte = chars[start].0;
    let (end_byte, last_char_len) = {
        let (b, c) = chars[end - 1];
        (b + c.len_utf8(), c.len_utf8())
    };
    let range = Range {
        start: Position {
            line: 0,
            character: start_byte as u32,
        },
        end: Position {
            line: 0,
            character: end_byte as u32,
        },
    };
    let _ = last_char_len;
    Some((stripped, range))
}
