/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::once;
use std::num::NonZero;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;

use dupe::Dupe;
use lsp_server::Connection;
use lsp_server::ErrorCode;
use lsp_server::Message;
use lsp_server::Request;
use lsp_server::RequestId;
use lsp_server::Response;
use lsp_types::CodeAction;
use lsp_types::CodeActionKind;
use lsp_types::CodeActionOptions;
use lsp_types::CodeActionOrCommand;
use lsp_types::CodeActionParams;
use lsp_types::CodeActionProviderCapability;
use lsp_types::CodeActionResponse;
use lsp_types::CompletionList;
use lsp_types::CompletionOptions;
use lsp_types::CompletionParams;
use lsp_types::CompletionResponse;
use lsp_types::ConfigurationItem;
use lsp_types::ConfigurationParams;
use lsp_types::Diagnostic;
use lsp_types::DidChangeConfigurationParams;
use lsp_types::DidChangeTextDocumentParams;
use lsp_types::DidChangeWatchedFilesClientCapabilities;
use lsp_types::DidChangeWatchedFilesParams;
use lsp_types::DidChangeWatchedFilesRegistrationOptions;
use lsp_types::DidChangeWorkspaceFoldersParams;
use lsp_types::DidCloseTextDocumentParams;
use lsp_types::DidOpenTextDocumentParams;
use lsp_types::DidSaveTextDocumentParams;
use lsp_types::DocumentDiagnosticParams;
use lsp_types::DocumentDiagnosticReport;
use lsp_types::DocumentHighlight;
use lsp_types::DocumentHighlightParams;
use lsp_types::DocumentSymbol;
use lsp_types::DocumentSymbolParams;
use lsp_types::DocumentSymbolResponse;
use lsp_types::FileSystemWatcher;
use lsp_types::FullDocumentDiagnosticReport;
use lsp_types::GlobPattern;
use lsp_types::GotoDefinitionParams;
use lsp_types::GotoDefinitionResponse;
use lsp_types::Hover;
use lsp_types::HoverContents;
use lsp_types::HoverParams;
use lsp_types::HoverProviderCapability;
use lsp_types::InitializeParams;
use lsp_types::InlayHint;
use lsp_types::InlayHintLabel;
use lsp_types::InlayHintParams;
use lsp_types::Location;
use lsp_types::NumberOrString;
use lsp_types::OneOf;
use lsp_types::Position;
use lsp_types::PositionEncodingKind;
use lsp_types::PrepareRenameResponse;
use lsp_types::PublishDiagnosticsParams;
use lsp_types::Range;
use lsp_types::ReferenceParams;
use lsp_types::Registration;
use lsp_types::RegistrationParams;
use lsp_types::RelatedFullDocumentDiagnosticReport;
use lsp_types::RelativePattern;
use lsp_types::RenameOptions;
use lsp_types::RenameParams;
use lsp_types::SemanticTokens;
use lsp_types::SemanticTokensFullOptions;
use lsp_types::SemanticTokensOptions;
use lsp_types::SemanticTokensParams;
use lsp_types::SemanticTokensRangeParams;
use lsp_types::SemanticTokensRangeResult;
use lsp_types::SemanticTokensResult;
use lsp_types::SemanticTokensServerCapabilities;
use lsp_types::ServerCapabilities;
use lsp_types::SignatureHelp;
use lsp_types::SignatureHelpOptions;
use lsp_types::SignatureHelpParams;
use lsp_types::SymbolInformation;
use lsp_types::TextDocumentPositionParams;
use lsp_types::TextDocumentSyncCapability;
use lsp_types::TextDocumentSyncKind;
use lsp_types::TextEdit;
use lsp_types::Unregistration;
use lsp_types::UnregistrationParams;
use lsp_types::Url;
use lsp_types::VersionedTextDocumentIdentifier;
use lsp_types::WatchKind;
use lsp_types::WorkspaceClientCapabilities;
use lsp_types::WorkspaceEdit;
use lsp_types::WorkspaceFoldersServerCapabilities;
use lsp_types::WorkspaceServerCapabilities;
use lsp_types::WorkspaceSymbolResponse;
use lsp_types::notification::Cancel;
use lsp_types::notification::DidChangeConfiguration;
use lsp_types::notification::DidChangeTextDocument;
use lsp_types::notification::DidChangeWatchedFiles;
use lsp_types::notification::DidChangeWorkspaceFolders;
use lsp_types::notification::DidCloseTextDocument;
use lsp_types::notification::DidOpenTextDocument;
use lsp_types::notification::DidSaveTextDocument;
use lsp_types::notification::Exit;
use lsp_types::notification::Notification as _;
use lsp_types::notification::PublishDiagnostics;
use lsp_types::request::CodeActionRequest;
use lsp_types::request::Completion;
use lsp_types::request::DocumentDiagnosticRequest;
use lsp_types::request::DocumentHighlightRequest;
use lsp_types::request::DocumentSymbolRequest;
use lsp_types::request::GotoDefinition;
use lsp_types::request::HoverRequest;
use lsp_types::request::InlayHintRequest;
use lsp_types::request::PrepareRenameRequest;
use lsp_types::request::References;
use lsp_types::request::RegisterCapability;
use lsp_types::request::Rename;
use lsp_types::request::Request as _;
use lsp_types::request::SemanticTokensFullRequest;
use lsp_types::request::SemanticTokensRangeRequest;
use lsp_types::request::SignatureHelpRequest;
use lsp_types::request::UnregisterCapability;
use lsp_types::request::WorkspaceConfiguration;
use lsp_types::request::WorkspaceSymbolRequest;
use pyrefly_python::PYTHON_EXTENSIONS;
use pyrefly_python::module::TextRangeWithModule;
use pyrefly_python::module_name::ModuleName;
use pyrefly_python::module_path::ModulePath;
use pyrefly_util::arc_id::ArcId;
use pyrefly_util::events::CategorizedEvents;
use pyrefly_util::lock::Mutex;
use pyrefly_util::lock::RwLock;
use pyrefly_util::prelude::VecExt;
use pyrefly_util::task_heap::CancellationHandle;
use pyrefly_util::task_heap::Cancelled;
use pyrefly_util::thread_pool::ThreadCount;
use pyrefly_util::thread_pool::ThreadPool;
use serde::de::DeserializeOwned;
use serde_json::Value;
use starlark_map::small_map::SmallMap;

use crate::commands::lsp::IndexingMode;
use crate::config::config::ConfigFile;
use crate::error::error::Error;
use crate::lsp::features::hover::get_hover;
use crate::lsp::lsp::apply_change_events;
use crate::lsp::lsp::as_notification;
use crate::lsp::lsp::as_request;
use crate::lsp::lsp::as_request_response_pair;
use crate::lsp::lsp::new_notification;
use crate::lsp::lsp::new_response;
use crate::lsp::module_helpers::handle_from_module_path;
use crate::lsp::module_helpers::make_open_handle;
use crate::lsp::module_helpers::module_info_to_uri;
use crate::lsp::module_helpers::to_lsp_location;
use crate::lsp::module_helpers::to_real_path;
use crate::lsp::queue::LspEvent;
use crate::lsp::queue::LspQueue;
use crate::lsp::transaction_manager::TransactionManager;
use crate::lsp::workspace::Workspace;
use crate::lsp::workspace::Workspaces;
use crate::module::from_path::module_from_path;
use crate::state::handle::Handle;
use crate::state::lsp::FindDefinitionItemWithDocstring;
use crate::state::require::Require;
use crate::state::semantic_tokens::SemanticTokensLegends;
use crate::state::state::State;
use crate::state::state::Transaction;

#[derive(Clone, Dupe)]
struct ServerConnection(Arc<Connection>);

impl ServerConnection {
    fn send(&self, msg: Message) {
        if self.0.sender.send(msg).is_err() {
            // On error, we know the channel is closed.
            // https://docs.rs/crossbeam/latest/crossbeam/channel/struct.Sender.html#method.send
            eprintln!("Connection closed.");
        };
    }

    fn publish_diagnostics_for_uri(&self, uri: Url, diags: Vec<Diagnostic>, version: Option<i32>) {
        self.send(Message::Notification(
            new_notification::<PublishDiagnostics>(PublishDiagnosticsParams::new(
                uri, diags, version,
            )),
        ));
    }

    fn publish_diagnostics(&self, diags: SmallMap<PathBuf, Vec<Diagnostic>>) {
        for (path, diags) in diags {
            let path = std::fs::canonicalize(&path).unwrap_or(path);
            match Url::from_file_path(&path) {
                Ok(uri) => self.publish_diagnostics_for_uri(uri, diags, None),
                Err(_) => eprint!("Unable to convert path to uri: {path:?}"),
            }
        }
    }
}

struct Server {
    connection: ServerConnection,
    /// A thread pool of size one for heavy read operations on the State
    async_state_read_threads: ThreadPool,
    lsp_queue: LspQueue,
    initialize_params: InitializeParams,
    indexing_mode: IndexingMode,
    state: Arc<State>,
    open_files: Arc<RwLock<HashMap<PathBuf, Arc<String>>>>,
    /// A set of configs where we have already indexed all the files within the config.
    indexed_configs: Mutex<HashSet<ArcId<ConfigFile>>>,
    cancellation_handles: Arc<Mutex<HashMap<RequestId, CancellationHandle>>>,
    workspaces: Arc<Workspaces>,
    outgoing_request_id: AtomicI32,
    outgoing_requests: Mutex<HashMap<RequestId, Request>>,
    filewatcher_registered: AtomicBool,
    version_info: Mutex<HashMap<PathBuf, i32>>,
}

/// At the time when we are ready to handle a new LSP event, it will help if we know the a list of
/// buffered requests and notifications ready to be processed, because we can potentially make smart
/// decisions (e.g. not process cancelled requests).
///
/// This function listens to the LSP events in the order they arrive, and dispatch them into event
/// channels with various priority:
/// - priority_events includes those that should be handled as soon as possible (e.g. know that a
///   request is cancelled)
/// - queued_events includes most of the other events.
fn dispatch_lsp_events(connection: &Connection, lsp_queue: LspQueue) {
    for msg in &connection.receiver {
        match msg {
            Message::Request(x) => {
                match connection.handle_shutdown(&x) {
                    Ok(is_shutdown) => {
                        if is_shutdown {
                            return;
                        }
                    }
                    Err(_) => {
                        return;
                    }
                }
                if lsp_queue.send(LspEvent::LspRequest(x)).is_err() {
                    return;
                }
            }
            Message::Response(x) => {
                if lsp_queue.send(LspEvent::LspResponse(x)).is_err() {
                    return;
                }
            }
            Message::Notification(x) => {
                let send_result = if let Some(Ok(params)) =
                    as_notification::<DidOpenTextDocument>(&x)
                {
                    lsp_queue.send(LspEvent::DidOpenTextDocument(params))
                } else if let Some(Ok(params)) = as_notification::<DidChangeTextDocument>(&x) {
                    lsp_queue.send(LspEvent::DidChangeTextDocument(params))
                } else if let Some(Ok(params)) = as_notification::<DidCloseTextDocument>(&x) {
                    lsp_queue.send(LspEvent::DidCloseTextDocument(params))
                } else if let Some(Ok(params)) = as_notification::<DidSaveTextDocument>(&x) {
                    lsp_queue.send(LspEvent::DidSaveTextDocument(params))
                } else if let Some(Ok(params)) = as_notification::<DidChangeWatchedFiles>(&x) {
                    lsp_queue.send(LspEvent::DidChangeWatchedFiles(params))
                } else if let Some(Ok(params)) = as_notification::<DidChangeWorkspaceFolders>(&x) {
                    lsp_queue.send(LspEvent::DidChangeWorkspaceFolders(params))
                } else if let Some(Ok(params)) = as_notification::<DidChangeConfiguration>(&x) {
                    lsp_queue.send(LspEvent::DidChangeConfiguration(params))
                } else if let Some(Ok(params)) = as_notification::<Cancel>(&x) {
                    let id = match params.id {
                        NumberOrString::Number(i) => RequestId::from(i),
                        NumberOrString::String(s) => RequestId::from(s),
                    };
                    lsp_queue.send(LspEvent::CancelRequest(id))
                } else if as_notification::<Exit>(&x).is_some() {
                    lsp_queue.send(LspEvent::Exit)
                } else {
                    eprintln!("Unhandled notification: {x:?}");
                    Ok(())
                };
                if send_result.is_err() {
                    return;
                }
            }
        }
    }
}

pub fn capabilities(
    indexing_mode: IndexingMode,
    initialization_params: &InitializeParams,
) -> ServerCapabilities {
    let augments_syntax_tokens = initialization_params
        .capabilities
        .text_document
        .as_ref()
        .and_then(|c| c.semantic_tokens.as_ref())
        .and_then(|c| c.augments_syntax_tokens)
        .unwrap_or(false);
    ServerCapabilities {
        position_encoding: Some(PositionEncodingKind::UTF16),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        definition_provider: Some(OneOf::Left(true)),
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
            ..Default::default()
        })),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec![".".to_owned()]),
            ..Default::default()
        }),
        document_highlight_provider: Some(OneOf::Left(true)),
        // Find references won't work properly if we don't know all the files.
        references_provider: match indexing_mode {
            IndexingMode::None => None,
            IndexingMode::LazyNonBlockingBackground | IndexingMode::LazyBlocking => {
                Some(OneOf::Left(true))
            }
        },
        rename_provider: match indexing_mode {
            IndexingMode::None => None,
            IndexingMode::LazyNonBlockingBackground | IndexingMode::LazyBlocking => {
                Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: Default::default(),
                }))
            }
        },
        signature_help_provider: Some(SignatureHelpOptions {
            trigger_characters: Some(vec!["(".to_owned(), ",".to_owned()]),
            ..Default::default()
        }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        inlay_hint_provider: Some(OneOf::Left(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        workspace_symbol_provider: Some(OneOf::Left(true)),
        semantic_tokens_provider: if augments_syntax_tokens {
            // We currently only return partial tokens (e.g. no tokens for keywords right now).
            // If the client doesn't support `augments_syntax_tokens` to fallback baseline
            // syntax highlighting for tokens we don't provide, it will be a regression
            // (e.g. users might lose keyword highlighting).
            // Therefore, we should not produce semantic tokens if the client doesn't support `augments_syntax_tokens`.
            Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
                SemanticTokensOptions {
                    legend: SemanticTokensLegends::lsp_semantic_token_legends(),
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                    range: Some(true),
                    ..Default::default()
                },
            ))
        } else {
            None
        },
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(true)),
            }),
            file_operations: None,
        }),
        ..Default::default()
    }
}

enum ProcessEvent {
    Continue,
    Exit,
}

const PYTHON_SECTION: &str = "python";

pub fn lsp_loop(
    connection: Arc<Connection>,
    initialization_params: InitializeParams,
    indexing_mode: IndexingMode,
) -> anyhow::Result<()> {
    eprintln!("Reading messages");
    let connection_for_dispatcher = connection.dupe();
    let lsp_queue = LspQueue::new();
    let server = Server::new(
        connection,
        lsp_queue.dupe(),
        initialization_params,
        indexing_mode,
    );
    let lsp_queue2 = lsp_queue.dupe();
    std::thread::spawn(move || {
        dispatch_lsp_events(&connection_for_dispatcher, lsp_queue2);
    });
    let mut ide_transaction_manager = TransactionManager::default();
    let mut canceled_requests = HashSet::new();
    while let Ok((subsequent_mutation, event)) = lsp_queue.recv() {
        match server.process_event(
            &mut ide_transaction_manager,
            &mut canceled_requests,
            subsequent_mutation,
            event,
        )? {
            ProcessEvent::Continue => {}
            ProcessEvent::Exit => break,
        }
    }
    eprintln!("waiting for connection to close");
    drop(server); // close connection
    Ok(())
}

impl Server {
    const FILEWATCHER_ID: &str = "FILEWATCHER";

    fn extract_request_params_or_send_err_response<T>(
        &self,
        params: Result<T::Params, serde_json::Error>,
        id: &RequestId,
    ) -> Option<T::Params>
    where
        T: lsp_types::request::Request,
        T::Params: DeserializeOwned,
    {
        match params {
            Ok(params) => Some(params),
            Err(err) => {
                self.send_response(Response::new_err(
                    id.clone(),
                    ErrorCode::InvalidParams as i32,
                    err.to_string(),
                ));
                None
            }
        }
    }

    /// Process the event and return next step.
    fn process_event<'a>(
        &'a self,
        ide_transaction_manager: &mut TransactionManager<'a>,
        canceled_requests: &mut HashSet<RequestId>,
        // After this event there is another mutation
        subsequent_mutation: bool,
        event: LspEvent,
    ) -> anyhow::Result<ProcessEvent> {
        match event {
            LspEvent::Exit => {
                return Ok(ProcessEvent::Exit);
            }
            LspEvent::RecheckFinished => {
                // We did a commit and want to get back to a stable state.
                self.validate_in_memory(ide_transaction_manager);
            }
            LspEvent::CancelRequest(id) => {
                eprintln!("We should cancel request {id:?}");
                if let Some(cancellation_handle) = self.cancellation_handles.lock().remove(&id) {
                    cancellation_handle.cancel();
                }
                canceled_requests.insert(id);
            }
            LspEvent::DidOpenTextDocument(params) => {
                self.did_open(ide_transaction_manager, subsequent_mutation, params);
            }
            LspEvent::DidChangeTextDocument(params) => {
                self.did_change(ide_transaction_manager, subsequent_mutation, params)?;
            }
            LspEvent::DidCloseTextDocument(params) => {
                self.did_close(params);
            }
            LspEvent::DidSaveTextDocument(params) => {
                self.did_save(params);
            }
            LspEvent::DidChangeWatchedFiles(params) => {
                self.did_change_watched_files(params);
            }
            LspEvent::DidChangeWorkspaceFolders(params) => {
                self.workspace_folders_changed(params);
            }
            LspEvent::DidChangeConfiguration(params) => {
                self.did_change_configuration(ide_transaction_manager, params);
            }
            LspEvent::LspResponse(x) => {
                if let Some(request) = self.outgoing_requests.lock().remove(&x.id) {
                    if let Some((request, response)) =
                        as_request_response_pair::<WorkspaceConfiguration>(&request, &x)
                    {
                        self.workspace_configuration_response(
                            ide_transaction_manager,
                            &request,
                            &response,
                        );
                    }
                } else {
                    eprintln!("Response for unknown request: {x:?}");
                }
            }
            LspEvent::LspRequest(x) => {
                // These are messages where VS Code will use results from previous document versions,
                // we really don't want to implicitly cancel those.
                const ONLY_ONCE: &[&str] = &[Completion::METHOD, SignatureHelpRequest::METHOD];

                let in_cancelled_requests = canceled_requests.remove(&x.id);
                if in_cancelled_requests
                    || (subsequent_mutation && !ONLY_ONCE.contains(&x.method.as_str()))
                {
                    let message = format!(
                        "Request {} ({}) is canceled due to {}",
                        x.method,
                        x.id,
                        if in_cancelled_requests {
                            "explicit cancellation"
                        } else {
                            "subsequent mutation"
                        }
                    );
                    eprintln!("{message}");
                    self.send_response(Response::new_err(
                        x.id,
                        ErrorCode::RequestCanceled as i32,
                        message,
                    ));
                    return Ok(ProcessEvent::Continue);
                }

                if subsequent_mutation {
                    // We probably didn't bother completing a previous check, but we are now answering a query that
                    // really needs a previous check to be correct.
                    // Validating sends out notifications, which isn't required, but this is the safest way.
                    self.validate_in_memory(ide_transaction_manager);
                }

                eprintln!("Handling non-canceled request {} ({})", x.method, &x.id);
                if let Some(params) = as_request::<GotoDefinition>(&x)
                    && let Some(params) = self
                        .extract_request_params_or_send_err_response::<GotoDefinition>(
                            params, &x.id,
                        )
                {
                    let default_response = GotoDefinitionResponse::Array(Vec::new());
                    let transaction =
                        ide_transaction_manager.non_commitable_transaction(&self.state);
                    self.send_response(new_response(
                        x.id,
                        Ok(self
                            .goto_definition(&transaction, params)
                            .unwrap_or(default_response)),
                    ));
                    ide_transaction_manager.save(transaction);
                } else if let Some(params) = as_request::<CodeActionRequest>(&x)
                    && let Some(params) = self
                        .extract_request_params_or_send_err_response::<CodeActionRequest>(
                            params, &x.id,
                        )
                {
                    let transaction =
                        ide_transaction_manager.non_commitable_transaction(&self.state);
                    self.send_response(new_response(
                        x.id,
                        Ok(self.code_action(&transaction, params).unwrap_or_default()),
                    ));
                    ide_transaction_manager.save(transaction);
                } else if let Some(params) = as_request::<Completion>(&x)
                    && let Some(params) = self
                        .extract_request_params_or_send_err_response::<Completion>(params, &x.id)
                {
                    let transaction =
                        ide_transaction_manager.non_commitable_transaction(&self.state);
                    self.send_response(new_response(x.id, self.completion(&transaction, params)));
                    ide_transaction_manager.save(transaction);
                } else if let Some(params) = as_request::<DocumentHighlightRequest>(&x)
                    && let Some(params) = self
                        .extract_request_params_or_send_err_response::<DocumentHighlightRequest>(
                            params, &x.id,
                        )
                {
                    let transaction =
                        ide_transaction_manager.non_commitable_transaction(&self.state);
                    self.send_response(new_response(
                        x.id,
                        Ok(self.document_highlight(&transaction, params)),
                    ));
                    ide_transaction_manager.save(transaction);
                } else if let Some(params) = as_request::<References>(&x)
                    && let Some(params) = self
                        .extract_request_params_or_send_err_response::<References>(params, &x.id)
                {
                    self.references(x.id, ide_transaction_manager, params);
                } else if let Some(params) = as_request::<PrepareRenameRequest>(&x)
                    && let Some(params) = self
                        .extract_request_params_or_send_err_response::<PrepareRenameRequest>(
                            params, &x.id,
                        )
                {
                    let transaction =
                        ide_transaction_manager.non_commitable_transaction(&self.state);
                    self.send_response(new_response(
                        x.id,
                        Ok(self.prepare_rename(&transaction, params)),
                    ));
                    ide_transaction_manager.save(transaction);
                } else if let Some(params) = as_request::<Rename>(&x)
                    && let Some(params) =
                        self.extract_request_params_or_send_err_response::<Rename>(params, &x.id)
                {
                    self.rename(x.id, ide_transaction_manager, params);
                } else if let Some(params) = as_request::<SignatureHelpRequest>(&x)
                    && let Some(params) = self
                        .extract_request_params_or_send_err_response::<SignatureHelpRequest>(
                            params, &x.id,
                        )
                {
                    let transaction =
                        ide_transaction_manager.non_commitable_transaction(&self.state);
                    self.send_response(new_response(
                        x.id,
                        Ok(self.signature_help(&transaction, params)),
                    ));
                    ide_transaction_manager.save(transaction);
                } else if let Some(params) = as_request::<HoverRequest>(&x)
                    && let Some(params) = self
                        .extract_request_params_or_send_err_response::<HoverRequest>(params, &x.id)
                {
                    let default_response = Hover {
                        contents: HoverContents::Array(Vec::new()),
                        range: None,
                    };
                    let transaction =
                        ide_transaction_manager.non_commitable_transaction(&self.state);
                    self.send_response(new_response(
                        x.id,
                        Ok(self.hover(&transaction, params).unwrap_or(default_response)),
                    ));
                    ide_transaction_manager.save(transaction);
                } else if let Some(params) = as_request::<InlayHintRequest>(&x)
                    && let Some(params) = self
                        .extract_request_params_or_send_err_response::<InlayHintRequest>(
                            params, &x.id,
                        )
                {
                    let transaction =
                        ide_transaction_manager.non_commitable_transaction(&self.state);
                    self.send_response(new_response(
                        x.id,
                        Ok(self.inlay_hints(&transaction, params).unwrap_or_default()),
                    ));
                    ide_transaction_manager.save(transaction);
                } else if let Some(params) = as_request::<SemanticTokensFullRequest>(&x)
                    && let Some(params) = self
                        .extract_request_params_or_send_err_response::<SemanticTokensFullRequest>(
                            params, &x.id,
                        )
                {
                    let default_response = SemanticTokensResult::Tokens(SemanticTokens {
                        result_id: None,
                        data: Vec::new(),
                    });
                    let transaction =
                        ide_transaction_manager.non_commitable_transaction(&self.state);
                    self.send_response(new_response(
                        x.id,
                        Ok(self
                            .semantic_tokens_full(&transaction, params)
                            .unwrap_or(default_response)),
                    ));
                    ide_transaction_manager.save(transaction);
                } else if let Some(params) = as_request::<SemanticTokensRangeRequest>(&x)
                    && let Some(params) = self
                        .extract_request_params_or_send_err_response::<SemanticTokensRangeRequest>(
                            params, &x.id,
                        )
                {
                    let default_response = SemanticTokensRangeResult::Tokens(SemanticTokens {
                        result_id: None,
                        data: Vec::new(),
                    });
                    let transaction =
                        ide_transaction_manager.non_commitable_transaction(&self.state);
                    self.send_response(new_response(
                        x.id,
                        Ok(self
                            .semantic_tokens_ranged(&transaction, params)
                            .unwrap_or(default_response)),
                    ));
                    ide_transaction_manager.save(transaction);
                } else if let Some(params) = as_request::<DocumentSymbolRequest>(&x)
                    && let Some(params) = self
                        .extract_request_params_or_send_err_response::<DocumentSymbolRequest>(
                            params, &x.id,
                        )
                {
                    let transaction =
                        ide_transaction_manager.non_commitable_transaction(&self.state);
                    self.send_response(new_response(
                        x.id,
                        Ok(DocumentSymbolResponse::Nested(
                            self.hierarchical_document_symbols(&transaction, params)
                                .unwrap_or_default(),
                        )),
                    ));
                    ide_transaction_manager.save(transaction);
                } else if let Some(params) = as_request::<WorkspaceSymbolRequest>(&x)
                    && let Some(params) = self
                        .extract_request_params_or_send_err_response::<WorkspaceSymbolRequest>(
                            params, &x.id,
                        )
                {
                    let transaction =
                        ide_transaction_manager.non_commitable_transaction(&self.state);
                    self.send_response(new_response(
                        x.id,
                        Ok(WorkspaceSymbolResponse::Flat(
                            self.workspace_symbols(&transaction, &params.query),
                        )),
                    ));
                    ide_transaction_manager.save(transaction);
                } else if let Some(params) = as_request::<DocumentDiagnosticRequest>(&x)
                    && let Some(params) = self
                        .extract_request_params_or_send_err_response::<DocumentDiagnosticRequest>(
                            params, &x.id,
                        )
                {
                    self.validate_in_memory(ide_transaction_manager);
                    let transaction =
                        ide_transaction_manager.non_commitable_transaction(&self.state);
                    self.send_response(new_response(
                        x.id,
                        Ok(self.document_diagnostics(&transaction, params)),
                    ));
                    ide_transaction_manager.save(transaction);
                } else {
                    eprintln!("Unhandled request: {x:?}");
                }
            }
        }
        Ok(ProcessEvent::Continue)
    }

    fn new(
        connection: Arc<Connection>,
        lsp_queue: LspQueue,
        initialize_params: InitializeParams,
        indexing_mode: IndexingMode,
    ) -> Self {
        let folders = if let Some(capability) = &initialize_params.capabilities.workspace
            && let Some(true) = capability.workspace_folders
            && let Some(folders) = &initialize_params.workspace_folders
        {
            folders
                .iter()
                .map(|x| x.uri.to_file_path().unwrap())
                .collect()
        } else {
            Vec::new()
        };

        let workspaces = Arc::new(Workspaces::new(Workspace::default(), &folders));

        let config_finder = Workspaces::config_finder(&workspaces);
        let s = Self {
            connection: ServerConnection(connection),
            async_state_read_threads: ThreadPool::with_thread_count(ThreadCount::NumThreads(
                NonZero::new(1).unwrap(),
            )),
            lsp_queue,
            initialize_params,
            indexing_mode,
            state: Arc::new(State::new(config_finder)),
            open_files: Arc::new(RwLock::new(HashMap::new())),
            indexed_configs: Mutex::new(HashSet::new()),
            cancellation_handles: Arc::new(Mutex::new(HashMap::new())),
            workspaces,
            outgoing_request_id: AtomicI32::new(1),
            outgoing_requests: Mutex::new(HashMap::new()),
            filewatcher_registered: AtomicBool::new(false),
            version_info: Mutex::new(HashMap::new()),
        };
        s.setup_file_watcher_if_necessary();
        s.request_settings_for_all_workspaces();
        s
    }

    fn send_response(&self, x: Response) {
        self.connection.send(Message::Response(x))
    }

    fn send_request<T>(&self, params: T::Params)
    where
        T: lsp_types::request::Request,
    {
        let id = RequestId::from(self.outgoing_request_id.fetch_add(1, Ordering::SeqCst));
        let request = Request {
            id: id.clone(),
            method: T::METHOD.to_owned(),
            params: serde_json::to_value(params).unwrap(),
        };
        self.connection.send(Message::Request(request.clone()));
        self.outgoing_requests.lock().insert(id, request);
    }

    fn validate_in_memory_for_transaction(
        state: &State,
        open_files: &RwLock<HashMap<PathBuf, Arc<String>>>,
        transaction: &mut Transaction<'_>,
    ) -> Vec<(Handle, Require)> {
        let handles = open_files
            .read()
            .keys()
            .map(|x| (make_open_handle(state, x), Require::Everything))
            .collect::<Vec<_>>();
        transaction.set_memory(
            open_files
                .read()
                .iter()
                .map(|x| (x.0.clone(), Some(x.1.dupe())))
                .collect::<Vec<_>>(),
        );
        transaction.run(&handles);
        handles
    }

    fn get_diag_if_shown(
        &self,
        e: &Error,
        open_files: &HashMap<PathBuf, Arc<String>>,
    ) -> Option<(PathBuf, Diagnostic)> {
        if let Some(path) = to_real_path(e.path()) {
            // When no file covers this, we'll get the default configured config which includes "everything"
            // and excludes `.<file>`s.
            let config = self
                .state
                .config_finder()
                .python_file(ModuleName::unknown(), e.path());
            if open_files.contains_key(&path)
                && !config.project_excludes.covers(&path)
                && !self
                    .workspaces
                    .get_with(path.to_path_buf(), |w| w.disable_type_errors)
            {
                return Some((path.to_path_buf(), e.to_diagnostic()));
            }
        }
        None
    }

    fn validate_in_memory<'a>(&'a self, ide_transaction_manager: &mut TransactionManager<'a>) {
        let mut possibly_committable_transaction =
            ide_transaction_manager.get_possibly_committable_transaction(&self.state);
        let transaction = match &mut possibly_committable_transaction {
            Ok(transaction) => transaction.as_mut(),
            Err(transaction) => transaction,
        };
        let handles =
            Self::validate_in_memory_for_transaction(&self.state, &self.open_files, transaction);

        let publish = |transaction: &Transaction| {
            let mut diags: SmallMap<PathBuf, Vec<Diagnostic>> = SmallMap::new();
            let open_files = self.open_files.read();
            for x in open_files.keys() {
                diags.insert(x.as_path().to_owned(), Vec::new());
            }
            for e in transaction
                .get_errors(handles.iter().map(|(handle, _)| handle))
                .collect_errors()
                .shown
            {
                if let Some((path, diag)) = self.get_diag_if_shown(&e, &open_files) {
                    diags.entry(path.to_owned()).or_default().push(diag);
                }
            }
            self.connection.publish_diagnostics(diags);
        };

        match possibly_committable_transaction {
            Ok(transaction) => {
                self.state.commit_transaction(transaction);
                // In the case where we can commit transactions, `State` already has latest updates.
                // Therefore, we can compute errors from transactions freshly created from `State``.
                let transaction = self.state.transaction();
                publish(&transaction);
            }
            Err(transaction) => {
                // In the case where transaction cannot be committed because there is an ongoing
                // recheck, we still want to update the diagnostics. In this case, we compute them
                // from the transactions that won't be committed. It will still contain all the
                // up-to-date in-memory content, but can have stale main `State` content.
                publish(&transaction);
                ide_transaction_manager.save(transaction);
            }
        }
    }

    fn populate_project_files_if_necessary(
        &self,
        config_to_populate_files: Option<ArcId<ConfigFile>>,
    ) {
        if let Some(config) = config_to_populate_files {
            match self.indexing_mode {
                IndexingMode::None => {}
                IndexingMode::LazyNonBlockingBackground => {
                    if self.indexed_configs.lock().insert(config.dupe()) {
                        let state = self.state.dupe();
                        let lsp_queue = self.lsp_queue.dupe();
                        std::thread::spawn(move || {
                            Self::populate_all_project_files_in_config(config, state, lsp_queue);
                        });
                    }
                }
                IndexingMode::LazyBlocking => {
                    if self.indexed_configs.lock().insert(config.dupe()) {
                        Self::populate_all_project_files_in_config(
                            config,
                            self.state.dupe(),
                            self.lsp_queue.dupe(),
                        );
                    }
                }
            }
        }
    }

    /// Perform an invalidation of elements on `State` and commit them.
    /// Runs asynchronously. Returns immediately and may wait a while for a committable transaction.
    fn invalidate(&self, f: impl FnOnce(&mut Transaction) + Send + 'static) {
        let state = self.state.dupe();
        let lsp_queue = self.lsp_queue.dupe();
        let cancellation_handles = self.cancellation_handles.dupe();
        std::thread::spawn(move || {
            let mut transaction = state.new_committable_transaction(Require::Indexing, None);
            f(transaction.as_mut());
            // Commit will be blocked until there are no ongoing reads.
            // If we have some long running read jobs that can be cancelled, we should cancel them
            // to unblock committing transactions.
            for (_, cancellation_handle) in cancellation_handles.lock().drain() {
                cancellation_handle.cancel();
            }
            // we have to run, not just commit to process updates
            state.run_with_committing_transaction(transaction, &[]);
            // After we finished a recheck asynchronously, we immediately send `RecheckFinished` to
            // the main event loop of the server. As a result, the server can do a revalidation of
            // all the in-memory files based on the fresh main State as soon as possible.
            let _ = lsp_queue.send(LspEvent::RecheckFinished);
        });
    }

    /// Certain IDE features (e.g. find-references) require us to know the dependency graph of the
    /// entire project to work. This blocking function should be called when we know that a project
    /// file is opened and if we intend to provide features like find-references, and should be
    /// called when config changes (currently this is a TODO).
    fn populate_all_project_files_in_config(
        config: ArcId<ConfigFile>,
        state: Arc<State>,
        lsp_queue: LspQueue,
    ) {
        let unknown = ModuleName::unknown();

        eprintln!("Populating all files in the config ({:?}).", config.root);
        let mut transaction = state.new_committable_transaction(Require::Indexing, None);

        let project_path_blobs = config.get_filtered_globs(None);
        let paths = project_path_blobs.files().unwrap_or_default();
        let mut handles = Vec::new();
        for path in paths {
            let module_path = ModulePath::filesystem(path.clone());
            let path_config = state.config_finder().python_file(unknown, &module_path);
            if config != path_config {
                continue;
            }
            let module_name = module_from_path(&path, path_config.search_path())
                .unwrap_or_else(ModuleName::unknown);
            handles.push((
                Handle::new(module_name, module_path, path_config.get_sys_info()),
                Require::Indexing,
            ));
        }

        eprintln!("Prepare to check {} files.", handles.len());
        transaction.as_mut().run(&handles);
        state.commit_transaction(transaction);
        // After we finished a recheck asynchronously, we immediately send `RecheckFinished` to
        // the main event loop of the server. As a result, the server can do a revalidation of
        // all the in-memory files based on the fresh main State as soon as possible.
        let _ = lsp_queue.send(LspEvent::RecheckFinished);
        eprintln!("Populated all files in the project path.");
    }

    fn did_save(&self, params: DidSaveTextDocumentParams) {
        let file = params.text_document.uri.to_file_path().unwrap();
        self.invalidate(move |t| t.invalidate_disk(&[file]));
    }

    fn did_open<'a>(
        &'a self,
        ide_transaction_manager: &mut TransactionManager<'a>,
        subsequent_mutation: bool,
        params: DidOpenTextDocumentParams,
    ) {
        let uri = params.text_document.uri.to_file_path().unwrap();
        let config_to_populate_files = if self.indexing_mode != IndexingMode::None
            && let Some(directory) = uri.as_path().parent()
        {
            self.state.config_finder().directory(directory)
        } else {
            None
        };
        self.version_info
            .lock()
            .insert(uri.clone(), params.text_document.version);
        self.open_files
            .write()
            .insert(uri, Arc::new(params.text_document.text));
        if !subsequent_mutation {
            self.validate_in_memory(ide_transaction_manager);
        }
        self.populate_project_files_if_necessary(config_to_populate_files);
        // rewatch files in case we loaded or dropped any configs
        self.setup_file_watcher_if_necessary();
    }

    fn did_change<'a>(
        &'a self,
        ide_transaction_manager: &mut TransactionManager<'a>,
        subsequent_mutation: bool,
        params: DidChangeTextDocumentParams,
    ) -> anyhow::Result<()> {
        let VersionedTextDocumentIdentifier { uri, version } = params.text_document;
        let file_path = uri.to_file_path().unwrap();

        let mut version_info = self.version_info.lock();
        let old_version = version_info.get(&file_path).unwrap_or(&0);
        if version < *old_version {
            return Err(anyhow::anyhow!(
                "Unexpected version in didChange notification: {version:?} is less than {old_version:?}"
            ));
        }
        version_info.insert(file_path.clone(), version);
        let mut lock = self.open_files.write();
        let original = lock.get_mut(&file_path).unwrap();
        *original = Arc::new(apply_change_events(
            original.as_str(),
            params.content_changes,
        ));
        drop(lock);
        if !subsequent_mutation {
            self.validate_in_memory(ide_transaction_manager);
        }
        Ok(())
    }

    fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        if !params.changes.is_empty() {
            self.invalidate(move |t| {
                t.invalidate_events(&CategorizedEvents::new_lsp(params.changes))
            });
        }
        // rewatch files in case we loaded or dropped any configs
        self.setup_file_watcher_if_necessary();
    }

    fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_file_path().unwrap();
        self.version_info.lock().remove(&uri);
        self.open_files.write().remove(&uri);
        self.connection
            .publish_diagnostics_for_uri(params.text_document.uri, Vec::new(), None);
        let state = self.state.dupe();
        let open_files = self.open_files.dupe();
        std::thread::spawn(move || {
            // Clear out the memory associated with this file.
            // Not a race condition because we immediately call validate_in_memory to put back the open files as they are now.
            // Having the extra file hanging around doesn't harm anything, but does use extra memory.
            let mut transaction = state.new_committable_transaction(Require::Indexing, None);
            transaction.as_mut().set_memory(vec![(uri, None)]);
            Self::validate_in_memory_for_transaction(&state, &open_files, transaction.as_mut());
            state.commit_transaction(transaction);
        });
    }

    fn workspace_folders_changed(&self, params: DidChangeWorkspaceFoldersParams) {
        self.workspaces.changed(params.event);
        self.setup_file_watcher_if_necessary();
        self.request_settings_for_all_workspaces();
    }

    fn did_change_configuration<'a>(
        &'a self,
        ide_transaction_manager: &mut TransactionManager<'a>,
        params: DidChangeConfigurationParams,
    ) {
        if let Some(workspace) = &self.initialize_params.capabilities.workspace
            && workspace.configuration == Some(true)
        {
            self.request_settings_for_all_workspaces();
            return;
        }

        let mut modified = false;
        if let Some(python) = params.settings.get(PYTHON_SECTION) {
            self.workspaces
                .apply_client_configuration(&mut modified, &None, python.clone());
        }

        if modified {
            // Once the config finishes changing, we'll recheck
            self.invalidate_config();
            // If disable_type_errors has changed, we want that to take effect immediately.
            self.validate_in_memory(ide_transaction_manager);
        }
    }

    fn workspace_configuration_response<'a>(
        &'a self,
        ide_transaction_manager: &mut TransactionManager<'a>,
        request: &ConfigurationParams,
        response: &[Value],
    ) {
        let mut modified = false;
        for (i, id) in request.items.iter().enumerate() {
            if let Some(value) = response.get(i) {
                self.workspaces.apply_client_configuration(
                    &mut modified,
                    &id.scope_uri,
                    value.clone(),
                );
            }
        }
        if modified {
            self.invalidate_config();
            // If disable_type_errors has changed, we want that to take effect immediately.
            self.validate_in_memory(ide_transaction_manager);
        }
    }

    /// Create a handle. Return None if the workspace has language services disabled (and thus you shouldn't do anything).
    fn make_handle_if_enabled(&self, uri: &Url) -> Option<Handle> {
        let path = uri.to_file_path().unwrap();
        self.workspaces.get_with(path.clone(), |workspace| {
            if workspace.disable_language_services {
                eprintln!("Skipping request - language services disabled");
                None
            } else {
                let module_path = if self.open_files.read().contains_key(&path) {
                    ModulePath::memory(path)
                } else {
                    ModulePath::filesystem(path)
                };
                Some(handle_from_module_path(&self.state, module_path))
            }
        })
    }

    fn goto_definition(
        &self,
        transaction: &Transaction<'_>,
        params: GotoDefinitionParams,
    ) -> Option<GotoDefinitionResponse> {
        let uri = &params.text_document_position_params.text_document.uri;
        let handle = self.make_handle_if_enabled(uri)?;
        let info = transaction.get_module_info(&handle)?;
        let range = info
            .lined_buffer()
            .from_lsp_position(params.text_document_position_params.position);
        let targets = transaction.goto_definition(&handle, range);
        let mut lsp_targets = targets
            .iter()
            .filter_map(to_lsp_location)
            .collect::<Vec<_>>();
        if lsp_targets.is_empty() {
            None
        } else if lsp_targets.len() == 1 {
            Some(GotoDefinitionResponse::Scalar(lsp_targets.pop().unwrap()))
        } else {
            Some(GotoDefinitionResponse::Array(lsp_targets))
        }
    }

    fn completion(
        &self,
        transaction: &Transaction<'_>,
        params: CompletionParams,
    ) -> anyhow::Result<CompletionResponse> {
        let uri = &params.text_document_position.text_document.uri;
        let handle = match self.make_handle_if_enabled(uri) {
            None => {
                return Ok(CompletionResponse::List(CompletionList {
                    is_incomplete: false,
                    items: Vec::new(),
                }));
            }
            Some(x) => x,
        };
        let items = transaction
            .get_module_info(&handle)
            .map(|info| {
                transaction.completion(
                    &handle,
                    info.lined_buffer()
                        .from_lsp_position(params.text_document_position.position),
                )
            })
            .unwrap_or_default();
        Ok(CompletionResponse::List(CompletionList {
            is_incomplete: false,
            items,
        }))
    }

    fn code_action(
        &self,
        transaction: &Transaction<'_>,
        params: CodeActionParams,
    ) -> Option<CodeActionResponse> {
        let uri = &params.text_document.uri;
        let handle = self.make_handle_if_enabled(uri)?;
        let module_info = transaction.get_module_info(&handle)?;
        let range = module_info.lined_buffer().from_lsp_range(params.range);
        let code_actions = transaction
            .local_quickfix_code_actions(&handle, range)?
            .into_map(|(title, info, range, insert_text)| {
                CodeActionOrCommand::CodeAction(CodeAction {
                    title,
                    kind: Some(CodeActionKind::QUICKFIX),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(
                            uri.clone(),
                            vec![TextEdit {
                                range: info.lined_buffer().to_lsp_range(range),
                                new_text: insert_text,
                            }],
                        )])),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            });
        Some(code_actions)
    }

    fn document_highlight(
        &self,
        transaction: &Transaction<'_>,
        params: DocumentHighlightParams,
    ) -> Option<Vec<DocumentHighlight>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let handle = self.make_handle_if_enabled(uri)?;
        let info = transaction.get_module_info(&handle)?;
        let position = info
            .lined_buffer()
            .from_lsp_position(params.text_document_position_params.position);
        Some(
            transaction
                .find_local_references(&handle, position)
                .into_map(|range| DocumentHighlight {
                    range: info.lined_buffer().to_lsp_range(range),
                    kind: None,
                }),
        )
    }

    /// Compute references of a symbol at a given position. This is a non-blocking function, the
    /// it will send a response to the LSP client once the results are found and transformed by
    /// `map_result`.
    fn async_find_references_helper<'a, V: serde::Serialize>(
        &'a self,
        request_id: RequestId,
        ide_transaction_manager: &mut TransactionManager<'a>,
        uri: &Url,
        position: Position,
        map_result: impl FnOnce(Vec<(Url, Vec<Range>)>) -> V + Send + 'static,
    ) {
        let Some(handle) = self.make_handle_if_enabled(uri) else {
            return self.send_response(new_response::<Option<V>>(request_id, Ok(None)));
        };
        let transaction = ide_transaction_manager.non_commitable_transaction(&self.state);
        let Some(info) = transaction.get_module_info(&handle) else {
            ide_transaction_manager.save(transaction);
            return self.send_response(new_response::<Option<V>>(request_id, Ok(None)));
        };
        let position = info.lined_buffer().from_lsp_position(position);
        let Some(FindDefinitionItemWithDocstring {
            metadata,
            definition_range,
            module,
            docstring_range: _,
        }) = transaction
            .find_definition(&handle, position, false)
            // TODO: handle more than 1 definition
            .into_iter()
            .next()
        else {
            ide_transaction_manager.save(transaction);
            return self.send_response(new_response::<Option<V>>(request_id, Ok(None)));
        };
        ide_transaction_manager.save(transaction);
        let state = self.state.dupe();
        let open_files = self.open_files.dupe();
        let cancellation_handles = self.cancellation_handles.dupe();

        let connection = self.connection.dupe();
        self.async_state_read_threads.async_spawn(move || {
            let mut transaction = state.cancellable_transaction();
            cancellation_handles
                .lock()
                .insert(request_id.clone(), transaction.get_cancellation_handle());
            Self::validate_in_memory_for_transaction(&state, &open_files, transaction.as_mut());
            match transaction.find_global_references_from_definition(
                handle.sys_info(),
                metadata,
                TextRangeWithModule::new(module, definition_range),
            ) {
                Ok(global_references) => {
                    let mut locations = Vec::new();
                    for (info, ranges) in global_references {
                        if let Some(uri) = module_info_to_uri(&info) {
                            locations.push((
                                uri,
                                ranges.into_map(|range| info.lined_buffer().to_lsp_range(range)),
                            ));
                        };
                    }
                    cancellation_handles.lock().remove(&request_id);
                    connection.send(Message::Response(new_response(
                        request_id,
                        Ok(Some(map_result(locations))),
                    )));
                }
                Err(Cancelled) => {
                    let message = format!("Find reference request {request_id} is canceled");
                    eprintln!("{message}");
                    connection.send(Message::Response(Response::new_err(
                        request_id,
                        ErrorCode::RequestCanceled as i32,
                        message,
                    )))
                }
            }
        });
    }

    fn references<'a>(
        &'a self,
        request_id: RequestId,
        ide_transaction_manager: &mut TransactionManager<'a>,
        params: ReferenceParams,
    ) {
        self.async_find_references_helper(
            request_id,
            ide_transaction_manager,
            &params.text_document_position.text_document.uri,
            params.text_document_position.position,
            move |results| {
                let mut locations = Vec::new();
                for (uri, ranges) in results {
                    for range in ranges {
                        locations.push(Location {
                            uri: uri.clone(),
                            range,
                        })
                    }
                }
                locations
            },
        );
    }

    fn rename<'a>(
        &'a self,
        request_id: RequestId,
        ide_transaction_manager: &mut TransactionManager<'a>,
        params: RenameParams,
    ) {
        self.async_find_references_helper(
            request_id,
            ide_transaction_manager,
            &params.text_document_position.text_document.uri,
            params.text_document_position.position,
            move |results| {
                let mut changes = HashMap::new();
                for (uri, ranges) in results {
                    changes.insert(
                        uri,
                        ranges.into_map(|range| TextEdit {
                            range,
                            new_text: params.new_name.clone(),
                        }),
                    );
                }
                WorkspaceEdit {
                    changes: Some(changes),
                    ..Default::default()
                }
            },
        );
    }

    fn prepare_rename(
        &self,
        transaction: &Transaction<'_>,
        params: TextDocumentPositionParams,
    ) -> Option<PrepareRenameResponse> {
        let uri = &params.text_document.uri;
        let handle = self.make_handle_if_enabled(uri)?;
        let info = transaction.get_module_info(&handle)?;
        let position = info.lined_buffer().from_lsp_position(params.position);
        transaction
            .prepare_rename(&handle, position)
            .map(|range| PrepareRenameResponse::Range(info.lined_buffer().to_lsp_range(range)))
    }

    fn signature_help(
        &self,
        transaction: &Transaction<'_>,
        params: SignatureHelpParams,
    ) -> Option<SignatureHelp> {
        let uri = &params.text_document_position_params.text_document.uri;
        let handle = self.make_handle_if_enabled(uri)?;
        let info = transaction.get_module_info(&handle)?;
        let position = info
            .lined_buffer()
            .from_lsp_position(params.text_document_position_params.position);
        transaction.get_signature_help_at(&handle, position)
    }

    fn hover(&self, transaction: &Transaction<'_>, params: HoverParams) -> Option<Hover> {
        let uri = &params.text_document_position_params.text_document.uri;
        let handle = self.make_handle_if_enabled(uri)?;
        let info = transaction.get_module_info(&handle)?;
        let position = info
            .lined_buffer()
            .from_lsp_position(params.text_document_position_params.position);

        get_hover(transaction, &handle, position)
    }

    fn inlay_hints(
        &self,
        transaction: &Transaction<'_>,
        params: InlayHintParams,
    ) -> Option<Vec<InlayHint>> {
        let uri = &params.text_document.uri;
        let range = &params.range;
        let handle = self.make_handle_if_enabled(uri)?;
        let info = transaction.get_module_info(&handle)?;
        let t = transaction.inlay_hints(&handle)?;
        let res = t
            .into_iter()
            .filter_map(|x| {
                let position = info.lined_buffer().to_lsp_position(x.0);
                // The range is half-open, so the end position is exclusive according to the spec.
                if position >= range.start && position < range.end {
                    Some(InlayHint {
                        position,
                        label: InlayHintLabel::String(x.1.clone()),
                        kind: None,
                        text_edits: Some(vec![TextEdit {
                            range: Range::new(position, position),
                            new_text: x.1,
                        }]),
                        tooltip: None,
                        padding_left: None,
                        padding_right: None,
                        data: None,
                    })
                } else {
                    None
                }
            })
            .collect();
        Some(res)
    }

    fn semantic_tokens_full(
        &self,
        transaction: &Transaction<'_>,
        params: SemanticTokensParams,
    ) -> Option<SemanticTokensResult> {
        let uri = &params.text_document.uri;
        let handle = self.make_handle_if_enabled(uri)?;
        Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: transaction
                .semantic_tokens(&handle, None)
                .unwrap_or_default(),
        }))
    }

    fn semantic_tokens_ranged(
        &self,
        transaction: &Transaction<'_>,
        params: SemanticTokensRangeParams,
    ) -> Option<SemanticTokensRangeResult> {
        let uri = &params.text_document.uri;
        let handle = self.make_handle_if_enabled(uri)?;
        let module_info = transaction.get_module_info(&handle)?;
        let range = module_info.lined_buffer().from_lsp_range(params.range);
        Some(SemanticTokensRangeResult::Tokens(SemanticTokens {
            result_id: None,
            data: transaction
                .semantic_tokens(&handle, Some(range))
                .unwrap_or_default(),
        }))
    }

    fn hierarchical_document_symbols(
        &self,
        transaction: &Transaction<'_>,
        params: DocumentSymbolParams,
    ) -> Option<Vec<DocumentSymbol>> {
        let uri = &params.text_document.uri;
        if self
            .workspaces
            .get_with(uri.to_file_path().unwrap(), |workspace| {
                workspace.disable_language_services
            })
            || !self
                .initialize_params
                .capabilities
                .text_document
                .as_ref()?
                .document_symbol
                .as_ref()?
                .hierarchical_document_symbol_support?
        {
            return None;
        }
        let handle = self.make_handle_if_enabled(uri)?;
        transaction.symbols(&handle)
    }

    #[allow(deprecated)] // The `deprecated` field
    fn workspace_symbols(
        &self,
        transaction: &Transaction<'_>,
        query: &str,
    ) -> Vec<SymbolInformation> {
        transaction
            .workspace_symbols(query)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(name, kind, location)| {
                to_lsp_location(&location).map(|location| SymbolInformation {
                    name,
                    kind,
                    location,
                    tags: None,
                    deprecated: None,
                    container_name: None,
                })
            })
            .collect()
    }

    fn document_diagnostics(
        &self,
        transaction: &Transaction<'_>,
        params: DocumentDiagnosticParams,
    ) -> DocumentDiagnosticReport {
        let handle = make_open_handle(
            &self.state,
            &params.text_document.uri.to_file_path().unwrap(),
        );
        let mut items = Vec::new();
        let open_files = &self.open_files.read();
        for e in transaction.get_errors(once(&handle)).collect_errors().shown {
            if let Some((_, diag)) = self.get_diag_if_shown(&e, open_files) {
                items.push(diag);
            }
        }
        DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                items,
                result_id: None,
            },
            related_documents: None,
        })
    }

    fn get_pattern_to_watch(
        root: &Path,
        pattern: String,
        relative_pattern_support: bool,
    ) -> GlobPattern {
        if relative_pattern_support && let Ok(url) = Url::from_directory_path(root) {
            GlobPattern::Relative(RelativePattern {
                base_uri: OneOf::Right(url),
                pattern,
            })
        } else {
            GlobPattern::String(root.join(pattern).to_string_lossy().into_owned())
        }
    }

    fn setup_file_watcher_if_necessary(&self) {
        let roots = self.workspaces.roots();
        match self.initialize_params.capabilities.workspace {
            Some(WorkspaceClientCapabilities {
                did_change_watched_files:
                    Some(DidChangeWatchedFilesClientCapabilities {
                        dynamic_registration: Some(true),
                        relative_pattern_support,
                        ..
                    }),
                ..
            }) => {
                let relative_pattern_support = relative_pattern_support.is_some_and(|b| b);
                if self.filewatcher_registered.load(Ordering::Relaxed) {
                    self.send_request::<UnregisterCapability>(UnregistrationParams {
                        unregisterations: Vec::from([Unregistration {
                            id: Self::FILEWATCHER_ID.to_owned(),
                            method: DidChangeWatchedFiles::METHOD.to_owned(),
                        }]),
                    });
                }
                // TODO(connernilsen): we need to dedup filewatcher patterns
                // preferably by figuring out if they're under another wildcard pattern with the same suffix
                let mut glob_patterns = Vec::new();
                for root in &roots {
                    PYTHON_EXTENSIONS.iter().for_each(|suffix| {
                        glob_patterns.push(Self::get_pattern_to_watch(
                            root,
                            format!("**/*.{suffix}"),
                            relative_pattern_support,
                        ));
                    });
                    ConfigFile::CONFIG_FILE_NAMES.iter().for_each(|config| {
                        glob_patterns.push(Self::get_pattern_to_watch(
                            root,
                            format!("**/{config}"),
                            relative_pattern_support,
                        ))
                    });
                }
                for (root, pattern) in self
                    .workspaces
                    .loaded_configs
                    .get_patterns_for_cached_configs()
                {
                    glob_patterns.push(Self::get_pattern_to_watch(
                        &root,
                        pattern,
                        relative_pattern_support,
                    ));
                }
                let watchers = glob_patterns
                    .into_iter()
                    .map(|glob_pattern| FileSystemWatcher {
                        glob_pattern,
                        kind: Some(WatchKind::Create | WatchKind::Change | WatchKind::Delete),
                    })
                    .collect::<Vec<_>>();
                self.send_request::<RegisterCapability>(RegistrationParams {
                    registrations: Vec::from([Registration {
                        id: Self::FILEWATCHER_ID.to_owned(),
                        method: DidChangeWatchedFiles::METHOD.to_owned(),
                        register_options: Some(
                            serde_json::to_value(DidChangeWatchedFilesRegistrationOptions {
                                watchers,
                            })
                            .unwrap(),
                        ),
                    }]),
                });
                self.filewatcher_registered.store(true, Ordering::Relaxed);
            }
            _ => (),
        }
    }

    fn request_settings_for_all_workspaces(&self) {
        if let Some(workspace) = &self.initialize_params.capabilities.workspace
            && workspace.configuration == Some(true)
        {
            let roots = self.workspaces.roots();
            self.send_request::<WorkspaceConfiguration>(ConfigurationParams {
                items: roots
                    .iter()
                    .map(|uri| Some(Url::from_file_path(uri).unwrap()))
                    // add default workspace
                    .chain(once(None))
                    .map(|url| ConfigurationItem {
                        scope_uri: url,
                        section: Some(PYTHON_SECTION.to_owned()),
                    })
                    .collect::<Vec<_>>(),
            });
        }
    }

    fn invalidate_config(&self) {
        self.invalidate(|t| t.invalidate_config());
    }
}
