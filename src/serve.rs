//! Serving a tracker over MCP.
//!
//! The same library the command line reads is what a client reads.
//!
//! Serving changes what a caller may do. A tracker holds the stated
//! intent of the people who use it, so a remote caller reads by
//! default. Where writes are allowed, they are the writes an agent
//! should make: appending to an issue's working notes, never closing an
//! issue or rewriting its body. Deciding an issue is done is a judgment
//! its owner makes.

use std::path::Path;
use std::sync::Arc;

use camino::Utf8PathBuf;
use mdstore::mcp::{Access, DocUri, ServeConfig, Surface};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ErrorData as McpError,
    Implementation, InitializeResult, ListResourcesResult, ListToolsResult,
    PaginatedRequestParams, ProtocolVersion, ReadResourceRequestParams, ReadResourceResponse,
    ReadResourceResult, Resource, ResourceContents, ResourcesCapability, ServerCapabilities, Tool,
    ToolsCapability,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler};
use serde_json::{Map, Value, json};

use crate::error::{Error, Result};
use crate::workspace::Workspace;

const SCHEME: &str = "tisket";
const KIND: &str = "issue";

/// A served tracker.
#[derive(Clone)]
pub struct TisketServer {
    config: Arc<ServeConfig>,
}

impl TisketServer {
    #[must_use]
    pub fn new(config: ServeConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    fn root(&self) -> Utf8PathBuf {
        Utf8PathBuf::from_path_buf(self.config.root.clone()).unwrap_or_default()
    }

    fn workspace(&self) -> Result<Workspace> {
        Workspace::open(&self.root())
    }

    fn uri_of(qualified_id: &str, aliases: &[String]) -> String {
        DocUri::new(
            SCHEME,
            KIND,
            mdstore::store::StoreRef::parse(qualified_id, &aliases.to_vec()),
        )
        .to_string()
    }

    fn issue_id_of(&self, uri: &str) -> Result<String> {
        let ws = self.workspace()?;
        let aliases = ws.aliases();
        let parsed = DocUri::parse(uri, &aliases).map_err(|e| Error::Store(e.to_string()))?;
        if parsed.scheme != SCHEME || parsed.kind != KIND {
            return Err(Error::IssueNotFound(uri.to_string()));
        }
        Ok(parsed.reference.to_string())
    }

    fn tool_definitions(&self) -> Vec<Tool> {
        let mut tools = vec![
            Tool::new(
                "tisket_list_issues",
                "List the issues this tracker holds, with status, labels, and assignee.",
                Arc::new(schema_with(&[(
                    "status",
                    "Only issues with this status.",
                    false,
                )])),
            ),
            Tool::new(
                "tisket_read_issue",
                "Return one issue by ID: its frontmatter, its body, and its working notes.",
                Arc::new(schema_with(&[("id", "The issue ID.", true)])),
            ),
            Tool::new(
                "tisket_rollup",
                "Return the children of an epic and their statuses, across trackers. A \
                 child in a tracker that cannot be read is reported as unknown, never \
                 dropped.",
                Arc::new(schema_with(&[("id", "The epic's issue ID.", true)])),
            ),
            Tool::new(
                "tisket_check",
                "Report the problems in this tracker: a reference that names no issue, an \
                 unreachable tracker, a cycle in the children of an epic.",
                Arc::new(schema_with(&[])),
            ),
        ];
        if self.config.access.writable() {
            // Appending to working notes is the write an agent should
            // make: it records what was found without deciding
            // anything. Closing an issue or rewriting its body is a
            // judgment its owner makes at the command line.
            tools.push(Tool::new(
                "tisket_append_scratch",
                "Append to an issue's working notes. This is the only write a served \
                 tracker allows: it records what was found without deciding anything.",
                Arc::new(schema_with(&[
                    ("id", "The issue ID.", true),
                    ("text", "The text to append.", true),
                ])),
            ));
        }
        tools
    }

    fn call(&self, name: &str, args: &Map<String, Value>) -> Result<String> {
        let text = |key: &str| args.get(key).and_then(Value::as_str).map(str::to_string);
        match name {
            "tisket_list_issues" => {
                let repo = crate::Repo::open(&self.root())?;
                // A terminal status lives in the closed directory, so
                // asking for it while looking only at open issues
                // returned an empty list.
                let status = text("status");
                let closed = match status.as_deref() {
                    Some(text) => text
                        .parse::<crate::issue::Status>()
                        .map_err(|_| {
                            Error::Store(format!(
                                "'{text}' is not a status; use todo, in_progress, done, or cancelled"
                            ))
                        })?
                        .is_terminal(),
                    None => false,
                };
                let issues = repo.list_issues(None, status.as_deref(), None, closed, &[])?;
                let listed: Vec<Value> = issues
                    .iter()
                    .map(|i| {
                        json!({
                            "id": i.id,
                            "title": i.frontmatter.title,
                            "status": i.frontmatter.status.to_string(),
                            "labels": i.frontmatter.labels,
                            "children": i.frontmatter.children,
                        })
                    })
                    .collect();
                Ok(pretty(&listed))
            }
            "tisket_read_issue" => {
                let id = text("id").ok_or_else(|| Error::IssueNotFound("id is required".into()))?;
                // An ID that names another tracker goes through the
                // workspace. A bare ID stays on the repository, which
                // also holds the closed issues.
                let ws = self.workspace()?;
                let repo = crate::Repo::open(&self.root())?;
                let owned: Option<crate::issue::Issue>;
                let (qualified, issue) = if ws.is_qualified(&id) {
                    let view = ws.find(&id)?;
                    (view.qualified.clone(), view.issue)
                } else {
                    owned = Some(repo.find_issue(&id)?);
                    let issue = owned.as_ref().expect("just assigned");
                    (issue.id.clone(), issue)
                };
                Ok(pretty(&json!({
                    "id": qualified,
                    "title": issue.frontmatter.title,
                    "status": issue.frontmatter.status.to_string(),
                    "labels": issue.frontmatter.labels,
                    "depends_on": issue.frontmatter.depends_on,
                    "children": issue.frontmatter.children,
                    "body": issue.body,
                    "scratch": issue.scratch,
                })))
            }
            "tisket_rollup" => {
                let id = text("id").ok_or_else(|| Error::IssueNotFound("id is required".into()))?;
                let ws = self.workspace()?;
                let view = ws.find(&id)?;
                Ok(pretty(&ws.rollup(&view)))
            }
            "tisket_check" => {
                let ws = self.workspace()?;
                let findings = ws.check(&self.root());
                if findings.is_empty() {
                    return Ok("no problems found".into());
                }
                Ok(pretty(&findings))
            }
            "tisket_append_scratch" => {
                self.config
                    .ensure_writable()
                    .map_err(|e| Error::Store(e.to_string()))?;
                let id = text("id").ok_or_else(|| Error::IssueNotFound("id is required".into()))?;
                let body =
                    text("text").ok_or_else(|| Error::IssueNotFound("text is required".into()))?;
                let repo = crate::Repo::open(&self.root())?;
                repo.scratch_append(&id, &body)?;
                Ok(format!("appended to the working notes of {id}"))
            }
            other => Err(Error::Store(format!("no tool named '{other}'"))),
        }
    }
}

fn pretty<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default()
}

fn schema_with(fields: &[(&str, &str, bool)]) -> Map<String, Value> {
    let mut properties = Map::new();
    let mut required = Vec::new();
    for (name, description, is_required) in fields {
        properties.insert(
            (*name).to_string(),
            json!({ "type": "string", "description": description }),
        );
        if *is_required {
            required.push(json!(name));
        }
    }
    let mut schema = Map::new();
    schema.insert("type".into(), json!("object"));
    schema.insert("properties".into(), Value::Object(properties));
    if !required.is_empty() {
        schema.insert("required".into(), Value::Array(required));
    }
    schema
}

fn to_mcp_error(e: &Error) -> McpError {
    McpError::internal_error(e.to_string(), None)
}

impl ServerHandler for TisketServer {
    fn get_info(&self) -> InitializeResult {
        let mut capabilities = ServerCapabilities::default();
        if self.config.surfaces.has(Surface::Tools) {
            capabilities.tools = Some(ToolsCapability::default());
        }
        if self.config.surfaces.has(Surface::Resources) {
            capabilities.resources = Some(ResourcesCapability::default());
        }

        let mut instructions = format!(
            "An issue tracker. Surfaces: {}. An issue body holds the stated intent of the \
             person who wrote it; treat it as a requirement, not a suggestion.",
            self.config.surfaces
        );
        if self.config.access == Access::ReadOnly {
            instructions.push_str(" This tracker is read-only.");
        } else {
            instructions.push_str(
                " You may append to an issue's working notes. Closing an issue and \
                 rewriting its body are decisions its owner makes.",
            );
        }

        InitializeResult::new(capabilities)
            .with_protocol_version(ProtocolVersion::default())
            .with_server_info({
                let mut info = Implementation::from_build_env();
                info.name.clone_from(&self.config.server_name);
                info.version = env!("CARGO_PKG_VERSION").to_string();
                info
            })
            .with_instructions(instructions)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListToolsResult, McpError> {
        if !self.config.surfaces.has(Surface::Tools) {
            return Ok(ListToolsResult::default());
        }
        Ok(ListToolsResult::with_all_items(self.tool_definitions()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<CallToolResponse, McpError> {
        if !self.config.surfaces.has(Surface::Tools) {
            return Err(McpError::invalid_request("tools are not served here", None));
        }
        let args = request.arguments.unwrap_or_default();
        // Every tool reads files. On the async worker pool that work
        // blocks the runtime, so one slow call delays every other
        // client's requests.
        let this = self.clone();
        let name = request.name.to_string();
        let called = tokio::task::spawn_blocking(move || this.call(&name, &args))
            .await
            .map_err(|e| McpError::internal_error(format!("the call did not finish: {e}"), None))?;
        match called {
            Ok(text) => Ok(CallToolResult::success(vec![ContentBlock::text(text)]).into()),
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(e.to_string())]).into()),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListResourcesResult, McpError> {
        if !self.config.surfaces.has(Surface::Resources) {
            return Ok(ListResourcesResult::default());
        }
        let repo = crate::Repo::open(&self.root()).map_err(|e| to_mcp_error(&e))?;
        let issues = repo
            .list_issues(None, None, None, false, &[])
            .map_err(|e| to_mcp_error(&e))?;
        let resources = issues
            .iter()
            .map(|i| {
                let mut resource =
                    Resource::new(Self::uri_of(&i.id, &[]), i.frontmatter.title.clone());
                resource.mime_type = Some("text/markdown".into());
                resource.description = Some(format!("status: {}", i.frontmatter.status));
                resource
            })
            .collect();
        Ok(ListResourcesResult::with_all_items(resources))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ReadResourceResponse, McpError> {
        if !self.config.surfaces.has(Surface::Resources) {
            return Err(McpError::invalid_request(
                "resources are not served here",
                None,
            ));
        }
        let id = self.issue_id_of(&request.uri).map_err(|e| to_mcp_error(&e))?;
        let mut args = Map::new();
        args.insert("id".into(), json!(id));
        let text = self
            .call("tisket_read_issue", &args)
            .map_err(|e| to_mcp_error(&e))?;
        Ok(ReadResourceResult::new(vec![ResourceContents::text(text, request.uri)]).into())
    }
}

/// Serve a tracker, on stdio or over HTTP.
pub fn run(config: ServeConfig, bind: Option<&str>) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::Store(e.to_string()))?;
    runtime.block_on(async move {
        match bind {
            None => serve_stdio(config).await,
            Some(addr) => serve_http(config, addr).await,
        }
    })
}

async fn serve_stdio(config: ServeConfig) -> Result<()> {
    use rmcp::ServiceExt;
    let service = TisketServer::new(config)
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| Error::Store(e.to_string()))?;
    service
        .waiting()
        .await
        .map_err(|e| Error::Store(e.to_string()))?;
    Ok(())
}

async fn serve_http(config: ServeConfig, addr: &str) -> Result<()> {
    use rmcp::transport::streamable_http_server::{
        StreamableHttpService, session::local::LocalSessionManager,
    };

    let read_only = config.access == Access::ReadOnly;
    let server = TisketServer::new(config);
    let service = StreamableHttpService::new(
        move || Ok(server.clone()),
        LocalSessionManager::default().into(),
        rmcp::transport::streamable_http_server::StreamableHttpServerConfig::default(),
    );
    let router = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| Error::Store(e.to_string()))?;
    let bound = listener
        .local_addr()
        .map_err(|e| Error::Store(e.to_string()))?;
    eprintln!(
        "tisket serving on http://{bound}/mcp ({})",
        if read_only { "read-only" } else { "read-write" }
    );
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .map_err(|e| Error::Store(e.to_string()))?;
    Ok(())
}

/// Build a serve configuration from the command line.
pub fn config_from(
    root: &Path,
    surfaces: &str,
    access: &str,
    server_name: &str,
) -> Result<ServeConfig> {
    let surfaces =
        mdstore::mcp::Surfaces::parse_list(surfaces).map_err(|e| Error::Store(e.to_string()))?;
    let access: Access = access.parse().map_err(|e: mdstore::Error| Error::Store(e.to_string()))?;
    let mut config = ServeConfig::read_only(root, surfaces, server_name);
    config.access = access;
    Ok(config)
}
