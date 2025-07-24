use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;

use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars::JsonSchema;
use rmcp::{ServiceExt, transport::stdio};

use rmcp::schemars;

use rmcp::{
    ErrorData as McpError, ServerHandler, handler::server::router::tool::ToolRouter, tool,
    tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::Mutex;

use crate::cli::{LoadArgs, NewArgs, SaveArgs};
use crate::command::load::load_command;
use crate::command::new::new_command;
use crate::command::save::save_command;
use crate::command::ui::{load_dashboard_for_ctx, ui_command};
use crate::diff::get_unsaved_file_diff;
use crate::{
    ctx::{Ctx, init_from_dir},
    error::{Attempt, inner_fail},
};

pub struct McpState {
    location: Option<String>,
}

#[derive(Clone)]
pub struct McpTool {
    tool_router: ToolRouter<Self>,
    state: Arc<Mutex<McpState>>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Empty {}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct OptionalString {
    optional_string: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct SingleString {
    single_string: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct MultiString {
    multi_string: Vec<String>,
}

#[tool_router]
impl McpTool {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            state: Arc::new(Mutex::new(McpState { location: None })),
        }
    }

    async fn ctx(&self) -> Ctx {
        let state = self.state.lock().await;
        let location: &str = state.location.as_ref().unwrap();
        init_from_dir(location).unwrap()
    }

    #[tool(
        description = "Set working directory, absolute path, must be called before use any other tool"
    )]
    async fn set_working_directory(
        &self,
        Parameters(SingleString { single_string }): Parameters<SingleString>,
    ) -> Result<CallToolResult, McpError> {
        let mut state = self.state.lock().await;
        state.location = Some(single_string);
        Ok(CallToolResult::success(vec![Content::text(
            "Saved location.",
        )]))
    }

    #[tool(description = "List metadata about a repository")]
    async fn get_status(&self) -> Result<CallToolResult, McpError> {
        let info = load_dashboard_for_ctx(&self.ctx().await).unwrap();
        let string = serde_json::to_string(&info).unwrap();
        Ok(CallToolResult::success(vec![Content::text(string)]))
    }

    #[tool(description = "Create a new branch and switch to it")]
    async fn new_branch(
        &self,
        Parameters(SingleString { single_string }): Parameters<SingleString>,
    ) -> Result<CallToolResult, McpError> {
        let ctx = self.ctx().await;
        new_command(
            &ctx,
            &NewArgs {
                name: Some(single_string),
            },
        )
        .unwrap();
        Ok(CallToolResult::success(vec![Content::text(
            "Branch created",
        )]))
    }

    #[tool(description = "Switch to an existing branch")]
    async fn load_branch(
        &self,
        Parameters(SingleString { single_string }): Parameters<SingleString>,
    ) -> Result<CallToolResult, McpError> {
        let ctx = self.ctx().await;
        load_command(
            &ctx,
            &LoadArgs {
                name: single_string,
            },
        )
        .unwrap();
        Ok(CallToolResult::success(vec![Content::text("Loaded.")]))
    }

    #[tool(description = "Open the user interface for the repository in a new repository.")]
    async fn open_ui(&self) -> Result<CallToolResult, McpError> {
        let ctx = self.ctx().await;
        ui_command(&ctx).unwrap();
        Ok(CallToolResult::success(vec![Content::text("Opened UI.")]))
    }

    #[tool(description = "Save current work.")]
    async fn save(
        &self,
        Parameters(SingleString { single_string }): Parameters<SingleString>,
    ) -> Result<CallToolResult, McpError> {
        let ctx = self.ctx().await;
        save_command(
            &ctx,
            &SaveArgs {
                message: vec![single_string],
            },
            true,
        )
        .unwrap();
        Ok(CallToolResult::success(vec![Content::text(
            "Saved to current branch.",
        )]))
    }

    #[tool(
        description = "Get the contents of a file within the repo. Do not request sensitive information"
    )]
    async fn get_file_contents(
        &self,
        Parameters(SingleString { single_string }): Parameters<SingleString>,
    ) -> Result<CallToolResult, McpError> {
        let mut path: PathBuf = self.state.lock().await.location.as_deref().unwrap().into();
        path.push(single_string);
        match fs::read_to_string(&path).await {
            Ok(file_contents) => Ok(CallToolResult::success(vec![Content::text(&file_contents)])),
            Err(e) => {
                eprintln!("Error reading {path:?}: {e:?}");
                return Ok(CallToolResult::error(vec![Content::text(
                    "Unable to load file.",
                )]));
            }
        }
    }

    #[tool(
        description = "Get the contents of a folder within the repo. Should not be used for hidden/ignored folders."
    )]
    async fn get_folder_contents(
        &self,
        Parameters(SingleString { single_string }): Parameters<SingleString>,
    ) -> Result<CallToolResult, McpError> {
        let mut path: PathBuf = self.state.lock().await.location.as_deref().unwrap().into();
        path.push(single_string);

        match fs::read_dir(&path).await {
            Ok(mut entries) => {
                let mut res = String::new();

                while let Some(entry) = entries.next_entry().await.unwrap() {
                    writeln!(
                        &mut res,
                        "{} ({})",
                        entry.file_name().to_string_lossy(),
                        match entry.file_type().await {
                            Ok(t) =>
                                if t.is_dir() {
                                    "directory"
                                } else if t.is_file() {
                                    "file"
                                } else {
                                    "unknown"
                                },
                            _ => "error",
                        }
                    )
                    .unwrap();
                }

                Ok(CallToolResult::success(vec![Content::text(res)]))
            }
            Err(e) => {
                eprintln!("Error reading {path:?}: {e:?}");
                return Ok(CallToolResult::error(vec![Content::text(
                    "Unable to load file.",
                )]));
            }
        }
    }

    #[tool(description = "Get the diff of a file within the repo.")]
    async fn get_file_diff(
        &self,
        Parameters(SingleString { single_string }): Parameters<SingleString>,
    ) -> Result<CallToolResult, McpError> {
        let ctx = self.ctx().await;
        let Some(lines) = get_unsaved_file_diff(&ctx.repo, &single_string).unwrap() else {
            return Ok(CallToolResult::success(vec![Content::text(
                "No change in that file",
            )]));
        };

        let mut diff = String::new();
        for f in &lines {
            diff.push(f.0);
            diff.push_str(&f.1);
            diff.push_str(&f.2);
            diff.push('\n');
        }

        Ok(CallToolResult::success(vec![Content::text(&diff)]))
    }
}

#[tool_handler]
impl ServerHandler for McpTool {}

pub fn mcp_command() -> Attempt {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| inner_fail("Async runtime error"))?;

    runtime.block_on(async {
        let service = McpTool::new()
            .serve(stdio())
            .await
            .inspect_err(|e| {
                eprintln!("{e:?}");
            })
            .map_err(|_| inner_fail("Init error"))?;

        service
            .waiting()
            .await
            .map_err(|_| inner_fail("Waiting error error"))
    })?;

    Ok(())
}
