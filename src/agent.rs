use crate::conversation::Conversation;
use crate::hooks::{DynHook, HookRunner};
#[cfg(not(target_arch = "wasm32"))]
use crate::local::LocalConnectionStrategy;
use crate::policy::{self, Policy};
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
    ///
    /// Defaults to a per-conversation directory under the system temp
    /// directory, so a harness with nowhere to write does not scatter state
    /// through the caller's working directory.
    pub save_dir: Option<String>,
    /// Extra environment variables for the harness process.
    ///
    /// Added on top of the environment the harness inherits from this process;
    /// sent on `InputConfig.env`. Native transport only — a browser has no
    /// subprocess to give an environment to.
    pub env: std::collections::HashMap<String, String>,
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
    /// How the conversation attaches to harness-side session state.
    ///
    /// Leave unset for a new conversation. Set `CreateOrResume` when supplying
    /// a `conversation_id`: without it a current harness attempts a resume and
    /// fails when the conversation does not exist.
    pub session_continuation_mode: Option<crate::types::SessionContinuationMode>,
}

impl std::fmt::Debug for AgentConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentConfig")
            .field("binary_path", &self.binary_path)
            .field("gemini_config", &self.gemini_config)
            .field("capabilities", &self.capabilities)
            .field("system_instructions", &self.system_instructions)
            .field("save_dir", &self.save_dir)
            .field("env_keys", &self.env.keys().collect::<Vec<_>>())
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
            .field("session_continuation_mode", &self.session_continuation_mode)
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

            // Single resolution point for workspace roots: the client-side policy
            // layer and the harness must be told about the same directories.
            // Upstream local_connection_config.py:54 (default `[os.getcwd()]`),
            // :144-150 (feeds the workspace policies) and :185 (passed to the
            // strategy) all read the one field.
            let workspaces = crate::workspace::resolve(self.config.workspaces.as_ref());

            // Upstream constrains the id (connection.py:100-107): the harness
            // requires at least 32 characters and rejects anything outside
            // [a-zA-Z0-9-], which otherwise surfaces as an opaque failure at
            // connect time.
            if let Some(ref id) = self.config.conversation_id {
                if id.len() < 32 {
                    return Err(anyhow!(
                        "conversation_id must be at least 32 characters, got {}",
                        id.len()
                    ));
                }
                if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                    return Err(anyhow!(
                        "conversation_id must contain only [a-zA-Z0-9-], got '{id}'"
                    ));
                }
            }

            // Upstream rejects RESUME without an id at config time
            // (connection.py:109-117); this crate has no config-validation hook,
            // so it is checked here.
            if self.config.session_continuation_mode
                == Some(crate::types::SessionContinuationMode::Resume)
                && self.config.conversation_id.is_none()
            {
                return Err(anyhow!(
                    "conversation_id must be specified when session_continuation_mode is Resume"
                ));
            }

            // 4. Set up policies
            let final_policies = compose_policies(
                self.config.policies.clone(),
                &workspaces,
                self.config.app_data_dir.as_deref(),
            )?;

            // Safety policy check: if write tools are enabled, policies cannot be empty
            if has_write_tools && final_policies.is_empty() {
                return Err(anyhow!(
                    "Write tools are enabled without a safety policy. Add policies=[policy.allow_all()] to approve all tool calls, or policies=[policy.deny_all(), policy.allow(\"tool_name\")] to selectively allow specific tools."
                ));
            }

            if !final_policies.is_empty() {
                // Route through `enforce()` rather than constructing the
                // enforcer directly: it runs the two fail-closed startup
                // validations (an AskUser policy with no handler, and MCP
                // policies without registered servers) and derives the MCP
                // server names, without which every `server/tool` policy is
                // silently inert. Upstream agent.py:105-110.
                let enforcer = Arc::new(policy::enforce(
                    final_policies,
                    Some(&self.config.mcp_servers),
                )?);
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
                    workspaces: workspaces.clone(),
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
                    workspaces.clone(),
                    self.config.skills_paths.clone(),
                    Some(self.tool_runner.clone()),
                    Some(self.hook_runner.clone()),
                    self.config.conversation_id.clone().unwrap_or_default(),
                    self.config.session_continuation_mode,
                    self.config.mcp_servers.clone(),
                );
                let strategy = LocalConnectionStrategy {
                    env: self.config.env.clone(),
                    ..strategy
                };

                let conn = strategy.connect().await?;
                // A resumed session's history comes back in the handshake reply.
                // Seeded before the first turn so `history()`, `turn_count()`
                // and `last_response()` describe the session that was resumed.
                let replayed = conn.initial_history().to_vec();
                let conversation = Arc::new(Conversation::new(
                    crate::connection::AnyConnection::Local(Arc::new(conn)),
                    None,
                ));
                conversation.seed_history(replayed).await;

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
        // Upstream rejects an empty prompt rather than sending it (agent.py).
        // An empty UserInput reaches the harness as a turn with no content, so
        // the model is asked to respond to nothing and the turn is wasted.
        if prompt.trim().is_empty() {
            return Err(anyhow!("prompt must not be empty"));
        }
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

    /// Adds environment variables for the harness process.
    ///
    /// Merged into whatever was set before, so it can be called more than once.
    pub fn env<K, V>(mut self, vars: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.config
            .env
            .extend(vars.into_iter().map(|(k, v)| (k.into(), v.into())));
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

    /// Sets how the conversation attaches to harness-side session state.
    ///
    /// Pair with [`conversation_id`](Self::conversation_id): a current harness
    /// refuses a caller-supplied id it has never seen unless this is
    /// `CreateOrResume`.
    pub const fn session_continuation_mode(
        mut self,
        mode: crate::types::SessionContinuationMode,
    ) -> Self {
        self.config.session_continuation_mode = Some(mode);
        self
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

    /// Sets the policy set from a mix of groups and individual policies.
    ///
    /// The group builders return `Vec<Policy>` and the individual ones return a
    /// `Policy`, so composing them previously meant assembling the vector by
    /// hand. Upstream flattens nested sequences for the same reason
    /// (`connection.py:138-159`).
    ///
    /// ```no_run
    /// use antigravity_sdk_rust::{agent::Agent, policy};
    ///
    /// let agent = Agent::builder()
    ///     .policy_groups([
    ///         policy::workspace_only(vec!["/srv/app".to_string()]),
    ///         vec![policy::deny("RUN_COMMAND"), policy::allow_all()],
    ///     ])
    ///     .build();
    /// ```
    pub fn policy_groups<I, G>(self, groups: I) -> AgentBuilder<HasPolicies>
    where
        I: IntoIterator<Item = G>,
        G: crate::policy::IntoPolicies,
    {
        let flattened: Vec<Policy> = groups
            .into_iter()
            .flat_map(crate::policy::IntoPolicies::into_policies)
            .collect();
        self.policies(flattened)
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

/// Builds the effective policy list for an agent.
///
/// Extracted from `Agent::start` so the composition can be tested without a
/// harness binary: `start()` resolves the binary first, which would otherwise
/// make every one of these rules unreachable in a unit test.
///
/// * `configured` — the caller's policies, or `None` for the default
///   `confirm_run_command(None)`.
/// * `workspaces` — already resolved and normalized (see [`crate::workspace`]).
/// * `app_data_dir` — the caller's override, or `None` for `~/.gemini/antigravity`.
///
/// # Errors
///
/// Returns an error when `app_data_dir` is relative, uses an unsupported
/// `~user` form, or is defaulted while the home directory is unknown.
fn compose_policies(
    configured: Option<Vec<Policy>>,
    workspaces: &[String],
    app_data_dir: Option<&str>,
) -> Result<Vec<Policy>, anyhow::Error> {
    let mut final_policies = configured.unwrap_or_else(|| policy::confirm_run_command(None));

    // Workspace scoping is applied unconditionally, matching upstream's
    // model-validator (local_connection_config.py:130-141 at 0.1.1, :112-133 at
    // 0.1.9). It used to be skipped whenever the policy set contained
    // `allow_all()` — which is what the README, the crate docs and every
    // example recommend — so the default posture was less sandboxed than
    // upstream's. Upstream documents `allow_all()` as the way to get autonomous
    // shell access *while* file tools stay scoped; the opt-out is
    // `workspaces(vec![])`, not a policy name.
    //
    // Drop any workspace_only policies already present first, so re-application
    // cannot stack duplicate DENY rules (0.1.9 :114-118). This runs
    // unconditionally, so `workspaces(vec![])` also strips a hand-passed
    // workspace_only group — upstream's semantics.
    final_policies.retain(|p| p.name != "workspace_only");

    if !workspaces.is_empty() {
        let app_data_dir = match app_data_dir {
            Some(raw) => {
                let expanded = crate::path_safety::expand_home(raw)?;
                if !std::path::Path::new(&expanded).is_absolute() {
                    // Message copied from upstream local_connection_config.py:112.
                    return Err(anyhow!(
                        "app_data_dir must be an absolute path, got '{raw}'"
                    ));
                }
                crate::path_safety::secure_normalize_path(&expanded)
                    .map_or(expanded, |p| p.to_string_lossy().into_owned())
            }
            // No /tmp fallback: granting a workspace root under a
            // world-writable directory when HOME is unset is worse than
            // refusing to start, and after the path-resolution fix a
            // pre-planted symlink there would be followed faithfully.
            None => crate::path_safety::default_app_data_dir()?
                .to_string_lossy()
                .into_owned(),
        };
        let mut allowed_paths = workspaces.to_vec();
        allowed_paths.push(app_data_dir);
        let mut ws_policies = policy::workspace_only(allowed_paths);
        ws_policies.append(&mut final_policies);
        final_policies = ws_policies;
    }

    Ok(final_policies)
}

#[cfg(not(target_arch = "wasm32"))]
fn get_default_binary_path() -> Option<String> {
    if let Ok(path) = std::env::var("ANTIGRAVITY_HARNESS_PATH") {
        return Some(path);
    }
    let binary_name = if cfg!(target_os = "windows") {
        "localharness.exe"
    } else {
        "localharness"
    };
    // Check ./bin/localharness relative to the current working directory
    // (this is where `just install` / `scripts/install_harness.sh` places the binary)
    if let Ok(cwd) = std::env::current_dir() {
        let local_bin = cwd.join("bin").join(binary_name);
        if local_bin.exists() {
            return Some(local_bin.to_string_lossy().into_owned());
        }
    }
    // Check if it is in standard PATH (e.g. via `pip install google-antigravity`)
    if let Ok(paths) = std::env::var("PATH") {
        for path in std::env::split_paths(&paths) {
            let p = path.join(binary_name);
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
                .join(binary_name);
            if p.exists() {
                return Some(p.to_string_lossy().into_owned());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use crate::policy::Decision;

    fn names(policies: &[Policy]) -> Vec<(&str, &str, Decision)> {
        policies
            .iter()
            .map(|p| (p.name.as_str(), p.tool.as_str(), p.decision))
            .collect()
    }

    fn workspace_denies(policies: &[Policy]) -> Vec<&str> {
        policies
            .iter()
            .filter(|p| p.name == "workspace_only")
            .map(|p| p.tool.as_str())
            .collect()
    }

    /// The regression this whole item exists for: `allow_all()` used to skip
    /// workspace scoping entirely, which is what the README, the crate docs and
    /// every example recommend — so the default posture was less sandboxed than
    /// upstream's. Upstream applies scoping unconditionally and documents
    /// `allow_all()` as autonomous shell access *with* file tools still scoped.
    #[test]
    fn allow_all_still_gets_workspace_scoping() {
        let composed = compose_policies(
            Some(vec![policy::allow_all()]),
            &["/ws".to_string()],
            Some("/app-data"),
        )
        .unwrap();

        assert_eq!(
            workspace_denies(&composed),
            crate::types::BuiltinTools::path_scoped_tools()
                .iter()
                .map(BuiltinTools::as_str)
                .collect::<Vec<_>>(),
        );
        // The caller's own policies survive, and follow the DENY prefix so the
        // bucket ordering still resolves in their favour for unscoped tools.
        assert_eq!(composed.last().map(|p| p.name.as_str()), Some("allow_all"));
    }

    /// `workspaces(vec![])` is upstream's documented opt-out — the only one.
    #[test]
    fn empty_workspace_list_is_the_opt_out() {
        let composed =
            compose_policies(Some(vec![policy::allow_all()]), &[], Some("/app-data")).unwrap();
        assert!(workspace_denies(&composed).is_empty());
        assert_eq!(
            names(&composed),
            vec![("allow_all", "*", Decision::Approve)]
        );
    }

    /// Re-application must not stack duplicate DENY rules (upstream 0.1.9
    /// local_connection_config.py:114-118).
    #[test]
    fn workspace_policies_are_not_stacked() {
        let mut configured = policy::workspace_only(vec!["/old".to_string()]);
        configured.push(policy::allow_all());

        let composed =
            compose_policies(Some(configured), &["/ws".to_string()], Some("/app-data")).unwrap();

        assert_eq!(
            workspace_denies(&composed).len(),
            crate::types::BuiltinTools::path_scoped_tools().len(),
            "a pre-existing workspace_only group must be replaced, not appended to"
        );
    }

    /// Upstream `local_connection_config.py:109-113` rejects a relative
    /// `app_data_dir` with this exact message.
    #[test]
    fn relative_app_data_dir_is_rejected() {
        let err = compose_policies(None, &["/ws".to_string()], Some("relative/dir"))
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(err.contains("must be an absolute path"), "{err}");
    }

    #[test]
    fn tilde_user_app_data_dir_is_rejected() {
        assert!(compose_policies(None, &["/ws".to_string()], Some("~someone/x")).is_err());
    }

    /// The `app_data_dir` joins the workspace allow-list, so the agent can reach
    /// its own state directory (upstream :122-127).
    #[test]
    fn app_data_dir_joins_the_allowlist() {
        let composed = compose_policies(None, &["/ws".to_string()], Some("/app-data")).unwrap();
        let scoped = composed
            .iter()
            .find(|p| p.name == "workspace_only" && p.tool == "VIEW_FILE")
            .and_then(|p| p.when.clone())
            .unwrap();

        let inside_app_data = crate::types::ToolCall {
            id: "1".to_string(),
            name: "VIEW_FILE".to_string(),
            args: serde_json::json!({}),
            canonical_path: Some("/app-data/state.json".to_string()),
        };
        // `when` is "is outside the workspace", so false means allowed.
        assert!(!scoped(&inside_app_data));

        let elsewhere = crate::types::ToolCall {
            canonical_path: Some("/etc/passwd".to_string()),
            ..inside_app_data
        };
        assert!(scoped(&elsewhere));
    }

    /// With no policies configured at all, the default is still scoped.
    #[test]
    fn default_policies_are_scoped_too() {
        let composed = compose_policies(None, &["/ws".to_string()], Some("/app-data")).unwrap();
        assert!(!workspace_denies(&composed).is_empty());
        assert!(composed.iter().any(|p| p.name == "confirm_run_command"));
    }
}
