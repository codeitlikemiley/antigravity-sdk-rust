//! Safety policies and middleware for tool execution control.
//!
//! This module defines the safety policies that can be configured on an [`crate::agent::Agent`]
//! to restrict tool invocation permissions, establish sandbox/workspace boundaries,
//! or require explicit user confirmation.

use crate::hooks::Hook;
use crate::types::{HookResult, McpServerConfig, ToolCall};
use std::path::Path;
use std::sync::Arc;

/// Represents the safety enforcement action to take when a tool invocation is intercepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Allow execution of the tool automatically.
    Approve,
    /// Block execution and return an access denied error.
    Deny,
    /// Ask the user for confirmation before executing the tool.
    AskUser,
}

/// A policy configuration mapping a tool name to a safety [`Decision`].
///
/// Wildcard (`*`) rules are supported to act as a fallback, which are overridden
/// by specific tool rules according to a precedence bucketed hierarchy.
#[derive(Clone)]
pub struct Policy {
    /// The exact tool name, or `*` for a wildcard matching any tool.
    pub tool: String,
    /// The enforcement decision to apply when the tool matches.
    pub decision: Decision,
    /// An optional filter function that decides if the policy applies based on the tool parameters.
    pub when: Option<Arc<dyn Fn(&ToolCall) -> bool + Send + Sync>>,
    /// An optional callback deciding whether to prompt the user for confirmation.
    pub ask_user: Option<Arc<dyn Fn(&ToolCall) -> bool + Send + Sync>>,
    /// A descriptive name for the safety policy.
    pub name: String,
}

impl std::fmt::Debug for Policy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Policy")
            .field("tool", &self.tool)
            .field("decision", &self.decision)
            .field("when_is_some", &self.when.is_some())
            .field("ask_user_is_some", &self.ask_user.is_some())
            .field("name", &self.name)
            .finish()
    }
}

impl Policy {
    /// Creates a new policy rule.
    pub fn new(
        tool: String,
        decision: Decision,
        when: Option<Arc<dyn Fn(&ToolCall) -> bool + Send + Sync>>,
        ask_user: Option<Arc<dyn Fn(&ToolCall) -> bool + Send + Sync>>,
        name: String,
    ) -> Self {
        Self {
            tool,
            decision,
            when,
            ask_user,
            name,
        }
    }

    /// Adds a conditional filter function to the policy.
    pub fn when(mut self, when_fn: impl Fn(&ToolCall) -> bool + Send + Sync + 'static) -> Self {
        self.when = Some(Arc::new(when_fn));
        self
    }

    /// Customizes the policy's debug/tracing name.
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
}

/// Helper constructor to approve a specific tool invocation unconditionally.
pub fn allow(tool: &str) -> Policy {
    Policy::new(
        tool.to_string(),
        Decision::Approve,
        None,
        None,
        String::new(),
    )
}

/// Helper constructor to deny a specific tool invocation unconditionally.
pub fn deny(tool: &str) -> Policy {
    Policy::new(tool.to_string(), Decision::Deny, None, None, String::new())
}

/// Helper constructor to require user confirmation for a specific tool.
pub fn ask_user(tool: &str, handler: impl Fn(&ToolCall) -> bool + Send + Sync + 'static) -> Policy {
    Policy::new(
        tool.to_string(),
        Decision::AskUser,
        None,
        Some(Arc::new(handler)),
        String::new(),
    )
}

/// Helper constructor to approve all tool calls as a wildcard fallback.
pub fn allow_all() -> Policy {
    Policy::new(
        "*".to_string(),
        Decision::Approve,
        None,
        None,
        "allow_all".to_string(),
    )
}

/// Helper constructor to deny all tool calls as a wildcard fallback.
pub fn deny_all() -> Policy {
    Policy::new(
        "*".to_string(),
        Decision::Deny,
        None,
        None,
        "deny_all".to_string(),
    )
}

/// Creates a standard set of policies that requires user approval for commands or denies them.
pub fn confirm_run_command(
    handler: Option<Arc<dyn Fn(&ToolCall) -> bool + Send + Sync>>,
) -> Vec<Policy> {
    handler.map_or_else(
        || {
            vec![
                Policy::new(
                    "RUN_COMMAND".to_string(),
                    Decision::Deny,
                    None,
                    None,
                    "confirm_run_command".to_string(),
                ),
                allow_all(),
            ]
        },
        |h| {
            vec![
                Policy::new(
                    "RUN_COMMAND".to_string(),
                    Decision::AskUser,
                    None,
                    Some(h),
                    "confirm_run_command".to_string(),
                ),
                allow_all(),
            ]
        },
    )
}

/// Creates a set of policies restricting file system tools to specified workspace directory paths.
pub fn workspace_only(workspaces: Vec<String>) -> Vec<Policy> {
    let file_tools = vec![
        "CREATE_FILE",
        "EDIT_FILE",
        "VIEW_FILE",
        "LIST_DIR",
        "SEARCH_DIR",
    ];

    let is_outside_workspace = move |tc: &ToolCall| -> bool {
        let path_str = tc.canonical_path.as_deref().unwrap_or("");
        if path_str.is_empty() {
            return false;
        }
        let target_path = Path::new(path_str);
        if !target_path.is_absolute() {
            return true;
        }
        for ws in &workspaces {
            let ws_path = Path::new(ws);
            if ws_path.is_absolute() && target_path.starts_with(ws_path) {
                return false;
            }
        }
        true
    };

    let when_fn = Arc::new(is_outside_workspace);

    file_tools
        .into_iter()
        .map(|tool| {
            Policy::new(
                tool.to_string(),
                Decision::Deny,
                Some(when_fn.clone()),
                None,
                "workspace_only".to_string(),
            )
        })
        .collect()
}

/// Validates and compiles a list of [`Policy`] items into a [`PolicyEnforcer`].
///
/// # Arguments
///
/// * `policies` - Policy rules to enforce.
/// * `mcp_servers` - Registered MCP server configurations. Required when any policy
///   uses MCP-style targets (containing `/`). If MCP policies are present without
///   registered servers, returns an error to prevent silent security bypasses (fail-closed).
///
/// # Errors
///
/// Returns an error if any `AskUser` policy is missing a confirmation handler callback,
/// or if MCP policies are present but `mcp_servers` is empty/None.
pub fn enforce(
    policies: Vec<Policy>,
    mcp_servers: Option<&[McpServerConfig]>,
) -> Result<PolicyEnforcer, anyhow::Error> {
    // Validate MCP policies (fail-closed security guard).
    let has_mcp_policy = policies
        .iter()
        .any(|p| p.tool.contains('/') && p.tool != "*");
    let mcp_empty = mcp_servers.is_none_or(<[McpServerConfig]>::is_empty);
    if has_mcp_policy && mcp_empty {
        return Err(anyhow::anyhow!(
            "MCP policies (containing '/') were detected, but 'mcp_servers' was not \
             provided to enforce(). You must pass the registered MCP servers to \
             enable secure policy matching and prevent silent bypasses."
        ));
    }

    for p in &policies {
        if p.decision == Decision::AskUser && p.ask_user.is_none() {
            return Err(anyhow::anyhow!(
                "ASK_USER policy '{}' is missing an ask_user handler. \
                 Provide one via policy.ask_user(tool, handler).",
                if p.name.is_empty() { &p.tool } else { &p.name }
            ));
        }
    }

    let server_names: Vec<String> = mcp_servers
        .map(|servers| servers.iter().map(|s| s.name().to_string()).collect())
        .unwrap_or_default();

    Ok(PolicyEnforcer::new(policies, server_names))
}

/// Safety policy middleware enforcer implemented as a lifecycle [`Hook`].
pub struct PolicyEnforcer {
    buckets: [Vec<Policy>; 9],
    /// Known MCP server names, sorted descending by length for longest-match parsing.
    server_names: Vec<String>,
}

/// Priority bucket indices (lower = higher priority).
///
/// Specific tool matches (e.g. `run_command`, `server/tool`) have highest priority.
/// Prefix wildcards (e.g. `server/*`) have medium priority.
/// Global wildcards (`*`) have lowest priority.
const LEVEL_SPECIFIC_DENY: usize = 0;
const LEVEL_SPECIFIC_ASK: usize = 1;
const LEVEL_SPECIFIC_ALLOW: usize = 2;
const LEVEL_PREFIX_DENY: usize = 3;
const LEVEL_PREFIX_ASK: usize = 4;
const LEVEL_PREFIX_ALLOW: usize = 5;
const LEVEL_GLOBAL_DENY: usize = 6;
const LEVEL_GLOBAL_ASK: usize = 7;
const LEVEL_GLOBAL_ALLOW: usize = 8;

/// Returns true if the tool selector is a global wildcard ("*").
fn is_global_wildcard(tool: &str) -> bool {
    tool == "*"
}

/// Returns true if the tool selector is a prefix wildcard (e.g. "server/*").
fn is_prefix_wildcard(tool: &str) -> bool {
    tool.ends_with("/*")
}

/// Returns the priority bucket index for a policy.
fn bucket_index(p: &Policy) -> usize {
    if is_global_wildcard(&p.tool) {
        match p.decision {
            Decision::Deny => LEVEL_GLOBAL_DENY,
            Decision::AskUser => LEVEL_GLOBAL_ASK,
            Decision::Approve => LEVEL_GLOBAL_ALLOW,
        }
    } else if is_prefix_wildcard(&p.tool) {
        match p.decision {
            Decision::Deny => LEVEL_PREFIX_DENY,
            Decision::AskUser => LEVEL_PREFIX_ASK,
            Decision::Approve => LEVEL_PREFIX_ALLOW,
        }
    } else {
        match p.decision {
            Decision::Deny => LEVEL_SPECIFIC_DENY,
            Decision::AskUser => LEVEL_SPECIFIC_ASK,
            Decision::Approve => LEVEL_SPECIFIC_ALLOW,
        }
    }
}

impl PolicyEnforcer {
    /// Compiles policies into prioritized buckets for performance and precedence.
    pub fn new(policies: Vec<Policy>, server_names: Vec<String>) -> Self {
        let mut buckets: [Vec<Policy>; 9] = Default::default();
        for p in policies {
            buckets[bucket_index(&p)].push(p);
        }
        // Sort server names descending by length for longest-match-first parsing (security).
        let mut server_names = server_names;
        server_names.sort_by_key(|b| std::cmp::Reverse(b.len()));
        Self {
            buckets,
            server_names,
        }
    }

    /// Parses an MCP tool call name like `mcp_server_tool` into (`server`, `tool`)
    /// using the known server names list, trying longest names first for security.
    fn parse_mcp_tool(&self, tool_name: &str) -> Option<(String, String)> {
        let rest = tool_name.strip_prefix("mcp_")?;
        for server in &self.server_names {
            let prefix = format!("{}_", server);
            if let Some(tool) = rest.strip_prefix(&prefix).filter(|t| !t.is_empty()) {
                return Some((server.clone(), tool.to_string()));
            }
        }
        None
    }
}

impl std::fmt::Debug for PolicyEnforcer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolicyEnforcer")
            .field("buckets", &self.buckets)
            .field("server_names", &self.server_names)
            .finish()
    }
}

/// Matches a policy's tool selector against a parsed call target.
///
/// - Global wildcard ("*") matches everything.
/// - For MCP tools: prefix wildcard ("server/*") matches the server prefix;
///   exact match ("server/tool") matches exactly.
/// - For non-MCP tools: exact match only.
fn matches_target(policy_tool: &str, call_target: &str, is_mcp: bool) -> bool {
    if policy_tool == "*" {
        return true;
    }
    if is_mcp {
        if is_prefix_wildcard(policy_tool) {
            // "server/*" → extract "server" and compare with the call's server part
            let policy_server = &policy_tool[..policy_tool.len() - 2]; // strip "/*"
            if let Some((call_server, _)) = call_target.split_once('/') {
                return policy_server == call_server;
            }
            return false;
        }
        return policy_tool == call_target;
    }
    policy_tool == call_target
}

impl Hook for PolicyEnforcer {
    async fn pre_tool_call(&self, tool_call: &ToolCall) -> Result<HookResult, anyhow::Error> {
        // Parse MCP tool name once for all policy evaluations.
        let (call_target, is_mcp) = match self.parse_mcp_tool(&tool_call.name) {
            Some((server, tool)) => (format!("{server}/{tool}"), true),
            None => (tool_call.name.clone(), false),
        };

        for bucket in &self.buckets {
            for p in bucket {
                if !matches_target(&p.tool, &call_target, is_mcp) {
                    continue;
                }

                if let Some(ref when_fn) = p.when {
                    let when_fn = when_fn.clone();
                    let tc = tool_call.clone();
                    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                        when_fn(&tc)
                    }));
                    match res {
                        Ok(true) => {}
                        Ok(false) => continue,
                        Err(_) => {
                            let label = if p.name.is_empty() { &p.tool } else { &p.name };
                            return Ok(HookResult {
                                allow: false,
                                message: format!(
                                    "Policy evaluation failed for policy '{label}': predicate panicked."
                                ),
                            });
                        }
                    }
                }

                let label = if p.name.is_empty() { &p.tool } else { &p.name };
                match p.decision {
                    Decision::Deny => {
                        return Ok(HookResult {
                            allow: false,
                            message: format!("Denied by policy '{label}'."),
                        });
                    }
                    Decision::Approve => {
                        return Ok(HookResult {
                            allow: true,
                            message: String::new(),
                        });
                    }
                    Decision::AskUser => {
                        if let Some(ref handler) = p.ask_user {
                            let handler = handler.clone();
                            let tc = tool_call.clone();
                            let res =
                                std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                                    handler(&tc)
                                }));
                            match res {
                                Ok(true) => {
                                    return Ok(HookResult {
                                        allow: true,
                                        message: String::new(),
                                    });
                                }
                                Ok(false) => {
                                    return Ok(HookResult {
                                        allow: false,
                                        message: format!(
                                            "User denied tool '{}' (policy '{}').",
                                            tool_call.name, label
                                        ),
                                    });
                                }
                                Err(_) => {
                                    return Ok(HookResult {
                                        allow: false,
                                        message: format!(
                                            "Policy evaluation failed for policy '{label}': handler panicked."
                                        ),
                                    });
                                }
                            }
                        }
                        return Err(anyhow::anyhow!(
                            "ASK_USER policy '{}' is missing an ask_user handler.",
                            label
                        ));
                    }
                }
            }
        }

        Ok(HookResult {
            allow: true,
            message: String::new(),
        })
    }
}

/// Creates APPROVE policies for an MCP server's tools.
///
/// If `tools` is None, creates a prefix wildcard policy (`server/*`) for all tools.
/// If `tools` is Some, creates specific policies (`server/tool`) for each tool.
pub fn allow_mcp(server: &McpServerConfig, tools: Option<&[&str]>) -> Vec<Policy> {
    mcp_policies(server.name(), Decision::Approve, tools, None)
}

/// Creates DENY policies for an MCP server's tools.
pub fn deny_mcp(server: &McpServerConfig, tools: Option<&[&str]>) -> Vec<Policy> {
    mcp_policies(server.name(), Decision::Deny, tools, None)
}

/// Creates `ASK_USER` policies for an MCP server's tools.
pub fn ask_user_mcp(
    server: &McpServerConfig,
    tools: Option<&[&str]>,
    handler: impl Fn(&ToolCall) -> bool + Send + Sync + 'static + Clone,
) -> Vec<Policy> {
    mcp_policies(
        server.name(),
        Decision::AskUser,
        tools,
        Some(Arc::new(handler) as Arc<dyn Fn(&ToolCall) -> bool + Send + Sync>),
    )
}

/// Internal helper for generating MCP policies.
fn mcp_policies(
    server_name: &str,
    decision: Decision,
    tools: Option<&[&str]>,
    handler: Option<Arc<dyn Fn(&ToolCall) -> bool + Send + Sync>>,
) -> Vec<Policy> {
    match tools {
        None => {
            // Server-wide wildcard
            vec![Policy::new(
                format!("{server_name}/*"),
                decision,
                None,
                handler,
                format!("{}_{}_all", decision_label(decision), server_name),
            )]
        }
        Some(tools) => tools
            .iter()
            .map(|t| {
                Policy::new(
                    format!("{server_name}/{t}"),
                    decision,
                    None,
                    handler.clone(),
                    format!("{}_{}_{t}", decision_label(decision), server_name),
                )
            })
            .collect(),
    }
}

const fn decision_label(d: Decision) -> &'static str {
    match d {
        Decision::Approve => "allow",
        Decision::Deny => "deny",
        Decision::AskUser => "ask_user",
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unnecessary_map_or
    )]
    use super::*;
    use serde_json::json;

    fn make_tool_call(name: &str, args: serde_json::Value) -> ToolCall {
        let mut canonical_path = None;
        if let serde_json::Value::Object(ref map) = args {
            for path_key in &["path", "file_path", "TargetFile", "directory_path"] {
                if let Some(serde_json::Value::String(val)) = map.get(*path_key) {
                    canonical_path = Some(val.clone());
                    break;
                }
            }
        }
        ToolCall {
            id: "call_123".to_string(),
            name: name.to_string(),
            args,
            canonical_path,
        }
    }

    #[test]
    fn test_allow_approve_policy() {
        let p = allow("read_file").with_name("allow-read");
        assert_eq!(p.tool, "read_file");
        assert_eq!(p.decision, Decision::Approve);
        assert!(p.when.is_none());
        assert!(p.ask_user.is_none());
        assert_eq!(p.name, "allow-read");
    }

    #[test]
    fn test_deny_policy() {
        let p = deny("run_command").with_name("block-cmd");
        assert_eq!(p.tool, "run_command");
        assert_eq!(p.decision, Decision::Deny);
        assert_eq!(p.name, "block-cmd");
    }

    #[test]
    fn test_ask_user_policy() {
        let p = ask_user("run_command", |_| true).with_name("confirm-cmd");
        assert_eq!(p.decision, Decision::AskUser);
        assert!(p.ask_user.is_some());
        assert_eq!(p.name, "confirm-cmd");
    }

    #[test]
    fn test_deny_with_predicate() {
        let p = deny("run_command").when(|args| {
            args.args
                .get("CommandLine")
                .and_then(|v| v.as_str())
                .map_or(false, |s| s.contains("rm"))
        });
        assert!(p.when.is_some());
    }

    #[test]
    fn test_allow_all_creates_wildcard_approve() {
        let p = allow_all();
        assert_eq!(p.tool, "*");
        assert_eq!(p.decision, Decision::Approve);
        assert_eq!(p.name, "allow_all");
    }

    #[test]
    fn test_deny_all_creates_wildcard_deny() {
        let p = deny_all();
        assert_eq!(p.tool, "*");
        assert_eq!(p.decision, Decision::Deny);
        assert_eq!(p.name, "deny_all");
    }

    #[test]
    fn test_enforce_rejects_ask_user_without_handler() {
        let bad_policy = Policy::new(
            "run_command".to_string(),
            Decision::AskUser,
            None,
            None,
            "oops".to_string(),
        );
        let res = enforce(vec![bad_policy], None);
        assert!(res.is_err());
        let err_msg = res.err().unwrap().to_string();
        assert!(err_msg.contains("oops"));
        assert!(err_msg.contains("missing an ask_user handler"));
    }

    #[tokio::test]
    async fn test_specific_deny_overrides_wildcard_allow() {
        let enforcer = enforce(vec![allow_all(), deny("dangerous_tool")], None).unwrap();
        let res = enforcer
            .pre_tool_call(&make_tool_call("dangerous_tool", json!({})))
            .await
            .unwrap();
        assert!(!res.allow);
    }

    #[tokio::test]
    async fn test_specific_deny_overrides_specific_allow() {
        let enforcer = enforce(vec![allow("run_command"), deny("run_command")], None).unwrap();
        let res = enforcer
            .pre_tool_call(&make_tool_call("run_command", json!({})))
            .await
            .unwrap();
        assert!(!res.allow);
    }

    #[tokio::test]
    async fn test_specific_ask_overrides_wildcard_deny() {
        let enforcer = enforce(vec![deny_all(), ask_user("run_command", |_| true)], None).unwrap();
        let res = enforcer
            .pre_tool_call(&make_tool_call("run_command", json!({})))
            .await
            .unwrap();
        assert!(res.allow);
    }

    #[tokio::test]
    async fn test_specific_allow_overrides_wildcard_deny() {
        let enforcer = enforce(vec![deny_all(), allow("read_file")], None).unwrap();

        let res = enforcer
            .pre_tool_call(&make_tool_call("read_file", json!({})))
            .await
            .unwrap();
        assert!(res.allow);

        let res = enforcer
            .pre_tool_call(&make_tool_call("run_command", json!({})))
            .await
            .unwrap();
        assert!(!res.allow);
    }

    #[tokio::test]
    async fn test_wildcard_deny_blocks_unmatched_tools() {
        let enforcer = enforce(vec![deny_all()], None).unwrap();
        let res = enforcer
            .pre_tool_call(&make_tool_call("anything", json!({})))
            .await
            .unwrap();
        assert!(!res.allow);
    }

    #[tokio::test]
    async fn test_wildcard_ask_user() {
        let enforcer = enforce(vec![ask_user("*", |_| false)], None).unwrap();
        let res = enforcer
            .pre_tool_call(&make_tool_call("any_tool", json!({})))
            .await
            .unwrap();
        assert!(!res.allow);
    }

    #[tokio::test]
    async fn test_wildcard_allow() {
        let enforcer = enforce(vec![allow_all()], None).unwrap();
        let res = enforcer
            .pre_tool_call(&make_tool_call("any_tool", json!({})))
            .await
            .unwrap();
        assert!(res.allow);
    }

    #[tokio::test]
    async fn test_first_match_wins_within_deny_group() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

        let enforcer = enforce(
            vec![
                deny("run_command")
                    .when(|_| {
                        CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                        true
                    })
                    .with_name("first"),
                deny("run_command")
                    .when(|_| {
                        CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                        true
                    })
                    .with_name("second"),
            ],
            None,
        )
        .unwrap();

        let res = enforcer
            .pre_tool_call(&make_tool_call("run_command", json!({})))
            .await
            .unwrap();
        assert!(!res.allow);
        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_first_match_wins_within_allow_group() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

        let enforcer = enforce(
            vec![
                allow("read_file").when(|_| {
                    CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                    true
                }),
                allow("read_file").when(|_| {
                    CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                    true
                }),
            ],
            None,
        )
        .unwrap();

        let res = enforcer
            .pre_tool_call(&make_tool_call("read_file", json!({})))
            .await
            .unwrap();
        assert!(res.allow);
        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_skips_non_matching_predicate() {
        let enforcer = enforce(
            vec![
                deny("run_command").when(|_| false).with_name("skip-me"),
                deny("run_command").when(|_| true).with_name("catch-me"),
            ],
            None,
        )
        .unwrap();

        let res = enforcer
            .pre_tool_call(&make_tool_call("run_command", json!({})))
            .await
            .unwrap();
        assert!(!res.allow);
        assert!(res.message.contains("catch-me"));
    }

    #[tokio::test]
    async fn test_predicate_exception_matches_fail_closed() {
        let enforcer = enforce(
            vec![
                deny("run_command")
                    .when(|_| {
                        panic!("boom");
                    })
                    .with_name("broken"),
            ],
            None,
        )
        .unwrap();

        let res = enforcer
            .pre_tool_call(&make_tool_call("run_command", json!({})))
            .await
            .unwrap();
        assert!(!res.allow);
        assert!(res.message.contains("broken"));
        assert!(res.message.contains("panicked"));
    }

    #[tokio::test]
    async fn test_handler_exception_denies() {
        let enforcer = enforce(
            vec![
                ask_user("run_command", |_| {
                    panic!("handler broke");
                })
                .with_name("broken-ask"),
            ],
            None,
        )
        .unwrap();

        let res = enforcer
            .pre_tool_call(&make_tool_call("run_command", json!({})))
            .await
            .unwrap();
        assert!(!res.allow);
        assert!(res.message.contains("broken-ask"));
        assert!(res.message.contains("handler panicked"));
    }

    #[tokio::test]
    async fn test_no_matching_policy_allows() {
        let enforcer = enforce(vec![deny("other_tool")], None).unwrap();
        let res = enforcer
            .pre_tool_call(&make_tool_call("unrelated_tool", json!({})))
            .await
            .unwrap();
        assert!(res.allow);
    }

    #[tokio::test]
    async fn test_empty_policies_allows_all() {
        let enforcer = enforce(vec![], None).unwrap();
        let res = enforcer
            .pre_tool_call(&make_tool_call("any_tool", json!({})))
            .await
            .unwrap();
        assert!(res.allow);
    }

    #[tokio::test]
    async fn test_workspace_only() {
        let policies = workspace_only(vec!["/allowed/workspace".to_string()]);
        let enforcer = enforce(policies, None).unwrap();

        let tc1 = make_tool_call(
            "VIEW_FILE",
            json!({"path": "/allowed/workspace/subdir/file.rs"}),
        );
        let res1 = enforcer.pre_tool_call(&tc1).await.unwrap();
        assert!(res1.allow);

        let tc2 = make_tool_call("VIEW_FILE", json!({"path": "/forbidden/path/file.rs"}));
        let res2 = enforcer.pre_tool_call(&tc2).await.unwrap();
        assert!(!res2.allow);
    }

    #[tokio::test]
    async fn test_prefix_wildcard_matches_mcp_tool() {
        let server = McpServerConfig::Stdio {
            name: "math".to_string(),
            command: "echo".to_string(),
            args: vec![],
            enabled_tools: None,
            disabled_tools: None,
        };
        let mut policies = deny_mcp(&server, None); // "math/*" deny
        policies.push(allow_all());
        let enforcer = enforce(policies, Some(&[server])).unwrap();

        // MCP tool "mcp_math_add" should be denied by "math/*" prefix
        let res = enforcer
            .pre_tool_call(&make_tool_call("mcp_math_add", json!({})))
            .await
            .unwrap();
        assert!(!res.allow);

        // Non-MCP tool "read_file" should be allowed by wildcard
        let res = enforcer
            .pre_tool_call(&make_tool_call("read_file", json!({})))
            .await
            .unwrap();
        assert!(res.allow);
    }

    #[tokio::test]
    async fn test_specific_allow_beats_prefix_deny() {
        let server = McpServerConfig::Stdio {
            name: "calc".to_string(),
            command: "echo".to_string(),
            args: vec![],
            enabled_tools: None,
            disabled_tools: None,
        };
        let mut policies = deny_mcp(&server, None); // "calc/*" deny (level 3)
        policies.extend(allow_mcp(&server, Some(&["add"]))); // "calc/add" allow (level 2)
        let enforcer = enforce(policies, Some(&[server])).unwrap();

        // "add" should be allowed (specific > prefix)
        let res = enforcer
            .pre_tool_call(&make_tool_call("mcp_calc_add", json!({})))
            .await
            .unwrap();
        assert!(res.allow);

        // "subtract" should be denied (prefix deny applies)
        let res = enforcer
            .pre_tool_call(&make_tool_call("mcp_calc_subtract", json!({})))
            .await
            .unwrap();
        assert!(!res.allow);
    }

    #[tokio::test]
    async fn test_enforce_fails_closed_on_missing_servers() {
        let policy = Policy::new(
            "myserver/tool".to_string(),
            Decision::Deny,
            None,
            None,
            "mcp_deny".to_string(),
        );
        let result = enforce(vec![policy], None);
        assert!(result.is_err());
        let msg = result.err().unwrap().to_string();
        assert!(msg.contains("MCP policies"));
        assert!(msg.contains("mcp_servers"));
    }

    #[tokio::test]
    async fn test_longest_match_mcp_parsing() {
        let s1 = McpServerConfig::Stdio {
            name: "math".to_string(),
            command: "echo".to_string(),
            args: vec![],
            enabled_tools: None,
            disabled_tools: None,
        };
        let s2 = McpServerConfig::Stdio {
            name: "math_advanced".to_string(),
            command: "echo".to_string(),
            args: vec![],
            enabled_tools: None,
            disabled_tools: None,
        };

        let mut policies = deny_mcp(&s2, Some(&["calc"])); // "math_advanced/calc" deny
        policies.push(allow_all());
        let enforcer = enforce(policies, Some(&[s1, s2])).unwrap();

        // "mcp_math_advanced_calc" should be parsed as server="math_advanced", tool="calc"
        let res = enforcer
            .pre_tool_call(&make_tool_call("mcp_math_advanced_calc", json!({})))
            .await
            .unwrap();
        assert!(!res.allow);

        // "mcp_math_add" should parse as server="math", tool="add" → allowed by wildcard
        let res = enforcer
            .pre_tool_call(&make_tool_call("mcp_math_add", json!({})))
            .await
            .unwrap();
        assert!(res.allow);
    }
}
