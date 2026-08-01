# Upstream Parity Report — `google-antigravity` (Python) → `antigravity-sdk-rust`

**Audit date:** 2026-08-01
**Upstream latest:** `google-antigravity` 0.1.9 (PyPI, released 2026-07-29)
**Rust SDK baseline:** upstream **0.1.1 / 0.1.2** (`scripts/install_harness.sh` pins `VERSION="0.1.1"`)

Upstream has shipped **8 releases** since our baseline, roughly weekly:

| Version | Released |
|---|---|
| 0.1.1 | 2026-05-28 *(our baseline)* |
| 0.1.2 | 2026-06-04 |
| 0.1.3 | 2026-06-11 |
| 0.1.4 | 2026-06-18 |
| 0.1.5 | 2026-06-25 |
| 0.1.6 | 2026-07-09 |
| 0.1.7 | 2026-07-16 |
| 0.1.8 | 2026-07-23 |
| 0.1.9 | 2026-07-29 |

How this was determined: every platform wheel was downloaded from PyPI, the
Python sources diffed release-to-release, and the embedded
`localharness_pb2.py` descriptors decoded and compared field-by-field against
`proto/localharness.proto`. Result: our proto matches 0.1.1 exactly (0 types
missing); against 0.1.9 it is missing **42 message/enum types**.

---

## 1. Breaking wire change — highest priority

`HarnessConfig` dropped `gemini_config` (field 2) and `gemma_config` (field 3)
in **0.1.4**, replaced by a repeated `models` (field 15) of the new
`ModelConfig`:

```proto
// 0.1.9
message HarnessConfig {
  repeated ModelConfig models = 15;   // was: GeminiConfig gemini_config = 2;
  ...
}
message ModelConfig {
  string name = 1;
  repeated ModelType types = 2;                 // TEXT | IMAGE
  oneof endpoint {
    GeminiAPIEndpoint gemini_api_endpoint = 3;
    VertexEndpoint    vertex_endpoint     = 4;
    GemmaEndpoint     gemma_endpoint      = 6;
    CustomEndpoint    custom_endpoint     = 7;
  }
}
```

`src/local.rs:661` still emits `harness_config::ModelConfig::GeminiConfig(...)`
on field 2. A harness binary ≥ 0.1.4 **silently ignores it** — no model,
no API key, no Vertex settings reach the backend.

This is live today: `README.md` tells users to `pip install google-antigravity`,
which now installs the **0.1.9** harness, while `scripts/install_harness.sh`
pins 0.1.1. The two install paths produce incompatible binaries.

**Action:** replace `GeminiConfig`/`GemmaConfig`/`ModelEntry` in `src/types.rs`
with `ModelTarget` / `ModelEndpoint` (`GeminiAPIEndpoint`, `VertexEndpoint`,
`GemmaEndpoint`, `CustomEndpoint`) plus `GeminiModelOptions { thinking_level }`,
and emit field 15. Either bump the pin in `install_harness.sh` to 0.1.9 or keep
0.1.1 until the migration lands — but the two must agree.

---

## 2. Proto delta (0.1.1 → 0.1.9)

### Fields added to existing messages

| Message | Added |
|---|---|
| `InputConfig` | `client_info` (4), `env` (5) |
| `HarnessConfig` | `mcp_servers` (14), `models` (15), `enabled_hooks` (16), `custom_subagents` (17), `tool_output_truncation` (18), `session_continuation_mode` (19), `retry_config` (20) |
| `HarnessSideTools` | `search_web` (12), `read_url_content` (14), `tool_search_config` (15) |
| `Tool` | `defer_loading` (5) |
| `CustomSystemInstructions.Part` | `template` (2) |
| `OutputEvent` | `initialize_conversation_response` (13), `call_hook_request` (14), `session_end_response` (15) |
| `InputEvent` | `call_hook_response` (8), `session_end_request` (9) |
| `StepUpdate` | `mcp_tool` (33), `search_web` (34), `read_url_content` (35), `custom_tool` (36) |
| `UserInput.Part` | `slash_command` (3) |
| `ActionGenerateImage` | `aspect_ratio` (4) |
| `TrajectoryStateUpdate` | `error` (4) |
| `ToolResponse` | `error_message` (5) |

### Removed

`GeminiConfig`, `GemmaConfig`, `GenerateImageToolConfig.model_name` (2), and the
hand-rolled `Struct`/`Field`/`Value`/`ListValue` messages (our proto still
carries all of these).

### New messages / enums

Models: `ModelConfig`, `ModelType`, `GeminiModelOptions`, `GeminiAPIEndpoint`,
`VertexEndpoint`, `GemmaEndpoint`, `CustomEndpoint`, `RetryConfig`,
`ModelAPIRetryConfig`, `ModelOutputRetryConfig`.

MCP: `McpServerConfig` (+ `AuthProviderType`), `McpStdioTransport`,
`McpHttpTransport`, `ActionMcpTool`.

Harness-side hooks: `LifecycleHook` enum, `CallHookRequest`/`CallHookResponse`,
`PreToolArgs`/`PostToolArgs`/`OnToolErrorArgs`/`PreTurnArgs`/`PostTurnArgs`,
`PreToolResult` (ALLOW/DENY + `modified_arguments_json`), `PreTurnResult`,
`OnToolErrorResult`, `EmptyResult`.

Other: `CustomAgent` (custom subagents), `ToolOutputTruncation`
(+ `TruncateStrategy`/`ErrorStrategy`), `SearchWebToolConfig`,
`ReadUrlContentToolConfig`, `ToolSearchConfig`, `ActionSearchWeb`,
`ActionReadUrlContent`, `ActionCustomTool`, `InitializeConversationResponse`,
`UserInput.SlashCommand`, `CustomSystemInstructions.SystemInstructionTemplate`,
`HarnessConfig.SessionContinuationMode`.

---

## 3. Release-by-release changes

### 0.1.2
- **Removed `McpSseServer`** — SSE transport dropped; `McpServerConfig` is now
  stdio | streamable-http only. `src/types.rs:265` still has the `Sse` variant
  and `sse_read_timeout`.
- Added `AntigravityCancelledError` and `Conversation.cancel()`.
- Added `BuiltinSlashCommandName` (`plan`) and the `SlashCommand` content type.
- `send()` gained `**kwargs`; connection validation split into
  `_validate_connection()`; binary resolution split into an "external" path and
  SDK-version reporting via `ClientInfo`.
- Interactive CLI gained a `Spinner`.

### 0.1.3
- **MCP moved into the harness.** `google/antigravity/mcp/bridge.py` was deleted
  and the SDK now forwards `mcp_servers` in `HarnessConfig`, with tools exposed
  as `mcp_<server>_<tool>`. We already carry `McpServerConfig` in `src/types.rs`,
  so this is largely done — but the SSE variant must go.
- `ToolResult.timeout_seconds`; `TERMINAL_ERROR` removed from a status enum.

### 0.1.4 — the big one
- Model config rewrite (see §1); `models.py` added with `DEFAULT_MODEL`,
  `ModelTarget`, `ModelEndpoint`.
- `FileChange`/`FileChangeKind` moved from `types` to `triggers`.
- `ThinkingLevel`/`GenerationConfig` folded into `GeminiModelOptions.thinking_level`.
- `HookContext.get()/set()` renamed to `get_state()/set_state()` *(already done
  on our side in `src/tool_context.rs`)*.
- Public surface tightened: `Connection.send_tool_results`, `Conversation.delete`,
  `signal_idle`, `last_turn_usage`, `Agent.register_hook/register_trigger`,
  `ToolContext.is_idle/send` all became private or were removed.
- `BuiltinTools.SEARCH_WEB` added; `McpStdioServer.env` added.
- `run_interactive_loop` gained an `agent_class` parameter.

### 0.1.5
- **Harness-side hooks**: `hook_router.py`, the `LifecycleHook` enum, and
  `HarnessConfig.enabled_hooks` — the harness now calls back into the SDK
  mid-turn (`CallHookRequest`/`CallHookResponse`) and can DENY or rewrite tool
  arguments. We have no equivalent.
- `_PreStepHook`/`_PostStepHook` + `dispatch_pre_step`/`dispatch_post_step`.
- **Custom subagents**: `SubagentConfig`, `SubagentCapabilities`,
  `HarnessConfig.custom_subagents`.
- `TriggerConnection` protocol with `send_trigger_notification` *(we have this)*.
- `AgentConfig.policies` flattening validator; `_initial_history`.
- `utils/otel.py` — OpenTelemetry tracing hooks (optional extra).
- `types.from_bytes()`; `THINKING` step type.

### 0.1.6
- `local_connection.py` split into `event_processor.py` + `hook_router.py`;
  `LocalAgentConfig` refactored into `BaseLocalAgentConfig` + subclasses.
- **New backends**: `LiteRTConnectionStrategy` (+ `litert_server.py`,
  `LiteRTBackend`) and `LocalOpenAIAgentConfig` for OpenAI-compatible endpoints.
- `BuiltinTools.READ_URL_CONTENT` + `ReadUrlContentResult`.
- `app_data_dir` validation and workspace-policy auto-application moved into the
  config model.

### 0.1.7
- `utils/state.py` — a thread-safe hierarchical `StateStore`; `HookContext` and
  `ToolContext` now inherit from it (`ToolContext` is also a context manager).
- **Session continuation**: `SessionContinuationMode`
  (`RESUME` / `CREATE_OR_RESUME` / `CREATE_ONLY`) + `conversation_id` validation.
- WebSocket connect retries (max 5) and a 3-minute process wait timeout.

### 0.1.8
- Proto package moved to `google/antigravity/proto/`.
- Prompt sanitization — control characters stripped before send.
- `UsageMetadata.__add__` (replaces the private `_add_usage` helper).
- Tool argument coercion in `tool_runner._coerce_args`.
- `DEFAULT_MODEL` bumped **`gemini-3.5-flash` → `gemini-3.6-flash`**
  (`src/types.rs:12` is still on 3.5).

### 0.1.9
- `RetryConfig` / `ModelAPIRetryConfig` (max retries, initial backoff,
  exponential multiplier, jitter) / `ModelOutputRetryConfig`, plus
  `RetryConfig.benchmark()`; wired to `HarnessConfig.retry_config`.
- `DebugConfig` on `AgentConfig` — `enable_server_side_tracing` + logging level.
- `ToolExecutionError`.
- `local_openai_connection.py` split out; LiteRT warmup timeout 60s → 120s.
- `content_pb2.py` added alongside `localharness_pb2.py`.

---

## 4. Suggested order of work for the Rust SDK

1. **Regenerate `proto/localharness.proto` from the 0.1.9 descriptor** and fix
   `src/local.rs` to emit `models` (field 15). Bump `install_harness.sh` to
   0.1.9 in the same change. *Everything else depends on this.*
2. Model types in `src/types.rs`: `ModelTarget`/`ModelEndpoint` family,
   `GeminiModelOptions.thinking_level`, drop `GeminiConfig`/`GemmaConfig`/
   `ModelEntry`/`GenerationConfig`, bump `DEFAULT_MODEL` to `gemini-3.6-flash`,
   drop `GenerateImageToolConfig.model_name`.
3. Remove the MCP `Sse` variant; add `env` to the stdio variant.
4. `RetryConfig` + `DebugConfig` on the agent config (small, self-contained).
5. `SessionContinuationMode` + `conversation_id` validation; WebSocket connect
   retry loop; prompt control-character sanitization.
6. Harness-side hooks (`enabled_hooks`, `CallHookRequest`/`Response` routing) —
   the largest behavioural gap after the model rewrite.
7. Custom subagents (`CustomAgent`), `ToolOutputTruncation`, `defer_loading` +
   `ToolSearchConfig`.
8. `search_web` / `read_url_content` builtin tools and their step actions;
   slash commands; `aspect_ratio` on image generation.
9. Nice-to-have: hierarchical state store semantics, `UsageMetadata` addition,
   OTel tracing hooks behind a feature flag.
10. Out of scope for parity unless we want them: LiteRT and OpenAI-compatible
    local backends.
