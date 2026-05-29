use crate::conversation::Conversation;
use crate::hooks::{DynHook, HookRunner};
#[cfg(not(target_arch = "wasm32"))]
use crate::local::LocalConnectionStrategy;
use crate::policy::{self, Policy, PolicyEnforcer};
use crate::tools::{DynTool, ToolRunner};
use crate::triggers::{DynTrigger, TriggerRunner};
use crate::types::{
    BuiltinTools, CapabilitiesConfig, ChatResponse, GeminiConfig, McpServerConfig,
    SystemInstructions,
};
use anyhow::anyhow;
use futures_util::future::BoxFuture;
use std::sync::Arc;

/// Configuration settings used to customize the behavior and capabilities of an [`Agent`].
#[derive(Default)]
pub struct AgentConfig {
    /// Optional path to the `localharness` binary. If not provided, it will be automatically
    /// resolved via standard paths or standard environments.
    pub binary_path: Option<String>,
    /// Gemini LLM configuration details (API key, default models, thinking settings, etc.).
    pub gemini_config: GeminiConfig,
    /// Capabilities config specifying enabled/disabled tools and threshold limits.
    pub capabilities: CapabilitiesConfig,
    /// Optional system instructions (either appended template sections or fully custom text).
    pub system_instructions: Option<SystemInstructions>,
    /// Optional directory to save session state logs.
    pub save_dir: Option<String>,
    /// Configured workspaces. If not provided, defaults to the current working directory.
    pub workspaces: Option<Vec<String>>,
    /// Paths to local folders containing custom skill modules.
    pub skills_paths: Vec<String>,
    /// Set of safety policies (e.g., workspace lock, run command approvals) to restrict tool execution.
    pub policies: Option<Vec<Policy>>,
    /// Handlers triggered during agent lifecycle hooks (pre/post tool calls, start session, etc.).
    pub hooks: Vec<Arc<dyn DynHook>>,
    /// Custom triggers spawned when the agent starts.
    pub triggers: Vec<Arc<dyn DynTrigger>>,
    /// Custom Rust tools registered to be available for invocation.
    pub tools: Vec<Arc<dyn DynTool>>,
    /// Specific conversation ID to assign or resume.
    pub conversation_id: Option<String>,
    /// Path to the application data directory where cache/configs are stored.
    pub app_data_dir: Option<String>,
    /// Optional JSON schema constraining the final structured tool output.
    pub response_schema: Option<String>,
    /// MCP server configurations to connect to external tool servers.
    pub mcp_servers: Vec<McpServerConfig>,
}

impl std::fmt::Debug for AgentConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentConfig")
            .field("binary_path", &self.binary_path)
            .field("gemini_config", &self.gemini_config)
            .field("capabilities", &self.capabilities)
            .field("system_instructions", &self.system_instructions)
            .field("save_dir", &self.save_dir)
            .field("workspaces", &self.workspaces)
            .field("skills_paths", &self.skills_paths)
            .field("policies", &self.policies)
            .field("hooks_count", &self.hooks.len())
            .field("triggers_count", &self.triggers.len())
            .field("tools_count", &self.tools.len())
            .field("conversation_id", &self.conversation_id)
            .field("app_data_dir", &self.app_data_dir)
            .field("response_schema", &self.response_schema)
            .field("mcp_servers", &self.mcp_servers)
            .finish()
    }
}

/// High-level orchestrator that manages an agentic execution session.
///
/// An `Agent` encapsulates binary discovery, WebSocket upgrades, tool wiring, safety policy enforcement,
/// and observer hook dispatch. It provides a simple `chat` API for sending prompts and retrieving responses.
///
/// # Examples
///
/// ```no_run
/// use antigravity_sdk_rust::agent::Agent;
///
/// #[tokio::main]
/// async fn main() -> Result<(), anyhow::Error> {
///     let agent = Agent::builder()
///         .allow_all()
///         .build();
///     let agent = agent.start().await?;
///
///     let response = agent.chat("What is 2+2?").await?;
///     println!("Agent: {}", response.text);
///
///     let _ = agent.stop().await;
///     Ok(())
/// }
/// ```
/// Marker trait for all valid agent lifecycles.
pub trait AgentLifecycle: Send + Sync + std::fmt::Debug {}

/// Represents an agent that has been configured but not yet started.
#[derive(Debug)]
pub struct Unstarted;
impl AgentLifecycle for Unstarted {}

/// Represents an active, running agent session.
pub struct Started {
    pub(crate) conversation: Arc<Conversation>,
    pub(crate) trigger_runner: Option<TriggerRunner>,
}

impl AgentLifecycle for Started {}

impl std::fmt::Debug for Started {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Started")
            .field("conversation", &self.conversation)
            .field("trigger_runner", &self.trigger_runner)
            .finish()
    }
}

pub struct Agent<S: AgentLifecycle = Unstarted> {
    config: AgentConfig,
    tool_runner: ToolRunner,
    hook_runner: HookRunner,
    state: S,
}

impl<S: AgentLifecycle> std::fmt::Debug for Agent<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Agent")
            .field("config", &self.config)
            .field("tool_runner", &self.tool_runner)
            .field("hook_runner", &self.hook_runner)
            .field("state", &self.state)
            .finish()
    }
}

impl Agent<Unstarted> {
    /// Creates a new `Agent` with the given configuration.
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            tool_runner: ToolRunner::new(),
            hook_runner: HookRunner::new(),
            state: Unstarted,
        }
    }

    /// Returns an `AgentBuilder` to configure and construct an `Agent`.
    pub fn builder() -> AgentBuilder<NoPolicies> {
        AgentBuilder::new()
    }

    /// Registers a custom lifecycle hook during configuration.
    pub fn register_hook(&mut self, hook: Arc<dyn DynHook>) {
        self.config.hooks.push(hook);
    }

    /// Registers a custom background trigger.
    pub fn register_trigger(&mut self, trigger: Arc<dyn DynTrigger>) -> Result<(), anyhow::Error> {
        self.config.triggers.push(trigger);
        Ok(())
    }

    /// Registers a custom tool during configuration.
    pub fn register_tool(&mut self, tool: Arc<dyn DynTool>) {
        self.config.tools.push(tool);
    }

    /// Spawns the subprocess communication harness, initializes safety policies, registers tools/hooks,
    /// establishes the WebSocket session, and starts any configured triggers.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `localharness` binary cannot be resolved.
    /// - Write tools are enabled but no safety policies are configured.
    /// - The WebSocket upgrade or subprocess connection fails.
    #[allow(clippy::too_many_lines)]
    pub fn start(self) -> BoxFuture<'static, Result<Agent<Started>, anyhow::Error>> {
        Box::pin(async move {
            // 1. Resolve binary path
            #[cfg(not(target_arch = "wasm32"))]
        let binary_path = self.config.binary_path.clone()
            .or_else(get_default_binary_path)
            .ok_or_else(|| anyhow!("Could not find default localharness binary. Please specify binary_path explicitly."))?;

            // 2. Setup hook runner and register pending hooks
            for hook in &self.config.hooks {
                self.hook_runner.register(hook.clone()).await;
            }

            // 3. Process capabilities and active tools
            let enabled_tools = self.config.capabilities.enabled_tools.clone();
            let disabled_tools = self.config.capabilities.disabled_tools.clone();
            if enabled_tools.is_some() && disabled_tools.is_some() {
                return Err(anyhow!(
                    "enabled_tools and disabled_tools are mutually exclusive"
                ));
            }

            let active_tools = enabled_tools.unwrap_or_else(|| {
                disabled_tools.map_or_else(
                    || {
                        vec![
                            BuiltinTools::CreateFile,
                            BuiltinTools::EditFile,
                            BuiltinTools::FindFile,
                            BuiltinTools::ListDir,
                            BuiltinTools::RunCommand,
                            BuiltinTools::SearchDir,
                            BuiltinTools::ViewFile,
                            BuiltinTools::StartSubagent,
                            BuiltinTools::GenerateImage,
                            BuiltinTools::Finish,
                        ]
                    },
                    |disabled| {
                        let all = vec![
                            BuiltinTools::CreateFile,
                            BuiltinTools::EditFile,
                            BuiltinTools::FindFile,
                            BuiltinTools::ListDir,
                            BuiltinTools::RunCommand,
                            BuiltinTools::SearchDir,
                            BuiltinTools::ViewFile,
                            BuiltinTools::StartSubagent,
                            BuiltinTools::GenerateImage,
                            BuiltinTools::Finish,
                        ];
                        all.into_iter().filter(|t| !disabled.contains(t)).collect()
                    },
                )
            });

            let read_only = BuiltinTools::read_only();
            let has_write_tools = active_tools.iter().any(|t| !read_only.contains(t));

            // 4. Set up policies
            let mut final_policies = self.config.policies.clone().unwrap_or_else(|| {
                // Default to confirm_run_command
                policy::confirm_run_command(None)
            });

            // Prepend workspace scoping policies ONLY if the caller has not explicitly opted
            // into allow_all(). When allow_all() is in the policy set, the intent is to
            // approve every tool call (typically managed by a ConfirmHook instead). In that
            // case, prepending workspace_only would silently override the user's approval
            // because Deny policies have higher bucket priority than wildcard Approve policies.
            //
            // We detect allow_all by looking for a wildcard Approve policy named "allow_all".
            // If found, skip the workspace gate entirely.
            let has_allow_all = final_policies.iter().any(|p| {
                p.tool == "*"
                    && p.decision == crate::policy::Decision::Approve
                    && p.name == "allow_all"
            });

            if !has_allow_all {
                let workspaces = self.config.workspaces.clone().unwrap_or_else(|| {
                    std::env::current_dir().map_or_else(
                        |_| Vec::new(),
                        |cwd| vec![cwd.to_string_lossy().into_owned()],
                    )
                });

                if !workspaces.is_empty() {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                    let app_data_dir = self
                        .config
                        .app_data_dir
                        .clone()
                        .unwrap_or_else(|| format!("{home}/.gemini/antigravity"));
                    let mut allowed_paths = workspaces;
                    allowed_paths.push(app_data_dir);
                    let mut ws_policies = policy::workspace_only(allowed_paths);
                    ws_policies.append(&mut final_policies);
                    final_policies = ws_policies;
                }
            }

            // Safety policy check: if write tools are enabled, policies cannot be empty
            if has_write_tools && final_policies.is_empty() {
                return Err(anyhow!(
                    "Write tools are enabled without a safety policy. Add policies=[policy.allow_all()] to approve all tool calls, or policies=[policy.deny_all(), policy.allow(\"tool_name\")] to selectively allow specific tools."
                ));
            }

            if !final_policies.is_empty() {
                let enforcer = Arc::new(PolicyEnforcer::new(final_policies, Vec::new()));
                self.hook_runner.register(enforcer).await;
            }

            // 5. Register configured tools
            for tool in &self.config.tools {
                self.tool_runner.register(tool.clone()).await;
            }

            // 6. Build and connect strategy
            #[cfg(target_arch = "wasm32")]
            {
                let mut cap = self.config.capabilities.clone();
                if let Some(ref schema) = self.config.response_schema {
                    cap.finish_tool_schema_json = Some(schema.clone());
                }

                let strategy = crate::wasm::WasmConnectionStrategy {
                    gemini_config: self.config.gemini_config.clone(),
                    capabilities_config: cap,
                    system_instructions: self.config.system_instructions.clone(),
                    save_dir: self.config.save_dir.clone(),
                    workspaces: self.config.workspaces.clone().unwrap_or_default(),
                    skills_paths: self.config.skills_paths.clone(),
                    tool_runner: Some(self.tool_runner.clone()),
                    hook_runner: Some(self.hook_runner.clone()),
                    conversation_id: self.config.conversation_id.clone().unwrap_or_default(),
                };

                let conn = strategy.connect().await?;
                let conversation = Arc::new(Conversation::new(
                    crate::connection::AnyConnection::Wasm(Arc::new(conn)),
                    None,
                ));

                // 7. Start triggers
                let mut trigger_runner = None;
                if !self.config.triggers.is_empty() {
                    let runner = TriggerRunner::new(self.config.triggers.clone());
                    runner.start(&conversation.connection());
                    trigger_runner = Some(runner);
                }

                Ok(Agent {
                    config: self.config,
                    tool_runner: self.tool_runner,
                    hook_runner: self.hook_runner,
                    state: Started {
                        conversation,
                        trigger_runner,
                    },
                })
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let mut cap = self.config.capabilities.clone();
                if let Some(ref schema) = self.config.response_schema {
                    cap.finish_tool_schema_json = Some(schema.clone());
                }

                let strategy = LocalConnectionStrategy::new(
                    binary_path,
                    self.config.gemini_config.clone(),
                    cap,
                    self.config.system_instructions.clone(),
                    self.config.save_dir.clone(),
                    self.config.workspaces.clone().unwrap_or_default(),
                    self.config.skills_paths.clone(),
                    Some(self.tool_runner.clone()),
                    Some(self.hook_runner.clone()),
                    self.config.conversation_id.clone().unwrap_or_default(),
                    self.config.mcp_servers.clone(),
                );

                let conn = strategy.connect().await?;
                let conversation = Arc::new(Conversation::new(
                    crate::connection::AnyConnection::Local(Arc::new(conn)),
                    None,
                ));

                // 7. Start triggers
                let trigger_runner = if self.config.triggers.is_empty() {
                    None
                } else {
                    let runner = TriggerRunner::new(self.config.triggers.clone());
                    runner.start(&conversation.connection());
                    Some(runner)
                };

                Ok(Agent {
                    config: self.config,
                    tool_runner: self.tool_runner,
                    hook_runner: self.hook_runner,
                    state: Started {
                        conversation,
                        trigger_runner,
                    },
                })
            }
        }) // end Box::pin
    }
}

impl Agent<Started> {
    /// Sends a prompt message to the active agent session and awaits the final completed response.
    ///
    /// # Errors
    ///
    /// Returns an error if the execution stream encounters a failure.
    pub async fn chat(&self, prompt: &str) -> Result<ChatResponse, anyhow::Error> {
        self.state.conversation.chat_to_completion(prompt).await
    }

    /// Returns the active [`Conversation`] session.
    pub fn conversation(&self) -> Arc<Conversation> {
        self.state.conversation.clone()
    }

    /// Returns the active conversation ID.
    pub fn conversation_id(&self) -> String {
        self.state.conversation.conversation_id().to_string()
    }

    /// Gracefully stops the agent connection and disconnects the underlying harness.
    ///
    /// # Errors
    ///
    /// Returns an error if closing the connection fails.
    pub async fn stop(&self) -> Result<(), anyhow::Error> {
        self.state.conversation.disconnect().await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct NoPolicies;
#[derive(Debug)]
pub struct HasPolicies;

pub struct AgentBuilder<P = NoPolicies> {
    config: AgentConfig,
    _policy_marker: std::marker::PhantomData<P>,
}

impl<P> std::fmt::Debug for AgentBuilder<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentBuilder")
            .field("config", &self.config)
            .finish()
    }
}

impl AgentBuilder<NoPolicies> {
    pub fn new() -> Self {
        Self {
            config: AgentConfig::default(),
            _policy_marker: std::marker::PhantomData,
        }
    }
}

impl Default for AgentBuilder<NoPolicies> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> AgentBuilder<P> {
    pub fn binary_path(mut self, path: impl Into<String>) -> Self {
        self.config.binary_path = Some(path.into());
        self
    }

    pub fn gemini_config(mut self, gemini_config: GeminiConfig) -> Self {
        self.config.gemini_config = gemini_config;
        self
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.config.gemini_config.api_key = Some(key.into());
        self
    }

    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.config.gemini_config.models.default.name = model.into();
        self
    }

    pub fn capabilities(mut self, capabilities: CapabilitiesConfig) -> Self {
        self.config.capabilities = capabilities;
        self
    }

    pub fn system_instructions(mut self, system_instructions: SystemInstructions) -> Self {
        self.config.system_instructions = Some(system_instructions);
        self
    }

    pub fn save_dir(mut self, save_dir: impl Into<String>) -> Self {
        self.config.save_dir = Some(save_dir.into());
        self
    }

    pub fn workspaces(mut self, workspaces: Vec<String>) -> Self {
        self.config.workspaces = Some(workspaces);
        self
    }

    pub fn skills_paths(mut self, skills_paths: Vec<String>) -> Self {
        self.config.skills_paths = skills_paths;
        self
    }

    pub fn hooks(mut self, hooks: Vec<Arc<dyn DynHook>>) -> Self {
        self.config.hooks = hooks;
        self
    }

    pub fn triggers(mut self, triggers: Vec<Arc<dyn DynTrigger>>) -> Self {
        self.config.triggers = triggers;
        self
    }

    pub fn tools(mut self, tools: Vec<Arc<dyn DynTool>>) -> Self {
        self.config.tools = tools;
        self
    }

    pub fn tool(mut self, tool: Arc<dyn DynTool>) -> Self {
        self.config.tools.push(tool);
        self
    }

    pub fn hook(mut self, hook: Arc<dyn DynHook>) -> Self {
        self.config.hooks.push(hook);
        self
    }

    pub fn trigger(mut self, trigger: Arc<dyn DynTrigger>) -> Self {
        self.config.triggers.push(trigger);
        self
    }

    pub fn policy(mut self, policy: Policy) -> AgentBuilder<HasPolicies> {
        let mut policies = self.config.policies.take().unwrap_or_default();
        policies.push(policy);
        self.config.policies = Some(policies);
        AgentBuilder {
            config: self.config,
            _policy_marker: std::marker::PhantomData,
        }
    }

    pub fn conversation_id(mut self, conversation_id: impl Into<String>) -> Self {
        self.config.conversation_id = Some(conversation_id.into());
        self
    }

    pub fn app_data_dir(mut self, app_data_dir: impl Into<String>) -> Self {
        self.config.app_data_dir = Some(app_data_dir.into());
        self
    }

    pub fn response_schema(mut self, response_schema: impl Into<String>) -> Self {
        self.config.response_schema = Some(response_schema.into());
        self
    }

    /// Adds a single MCP server configuration.
    pub fn mcp_server(mut self, server: McpServerConfig) -> Self {
        self.config.mcp_servers.push(server);
        self
    }

    /// Sets the full list of MCP server configurations.
    pub fn mcp_servers(mut self, servers: Vec<McpServerConfig>) -> Self {
        self.config.mcp_servers = servers;
        self
    }

    pub fn policies(self, policies: Vec<Policy>) -> AgentBuilder<HasPolicies> {
        let mut config = self.config;
        config.policies = Some(policies);
        AgentBuilder {
            config,
            _policy_marker: std::marker::PhantomData,
        }
    }

    pub fn allow_all(self) -> AgentBuilder<HasPolicies> {
        let mut config = self.config;
        config.policies = Some(vec![policy::allow_all()]);
        AgentBuilder {
            config,
            _policy_marker: std::marker::PhantomData,
        }
    }

    pub fn read_only(self) -> AgentBuilder<HasPolicies> {
        let mut config = self.config;
        let read_only_tools = BuiltinTools::read_only();
        let mut policies = vec![policy::deny_all()];
        for tool in read_only_tools {
            policies.push(policy::allow(tool.as_str()));
        }
        config.policies = Some(policies);
        AgentBuilder {
            config,
            _policy_marker: std::marker::PhantomData,
        }
    }

    /// Builder escape hatch to construct `Agent<Unstarted>` without compile-time check for policies.
    pub fn build_unchecked(self) -> Agent<Unstarted> {
        Agent::new(self.config)
    }
}

impl AgentBuilder<HasPolicies> {
    pub fn build(self) -> Agent<Unstarted> {
        Agent::new(self.config)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn get_default_binary_path() -> Option<String> {
    if let Ok(path) = std::env::var("ANTIGRAVITY_HARNESS_PATH") {
        return Some(path);
    }
    // Check ./bin/localharness relative to the current working directory
    // (this is where `just install` / `scripts/install_harness.sh` places the binary)
    if let Ok(cwd) = std::env::current_dir() {
        let local_bin = cwd.join("bin").join("localharness");
        if local_bin.exists() {
            return Some(local_bin.to_string_lossy().into_owned());
        }
    }
    // Check if it is in standard PATH (e.g. via `pip install google-antigravity`)
    if let Ok(paths) = std::env::var("PATH") {
        for path in std::env::split_paths(&paths) {
            let p = path.join("localharness");
            if p.exists() {
                return Some(p.to_string_lossy().into_owned());
            }
        }
    }
    // Check Python site-packages as a fallback since google-antigravity Python package installs it there
    if let Some(output) = std::process::Command::new("python3")
        .args([
            "-c",
            "import site; print('\\n'.join(site.getsitepackages()))",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let p = std::path::Path::new(line.trim())
                .join("google")
                .join("antigravity")
                .join("bin")
                .join("localharness");
            if p.exists() {
                return Some(p.to_string_lossy().into_owned());
            }
        }
    }
    None
}
