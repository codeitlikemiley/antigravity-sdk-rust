# Upstream Parity: antigravity-sdk-rust vs `google-antigravity` 0.1.9

**Status:** audit complete, migration not started.
**Parity target:** upstream `0.1.9`.
**Port baseline:** upstream `0.1.1` (+ a `ClientInfo` back-port from 0.1.2) — what this crate was written against.
**Audit date:** 2026-08-01. Nine subsystem audits, each adversarially verified against the extracted wheels and the decoded harness descriptors, then the load-bearing claims reproduced against the shipped 0.1.9 harness binary (§2). 258 findings survived verification: 31 breaking, 62 high, 107 medium, 58 low.

This document replaces the first-pass parity note. Several of that note's claims are false and are corrected below (see [Corrections to the first-pass audit](#corrections-to-the-first-pass-audit)).

**Companion document:** `docs/fix-plan-current-defects.md` plans the subset of these findings that is wrong *right now*, against the 0.1.1 harness this crate pins — implementable without touching the wire format. It also rejects five findings from this audit with evidence (H8, H7-narrow, and the wording of S7, N6 and S15); those rejections are folded back in here.

---

## 1. What changed upstream, and what it costs us

Between 0.1.1 and 0.1.9 the harness wire protocol changed in ways that are not backward compatible, and the SDK's own architecture was reorganised twice (hooks moved into the harness in 0.1.5–0.1.6; `local_connection.py` was split into `event_processor.py` + `hook_router.py`).

Two facts turn every schema gap into a hard failure rather than graceful degradation:

1. **The WebSocket carries protojson, not binary protobuf.** `json_format.MessageToJson` / `Parse` on the Python side, `serde_json::from_str::<OutputEvent>` on ours (`src/local.rs:739`, `src/wasm.rs:383`). Field **names** and enum **value names** are matched, so a rename is as fatal as a renumber.
2. **`build.rs` never calls `pbjson_build::Builder::ignore_unknown_fields()`.** Verified: `build.rs:14-18` is exactly `Builder::new().register_descriptors(..).build(&[".antigravity.localharness"])`. pbjson-build 0.6.2 defaults that flag to `false` (`src/lib.rs:106`) and generates `_ => Err(serde::de::Error::unknown_field(value, FIELDS))` (`generator/message.rs:750`); the enum generator emits `unknown_variant` **unconditionally**, with no flag guard (`generator/enumeration.rs:154`).

Consequence: an `OutputEvent` carrying any post-0.1.1 field or enum name is not partially parsed — it is **discarded whole**, and the only handler is `tracing::error!(...)` followed by `continue` (`src/local.rs:1229-1235`).

### The practical symptom

Point this SDK at a `google-antigravity==0.1.9` harness and:

| Step | What happens |
|---|---|
| Init | We send `HarnessConfig.geminiConfig` (field 2). That field does **not exist** in the 0.1.9 binary. The harness rejects the frame and closes the connection — reproduced in §2, not inferred. |
| Handshake | The harness's first frame is `initializeConversationResponse` (`protobuf:"bytes,13,...,oneof"` is in the binary). We never `recv()` it, and when the reader loop sees it, it fails to deserialize. |
| Any turn | The harness signals completion with `{"trajectoryStateUpdate":{"state":"STATE_FULLY_IDLE"}}`. Our enum only knows `STATE_IDLE = 2` (`proto/localharness.proto:376`). The whole event is rejected, `is_idle` is never set, the IDLE sentinel is never pushed, and **`chat()` never returns** — on both the native and WASM paths. |
| Any custom tool | The harness sends a `stepUpdate.customTool` alongside the `toolCall`. Field 36 does not exist for us, so that entire `StepUpdate` is dropped. |
| Any MCP | We accept `mcp_servers`, validate MCP policies against them, and never put them on the wire. The user gets a confidently wrong security posture. |

So: **the SDK cannot connect to a current harness at all**, and the only reason CI is green is that `scripts/install_harness.sh:7` pins `VERSION="0.1.1"` while `README.md:28-38` tells users to `pip install google-antigravity` (now 0.1.9). Those two instructions install mutually incompatible binaries and nothing in the repo says so.

Beyond the wire, four whole capabilities documented as working are inert: harness-side lifecycle hooks, MCP, context-aware tools (`ToolContext` is never constructed), and multimodal / slash-command prompts. Two security controls are weaker than upstream: `workspace_only` accepts `..` traversal, and `pre_tool_call` fails **open** when a hook errors.

---

## 2. Reproduction

The two claims the rest of this document rests on were reproduced directly rather than reasoned about. Both are re-runnable.

### 2.1 Outbound — the 0.1.9 harness rejects what we send

`scripts/probe_harness.py` speaks the harness protocol itself (length-prefixed binary `InputConfig` on stdin → `OutputConfig` on stdout → protojson over the WebSocket), so the `InitializeConversationEvent` can be varied without going through either SDK. Against the `localharness` binary shipped in the `google-antigravity==0.1.9` wheel:

```
[0.1.1 geminiConfig]                                       REJECTED
  harness stderr: Failed to read init event: proto: unknown field "geminiConfig"
[0.1.9 models[], empty cascadeId]                          ACCEPTED
  -> {"initializeConversationResponse":{"cascadeId":"d6bc26016ecdc49a3ee8475a30f069ea"}, ...}
[0.1.9 models[], caller id, no sessionContinuationMode]    REJECTED
  harness stderr: Failed to create agent: conversation "…" not found in "…" (cannot resume)
[0.1.9 models[], caller id, CREATE_OR_RESUME]              ACCEPTED
  -> {"initializeConversationResponse":{"cascadeId":"…"}, ...}
```

Three things follow, and they change the severity of two findings:

1. **W3 is a hard failure, not a silent one.** Go's `protojson` runs with `DiscardUnknown=false`; the harness logs the unknown field and closes the socket. There is no degraded mode in which the model config is merely ignored.
2. **The harness's first frame is `initializeConversationResponse`** — confirming W4/W12 from the other side. Nothing else can happen until it is read.
3. **W15 (`session_continuation_mode`) is breaking for anyone who sets `conversation_id`.** With the field absent (`SESSION_CONTINUATION_MODE_UNSPECIFIED`) and a caller-supplied id, the harness attempts a resume and *fails* when the conversation does not exist. Upstream's `CREATE_OR_RESUME` is what makes a caller-chosen id work. `Agent::builder().conversation_id(...)` and `examples/persistence.rs` land exactly on the failing case; the default path (empty `cascade_id`) is unaffected.

### 2.2 Inbound — we reject what the 0.1.9 harness sends

Feeding real 0.1.9 frames through `serde_json::from_str::<OutputEvent>` (the exact call at `src/local.rs:739`):

```
INIT_RESPONSE    => Err("unknown field `initializeConversationResponse`, expected one of
                        `seqNum`, `timestampMicros`, `usageMetadata`, `stepUpdate`,
                        `trajectoryStateUpdate`, `toolCall`")
FULLY_IDLE       => Err("unknown variant `STATE_FULLY_IDLE`, expected one of
                        `STATE_UNSPECIFIED`, `STATE_RUNNING`, `STATE_IDLE`")
RUNNING          => Ok
CUSTOM_TOOL_STEP => Err("unknown field `customTool`, …")
```

`FULLY_IDLE` is the load-bearing one: it is how a turn ends. The event is discarded whole, `is_idle` is never set, and `chat()` never returns — so even after the outbound config is fixed, the first turn hangs until W1 and W2 are fixed too.

Note that `ignore_unknown_fields` would **not** have saved the idle case: pbjson emits `unknown_variant` for enums regardless of that flag. The rename must be fixed in the schema.

---

## 3. Findings, severity ordered

Effort: XS ≤ 1h · S ≤ half a day · M ≤ 2 days · L ≤ 1 week · XL > 1 week.

### Breaking — the SDK does not work, or silently drops user configuration

| ID | Area | What | Where in Rust | Effort |
|---|---|---|---|---|
| W1 | wire | Strict pbjson deserializer: one unknown field/variant discards the entire `OutputEvent` | `build.rs:14-18`; `src/local.rs:739,1229-1235`; `src/wasm.rs:383,869` | XS |
| W2 | wire | `TrajectoryStateUpdate.State` value 2 renamed `STATE_IDLE`→`STATE_FULLY_IDLE`; `STATE_CANCELLED=3` missing. Turn never terminates | `proto/localharness.proto:376`; `src/local.rs:1044-1054`; `src/wasm.rs:673-704` | S |
| W3 | wire | `HarnessConfig.gemini_config=2`/`gemma_config=3` removed in 0.1.4, replaced by `repeated ModelConfig models = 15`. We still emit field 2 | `proto:28-31`; `src/local.rs:661-666`; `src/wasm.rs:297-302` | L |
| W4 | wire | `OutputEvent` oneof arms 13/14/15 absent; arm 13 is the mandatory first frame | `proto:168-177` | M |
| W5 | wire | `StepUpdate` action arms 33–36 (`mcp_tool`, `search_web`, `read_url_content`, `custom_tool`) absent | `proto:214-228`; `src/local.rs:786-805,1294-1411` | L |
| W6 | wire | `HarnessConfig.mcp_servers=14` absent; the field on the strategy is stored and never read | `proto:26-41`; `src/local.rs:345,375,661-675`; `src/agent.rs:562-571` | M |
| S1 | policy | `workspace_only()` accepts `../` and symlink escapes — Rust **allows** what upstream **denies** | `src/policy.rs:175-191` | M |
| S2 | policy | `pre_tool_call` fails **open** on hook `Err` at four sites | `src/local.rs:1005-1013,1103`; `src/wasm.rs:647-651,742`; `src/hooks.rs:251-266` | XS |
| H1 | hooks | `dispatch_pre_turn` / `post_turn` / `session_end` / `on_compaction` have zero production callers | `src/hooks.rs:237,310,319,328` | M |
| H2 | hooks | Entire harness-side hook channel absent (`LifecycleHook`, `CallHookRequest/Response`, `enabled_hooks=16`, `InputEvent` 8/9, no `HookRouter`) | proto; `src/local.rs`; new `src/hook_router.rs` | XL |
| X1 | wasm | `src/wasm.rs` is a hand-copied fork of `src/local.rs` frozen at 0.1.1 — every wire fix must land twice or the WASM path stays broken | `src/wasm.rs` vs `src/local.rs` | L |

### High

| ID | Area | What | Where in Rust | Effort |
|---|---|---|---|---|
| W7 | wire | `TrajectoryStateUpdate.error = 4` absent — turn-level and MCP-load failures are invisible | `proto:372-381`; `src/local.rs:1034-1065` | S |
| W8 | wire | `ToolResponse.error_message = 5` absent — failing tools are reported to the harness as **successes** carrying an `"error"` key | `proto:415-420`; `src/local.rs:239-250,1127-1132,1205-1216`; `src/wasm.rs:768,851,1092` | S |
| W9 | wire | `GenerateImageToolConfig.model_name = 2` deleted in 0.1.4; we still emit it, poisoning the init JSON and orphaning `CapabilitiesConfig::image_model` | `proto:154-157`; `src/local.rs:655-658`; `src/wasm.rs:291-294` | S |
| W10 | wire | `ActionGenerateImage.aspect_ratio = 4` absent — an image step that sets it destroys the whole event | `proto:231-235` | XS |
| W11 | wire | `UserInput.Part.slash_command = 3` absent and `complex_user_input` never constructed — images, documents, audio, video and `/plan` are unreachable | `proto:349-364`; `src/connection.rs:27`; `src/local.rs:176-184`; `src/wasm.rs:1023-1031` | L |
| W12 | wire | `InitializeConversationResponse` never read — resumed-session history is silently lost | `src/local.rs:678-683`; `src/wasm.rs:314-318` | M |
| C1 | conn | `disconnect()` SIGKILLs the harness; no ws-close → stdin-close → wait → terminate → kill. Trajectory is never persisted | `src/local.rs:313-318` | S |
| C2 | conn | A freshly connected connection reports `is_idle == false`; upstream starts idle | `src/local.rs:699-700` | XS |
| C3 | conn | `receive_steps()` returns on the first IDLE sentinel, dropping steps queued behind it; sentinel suppressed for every idle after the first | `src/local.rs:111-153,1057` | M |
| C4 | conn | Idle tracking is the 0.1.1 `parent_idle`+`active_subagent_ids` shape with a `cascade_id == trajectory_id` learning heuristic in a never-reset `OnceLock` — a subagent can end the caller's turn early | `src/local.rs:52,60-62,748-760,1034-1064` | M |
| C5 | conn | `StepTracker` never clears `handled_requests` on leaving `WAITING_FOR_USER`; a re-asked question deadlocks the harness | `src/local.rs:1276-1291` | XS |
| C6 | conn | No `cancel()` and no `AntigravityCancelledError` — a halted turn is indistinguishable from a completed one | `src/local.rs:199-208`; `src/conversation.rs`; `src/error.rs:17-35` | S |
| C7 | conn | An unexpected WS close (harness crash) ends the stream silently; stderr is logged and discarded, never attached to the error | `src/local.rs:718-729,1235-1244` | S |
| S3 | policy | `Agent::start` calls `PolicyEnforcer::new(final_policies, Vec::new())` instead of `policy::enforce()` — both fail-closed validations skipped, MCP server names dropped | `src/agent.rs:293` | XS |
| S4 | policy | `allow_all()` suppresses the `workspace_only` prefix that upstream applies unconditionally | `src/agent.rs:256-283` | XS |
| S5 | policy | `normalize_wire_path` has no Rust equivalent — `file://` / `cns://` URIs reach `canonical_path` and the workspace predicate unparsed | `src/local.rs:588-595,1294-1411`; `src/agent.rs:263-278`; `src/wasm.rs:1170-1275` | M |
| S6 | policy | MCP targets still derived by sniffing `mcp_<server>_<tool>`; upstream deleted `_parse_mcp_tool` in 0.1.6 for wire-supplied `ToolCall.server_name` | `src/policy.rs:318-338`; `src/types.rs:348-358` | M |
| H3 | hooks | `HookRunner` stores one untyped `Vec<Arc<dyn DynHook>>`; no way to know which lifecycle categories are registered (blocks `enabled_hooks` and router dispatch) | `src/hooks.rs:205-227` | M |
| H4 | hooks | `on_tool_error` turns a failed tool into a **fabricated success**; upstream may only replace the message | `src/local.rs:1159-1166`; `src/wasm.rs:799-807`; `docs/hooks.md:516-559` | S |
| H5 | hooks | `HookContext` is never constructed or passed — no session→turn→operation state scoping at all | `src/context.rs:18-70`; `src/hooks.rs:16-101` | L |
| H6 | hooks | `pre_turn` hooks receive no data; upstream passes the user's multimodal prompt | `src/hooks.rs:25-34,111,237` | M |
| H7 | hooks | Built-in `post_tool_call`/`on_tool_error` — and therefore policy gating — only fire when a `tool_confirmation_request` happened first | `src/local.rs:998-1030,899-925` | M |
| T1 | tools | `ToolContext` is dead code: nothing constructs one, `call_with_context`/`needs_context` are never invoked | `src/tools.rs:142-150`; `src/agent.rs:297-300` | S |
| T2 | tools | Named subagents with their own tools/instructions (`SubagentConfig`, `HarnessConfig.custom_subagents=17`) entirely absent | `src/types.rs`; `src/agent.rs:18-50`; proto | L |
| A1 | agent | Triggers are spawned detached and never stopped; they outlive `Agent::stop()` and keep notifying a dead connection | `src/triggers.rs:44-74`; `src/agent.rs:422-425` | S |
| A2 | agent | `ChatResponse.usage_metadata` reports cumulative session usage; upstream reports last-turn usage | `src/conversation.rs:307` | XS |
| A3 | agent | The cwd `workspaces` default is computed for policy purposes only; the harness receives an **empty** workspace list | `src/agent.rs:263-268,315,360`; `src/local.rs:588-595` | XS |
| A4 | agent | `run_interactive_loop` takes an already-started `Agent`, so it can inject neither the `AskQuestionHook` nor the ASK_USER policy upgrade — the two things upstream's function exists to do | `src/interactive.rs:28-72` | L |
| X2 | tooling | `src/bin/mock_localharness.rs` speaks 0.1.1 JSON and never answers the handshake — it actively certifies the stale format | `src/bin/mock_localharness.rs:74-150` | M |
| X3 | tooling | CI has no drift detection, never builds a wasm target, never runs doctests, excludes the three directory examples, and `--all-features` is a no-op | `.github/workflows/ci.yml`; `Cargo.toml` | M |
| X4 | docs | README/`docs/mcp.md` document MCP as working (and an SSE transport upstream deleted in 0.1.2); the skill simultaneously claims MCP is file-configured via a `~/.gemini/antigravity/mcp_config.json` that has no upstream basis | `README.md:341-375`; `docs/mcp.md`; `skills/.../mcp_tools.md:11-17` | S |

### Medium

| ID | Area | What | Where | Effort |
|---|---|---|---|---|
| W13 | wire | `HarnessSideTools.search_web=12` / `read_url_content=14` absent — cannot be enabled or disabled | `proto:104-116`; `src/local.rs:626-659` | M |
| W14 | wire | `InputConfig.env = 5` absent; harness spawned with only `SHELL`/`PATH` | `proto:15-20`; `src/local.rs:422-458` | S |
| W15 | wire | `HarnessConfig.session_continuation_mode = 19` absent. Breaking for any caller-supplied `conversation_id`: the harness attempts a resume and fails when the conversation does not exist (§2.1). Medium only for the default empty-`cascade_id` path | `proto:26-41`; `src/agent.rs:547-550`; `examples/persistence.rs` | S |
| W16 | wire | `HarnessConfig.retry_config = 20` + `RetryConfig`/`ModelAPIRetryConfig`/`ModelOutputRetryConfig` absent | `proto:26-41` | S |
| W17 | wire | `HarnessConfig.custom_subagents = 17` + `CustomAgent` absent (wire half of T2) | proto | L |
| W18 | wire | `HarnessConfig.app_data_dir` populated from `save_dir`; the two are distinct upstream | `src/local.rs:454,674`; `src/wasm.rs:310` | S |
| W19 | wire | `GeminiConfig.enable_google_search` / `enable_url_context` are public Rust options with **no field anywhere** in the 0.1.9 schema | `src/types.rs:116-121`; `src/local.rs:581-582` | S |
| C8 | conn | Prompt control-character sanitisation missing (upstream applies it to text `Part`s) | `src/local.rs:176-183` | XS |
| C9 | conn | No `127.0.0.1` fallback on connect; failure message omits the harness stderr | `src/local.rs:483-506` | S |
| C10 | conn | WS message size at tungstenite defaults; upstream sets `max_size=None` for multi-MB tool payloads | `src/local.rs:492` | XS |
| C11 | conn | `conversation_id` not validated (≥32 chars, `[a-zA-Z0-9-]`) | `src/agent.rs:547-550` | XS |
| C12 | conn | `DebugConfig` (server-side tracing + logging level) absent from config and the `Connection` trait | `src/agent.rs:18-50`; `src/connection.rs` | S |
| C13 | conn | `save_dir` not defaulted to a `antigravity_*` temp dir when unset | `src/agent.rs:357`; `src/local.rs:454` | XS |
| C14 | conn | `Step.error` read only from `StepUpdate.error_message`, never `ActionError.error_message` — a step can carry `http_code=403` with an empty error string | `src/local.rs:853-854` | XS |
| C15 | conn | `supplemental_media` always empty; tool-returned media reaches the model as opaque base64 inside `response_json` | `src/local.rs:248,1130,1214`; `src/wasm.rs:769,852,1093` | M |
| C19 | conn | Concurrent `receive_steps()` calls silently split the stream; upstream raises | `src/local.rs:92-157` | S |
| H8 | hooks | `post_tool_call` never sees a failed `ToolResult` (`else if` with `on_tool_error`) | `src/local.rs:1159-1169`; `src/wasm.rs:798-808` | XS |
| H9 | hooks | An erroring `OnToolError` hook aborts the chain and the failure is swallowed | `src/hooks.rs:276-294`; `src/local.rs:1160` | XS |
| H10 | hooks | Custom-tool pre-tool denial replaces the hook's `HookResult.message` with a fixed string | `src/local.rs:1110-1139` | XS |
| H11 | hooks | No `ToolExecutionError` — `on_tool_error` gets a bare `anyhow::Error` with no `tool_name`/`server_name` | `src/error.rs`; `src/local.rs:917-921,1160` | S |
| H12 | hooks | `start_subagent` completion produces no `post_tool_call` — a regression against 0.1.1 as well as 0.1.9 | `src/local.rs:1034-1065` | M |
| H13 | hooks | `HookContext`/`ToolContext` stuck on the pre-0.1.7 `get`/`set` store: no `update_state`, no `lock()`, non-reentrant mutex, poisoning swallowed | `src/context.rs:44-69`; `src/tool_context.rs:41-73` | M |
| H19 | docs | `docs/hooks.md` documents four never-dispatched hooks, the inverted `on_tool_error` contract, and `HookContext` usage as working features | `docs/hooks.md:114-189,205-265,428-467,516-559` | S |
| S7 | policy | `workspace_only()` scopes 5 tools; upstream's `file_tools()` is 3 — we deny `list_directory`/`search_directory` outside the workspace where upstream allows them, while omitting `FIND_FILE` which does carry a path | `src/policy.rs:167-173` | XS |
| S8 | policy | Built-in tool identifiers are UPPERCASE; upstream's public/wire names are lowercase, mapped via `PROTO_FIELD_TO_SDK_NAME` | `src/types.rs:196-211`; `src/local.rs:1301-1409` | M |
| S9 | policy | Predicate `args` are hand-built per tool: `ActionEditFile.diff_block` is dropped entirely, and non-proto keys are injected | `src/local.rs:1294-1411` | M |
| S10 | policy | `when` predicates and `ask_user` handlers are sync-only; upstream awaits both | `src/policy.rs:34-36,388-455` | L |
| S12 | policy | `BuiltinTools::read_only()` omits `FINISH` (present upstream since 0.1.1) and `READ_URL_CONTENT`; `has_write_tools` is therefore permanently true | `src/types.rs:213-221`; `src/agent.rs:239-240` | XS |
| T3 | tools | Invented `google_search`/`web_search` fallback that shells out to `python3` to scrape DuckDuckGo — no upstream equivalent, undeclared network egress, hidden Python dependency, and it shadows the real harness-side `search_web` | `src/tools.rs:168-293` | S |
| T4 | tools | No `_coerce_args` (0.1.8): a model returning `{"count":"5"}` fails against `Value::as_u64` — including in `examples/custom_tools.rs:87-90` | `src/tools.rs:150` | M |
| T5 | tools | Absent `arguments_json` yields `Value::Null` instead of `{}`; zero-arg tools break | `src/local.rs:1074`; `src/wasm.rs:713` | XS |
| T6 | tools | `ToolCall`/`ToolResult` lack `server_name`; `ToolResult` discards the typed error | `src/types.rs:346-374`; `src/tools.rs:159-166` | S |
| A5 | agent | `Conversation::send` has an empty `if !self.conn.is_idle() { }` — back-to-back sends corrupt turn boundaries and lose turn-1 usage | `src/conversation.rs:140-152` | M |
| A6 | agent | No `wait_for_idle` / `wait_for_wakeup` on `Connection` or `Conversation` (both present upstream since 0.1.1) | `src/connection.rs:16-68` | M |
| A7 | agent | No `get_last_structured_output()` / `ChatResponse::structured_output()`; `response_schema` is not validated as JSON | `src/conversation.rs`; `src/types.rs:601-610`; `src/agent.rs:557-560` | S |
| A8 | agent | `Conversation::new` cannot seed restored history (`_initial_history`, 0.1.5) | `src/conversation.rs:57-69`; `src/agent.rs:369-372` | M |
| A10 | agent | Triggers receive the raw `AnyConnection` and can forge tool results, answer questions, halt turns and disconnect; upstream narrows to a one-method `TriggerContext` | `src/triggers.rs:11-32` | S |
| A11 | agent | `trigger_helpers::every()` sends a fixed string instead of invoking a callback, and accepts a zero interval | `src/trigger_helpers.rs:16-45` | S |
| A12 | agent | `on_file_change` absent; `FileChange`/`FileChangeKind` are dead types still living in `types` (upstream moved them to `triggers` in 0.1.4) | `src/trigger_helpers.rs`; `src/types.rs:970-990` | M |
| A13 | agent | `BuiltinTools` has 10 of upstream's 13 (no `ASK_QUESTION`/`SEARCH_WEB`/`READ_URL_CONTENT`); the default list is hardcoded three times | `src/types.rs:163-194`; `src/agent.rs:208-233`; `src/local.rs:597-608` | S |
| X5 | tooling | `install_harness.sh` pins 0.1.1; README says `pip install google-antigravity` (0.1.9). Nothing warns they are incompatible | `scripts/install_harness.sh:7`; `README.md:28-38` | XS |
| X6 | tooling | README omits the `protoc` prerequisite that `build.rs` requires; CI installs it, users are not told | `README.md:18-24`; `build.rs:8-12` | XS |
| X7 | docs | `gemini-3.5-flash` hardcoded in 20+ places; upstream default is `gemini-3.6-flash` since 0.1.8, image model `gemini-3.1-flash-lite-image` since 0.1.7 | `src/types.rs:12,15` + examples/docs/skills/tests | S |
| X8 | docs | README documents `enable_google_search` and the DuckDuckGo fallback as features | `README.md:413-419` | S |
| X9 | tooling | Upstream added an `otel` extra + `utils/otel.py` in 0.1.5; the crate has no `[features]` section at all | `Cargo.toml` | L |
| X10 | tests | Tests pin the pre-0.1.4 `GeminiConfig`/`GenerationConfig`/`ModelEntry` types and the 11-positional-arg `LocalConnectionStrategy::new` | `tests/integration_tests.rs`; `tests/documentation_examples.rs:32-45` | S |
| X11 | docs | Built-in tool tables list 10 of 13 tools | `docs/tools.md`; `skills/.../built_in_tools.md:7-19` | S |

### Low

| ID | Area | What | Where | Effort |
|---|---|---|---|---|
| W20 | wire | `ClientInfo.os = 4` / `os_version = 5` absent (binary handshake, so harmless — telemetry only). `language_version` is also always `"unknown"`: neither `RUSTC_VERSION` nor `CARGO_PKG_RUST_VERSION` is set | `proto:9-13`; `src/local.rs:447-451,1430-1436` | XS |
| W21 | wire | `user_questions.enabled` hardcoded `true`; every sibling config is gated on `active_tools` | `src/local.rs:636-638` | XS |
| W22 | wire | `UsageMetadata` counters declared `int32`; upstream is `uint64` in **every** release including 0.1.1 (original transcription error) | `proto:453-459`; `src/types.rs:378-391` | S |
| W23 | wire | `STATE_TERMINAL_ERROR = 5` removed in 0.1.3; the enum value, the `Some(5)` arms, the break-the-reader-loop block, the mock and an integration test all still depend on it | `proto:186`; `src/local.rs:824,889-897,901`; `src/bin/mock_localharness.rs:104`; `tests/integration_tests.rs:176-233` | S |
| W24 | wire | Local `Struct`/`Field`/`Value`/`ListValue`/`NullValue` were moved to `.genai` (content.proto) in 0.1.9, and `Value.kind` slot 7 changed from `Media` to `Content`. Blast radius is nil today — neither SDK populates `ToolCall.arguments`/`ToolResponse.response` | `proto:5-7,383-406,412,419` | M |
| W25 | wire | `CustomSystemInstructions.Part.template = 2` / `SystemInstructionTemplate` absent (upstream Python never builds it — proto completeness only) | `proto:77-85` | S |
| W26 | wire | `HarnessConfig.tool_output_truncation = 18` absent (upstream Python never sets it either) | `proto:26-41` | S |
| W27 | wire | `Tool.defer_loading = 5`, `HarnessSideTools.tool_search_config = 15` absent (neither SDK sets them) | `proto:97-116` | S |
| W28 | tooling | Upstream is Edition 2023 (`syntax='editions'`, `edition=1001`); we compile proto3 with hand-added `optional`. A naive regeneration that drops those markers turns "absent" into "zero" and breaks the `.is_some()`-driven step classifier and request dedup | `proto:1`; `build.rs:7-11`; `src/local.rs:771-780,786-805` | M |
| C16 | conn | No `StepType::Thinking`; thinking-only steps classify as `Unknown` | `src/types.rs:395-414`; `src/local.rs:786-805` | XS |
| C17 | conn | `TARGET_MODEL` (2) mapped to `StepTarget::Environment`; upstream leaves it `UNKNOWN`. `StepTarget::Unspecified` is unreachable | `src/local.rs:828-832` | XS |
| C18 | conn | Question-hook responses zipped positionally over all questions, not over multiple-choice indices; no error-fallback answer, so an SDK error leaves the harness waiting | `src/local.rs:934-981` | XS |
| C20 | conn | `app_data_dir` not `~`-expanded, not canonicalised, not validated absolute; `HOME` unset falls back to `/tmp/.gemini/antigravity` | `src/agent.rs:271-278` | XS |
| H15 | hooks | `dispatch_interaction` takes a raw slice and returns `Option`, losing the `AskQuestionInteractionSpec`, the `OperationContext` and the "no handler" `HookResult` | `src/hooks.rs:74-80,296-307` | S |
| H16 | hooks | `on_compaction` receives `&str` instead of the compaction `Step` | `src/hooks.rs:95-100,328-334` | XS |
| H17 | hooks | `_PreStepHook`/`_PostStepHook` (0.1.5, the basis of upstream's OTel integration) absent | `src/hooks.rs`; `src/local.rs:1270-1290` | S |
| H18 | hooks | `PreToolResult.modified_arguments_json` (0.1.7 harness capability) unrepresentable | proto | XS |
| S13 | policy | `policy::safe_defaults(handler)` missing | `src/policy.rs` | XS |
| S14 | policy | Startup guard omits the `has_mcp_servers` term and the pre-tool-decide-hook escape hatch | `src/agent.rs:285-290` | XS |
| S15 | policy | MCP builders accept no `when`/`name`; auto-names use `allow_` where upstream uses `approve_` (user-visible in deny messages) | `src/policy.rs:477-539` | XS |
| S16 | policy | Relative `canonical_path` denied outright; upstream resolves against cwd | `src/policy.rs:181-183` | XS |
| S17 | docs | `docs/agent.md:693` links `policy.md`, which does not exist; `docs/hooks.md:305,357,375` ship `policy::deny("run_command")` examples that can never match | `docs/` | S |
| T7 | tools | `register()` silently overwrites a duplicate name; unknown-tool message differs from upstream's pinned wording | `src/tools.rs:134-139,216-221` | XS |
| T8 | tools | Tool list built from a `HashMap`, so `HarnessConfig.tools` order is randomised per process | `src/tools.rs:114`; `src/local.rs:513-523` | XS |
| T9 | tools | `ToolContext` retains `is_idle()`/`send()`, deleted upstream in 0.1.4 | `src/tool_context.rs:45-53` | S |
| T10 | tools | `process_tool_calls` runs a batch sequentially; upstream gathers | `src/tools.rs:146-225` | S |
| T11 | tools | `McpServerConfig::Stdio` has no `env`; no variant has `timeout_seconds` | `src/types.rs:247-320` | XS |
| T12 | docs | `docs/tools.md` documents `ToolRunner::execute` (doesn't exist), a `GrepSearch` builtin (doesn't exist), the wrong storage type and the wrong `process_tool_calls` signature | `docs/tools.md:101-107,146-180,226-231` | S |
| A14 | agent | `Agent::chat` accepts empty/whitespace prompts; upstream raises | `src/agent.rs:403-405` | XS |
| A16 | agent | `Agent::register_hook`/`register_trigger`/`register_tool` retained; upstream removed them in 0.1.4 in favour of config-only registration | `src/agent.rs:157-171` | XS |
| A17 | agent | `Connection` publicly exposes `send_tool_response`/`send_tool_confirmation`/`send_question_response`; upstream has one private `_send_tool_results` | `src/connection.rs:43-64` | M |
| A18 | agent | `Conversation::last_turn_usage` public; upstream made it private in 0.1.4 (**do not hide until A2 lands** — it is currently the only way to get turn usage) | `src/conversation.rs:121-123` | XS |
| A19 | agent | Rust-only `ChatResponse.steps` carries whole-session history although `turn_start_indices` is tracked | `src/conversation.rs:306-313` | XS |
| X12 | tests | The in-file WASM mock test is a closed Rust→Rust loop encoding 0.1.1 semantics; it will keep passing while the real path hangs | `src/wasm.rs:1536-1591` | S |
| X13 | tooling | `install_harness.sh` falls back to a `macosx_10_9_x86_64` wheel that has never been published; unzips into and then `rm -rf`s a `google/` directory in the caller's CWD; downloads ~40 MB with no checksum although PyPI exposes `digests.sha256` | `scripts/install_harness.sh:43-47,141-166` | S |
| X14 | docs | README says Windows is unsupported; `win_amd64`/`win_arm64` wheels containing `localharness.exe` have shipped since 0.1.4, and `src/agent.rs:623-627` already handles them | `README.md:44-51` | XS |
| X15 | docs | `ARCHITECTURE.md:78-131` and `docs/connections.md:355` document the 0.1.1 handshake and `STATE_IDLE` | docs | S |
| X16 | docs | `docs/connections.md:204-219` publishes an exhaustive `WasmConnectionStrategy` struct literal; the struct is all-public with no ctor or `#[non_exhaustive]`, so every field addition is a semver break | `src/wasm.rs:73-83` | S |
| X17 | docs | `SKILL.md:53-92` uses 21 `file:///Volumes/goldcoders/...` absolute links; `plugin.json` claims Apache-2.0 / author "Google" for an MIT repo | `skills/` | XS |
| X18 | docs | `examples/README.md:9` misdescribes `hello_world`; wasip1 vs wasip2 inconsistency; no example constructs `WasmConnectionStrategy` at all despite the SDK being an optional dep of `leptos_ssr_axum` | examples | S |
| X19 | tests | No example or test exercises context-aware tools, which is why T1 went unnoticed | `examples/`, `tests/` | S |

> The types / model-configuration subsystem was audited separately and its 56 findings (`MC-*`, `CT-*`) are tabled in **§4.8** rather than repeated here.

### Completeness pass

A final agent walked the 0.1.9 tree looking for what the eight subsystem audits never opened — its answer was the three files that did not exist at 0.1.1 (`models.py`, `litert_connection*.py`, `local_openai_connection*.py`) plus the public `__init__.py` export list.

| ID | Area | What | Where in Rust | Effort |
|---|---|---|---|---|
| N1 | models | `GOOGLE_GENAI_USE_VERTEXAI` / `GOOGLE_GENAI_USE_ENTERPRISE` routing and `GOOGLE_CLOUD_PROJECT` / `GOOGLE_CLOUD_LOCATION` hydration absent — an environment configured for Vertex silently gets the Gemini Developer API | `src/local.rs:386-414` (`grep GOOGLE_CLOUD src/` is empty) | S |
| N2 | models | Vertex validation accepts an API key in place of project+location; upstream validates per model target, not once globally | `src/local.rs:398-414` vs `models.py:126-128` | S |
| N3 | tools | `connections/local/types.py`'s structured per-tool result models (`RunCommandResult`, `ListDirectoryResult`, `SearchWebResult`, `ReadUrlContentResult` …) have no Rust counterpart — `post_tool_call` hooks get raw display text | `src/local.rs:1413-1428` | M |
| N4 | strategies | LiteRT (on-device) and LocalOpenAI (Ollama / LM Studio, via `GemmaEndpoint`) strategies absent; both are top-level upstream exports since 0.1.6 | absent | XL |
| N5 | types | `ThinkingLevel::ExtraHigh` (0.1.8) missing. Watch the serde trap: the blanket `rename_all = "lowercase"` would emit `extrahigh`, not `extra_high` | `src/types.rs:17-29` | XS |
| N6 | types | `CapabilitiesConfig` does not reject `enabled_tools` and `disabled_tools` being set together — **only on the direct-strategy path**. `Agent::start` already validates this (`src/agent.rs:199-203`, pinned by `tests/integration_tests.rs:86-104`); the completeness pass missed that and its "Rust silently lets `enabled_tools` win" is false for the `Agent` path | `src/local.rs:610-623`; `src/wasm.rs:245-260` | XS |
| N7 | types | `SystemInstructionSection.title` has no default (upstream: `"user_system_instructions"`) and there is no `&str` → appended-section shorthand | `src/types.rs:125-131`; `src/agent.rs:487-489` | XS |
| N8 | agent | `AgentBuilder::policies` does not flatten nested groups; upstream's `_validate_policies` does, which is why every group builder can be composed inline | `src/agent.rs:~505` vs `connection.py:138-159` | XS |

N1, N2, N5 and N7 fold into WP-4; N3 and N6 into WP-7; N8 into WP-3; N4 is gated on §6 decision 2.

---

## 4. Findings in detail, by subsystem

Only findings whose mechanism is not obvious from the table are expanded. Everything else is fully specified by the table row plus the work package that closes it.

### 4.1 Wire protocol and generated code

#### W1 — strict pbjson deserializers (breaking, XS)
- **Upstream:** `0.1.9/.../local_connection.py:466-470` uses `json_format.Parse(raw_msg, event)`. Note the nuance: Python's `Parse` is *also* strict — but its descriptor is always current, so it never sees an unknown field.
- **Rust:** `build.rs:14-18` (verified verbatim). pbjson-build 0.6.2: `ignore_unknown_fields` is a plain `#[derive(Default)]` bool at `src/lib.rs:106` with a single setter at `:171`; when false, `generator/message.rs:741-764` emits `unknown_field`. `generator/enumeration.rs:154` emits `unknown_variant` **regardless of the flag**.
- **Change:** add `.ignore_unknown_fields()` to the builder chain — a deliberate divergence from upstream toward forward compatibility, justified because our descriptor will always lag. Separately, `src/local.rs:1229-1235` must stop being a silent `tracing::error!`: push `Err(...)` onto `step_tx` the way `src/local.rs:1240` already does for WS read errors, so a future schema drift surfaces as an error instead of a hang.
- **Caveat:** the flag does **not** cover enum variants, so W2 must be fixed independently.

#### W2 — `STATE_FULLY_IDLE` (breaking, S)
- **Upstream:** `upstream-0.1.9-localharness.proto:424-429`. Descriptor history: 0.1.1–0.1.4 `STATE_IDLE=2`; 0.1.5 adds `STATE_CANCELLED=3`; 0.1.8 adds a transient `STATE_WAITING_FOR_TASKS=4`; 0.1.9 renames 2 → `STATE_FULLY_IDLE` and drops 4. `strings .../bin/localharness` contains `STATE_FULLY_IDLE` and no `STATE_IDLE`.
- **Rust:** `proto/localharness.proto:376` (verified). `src/local.rs:1044/1048` compares `tsu.state == Some(1)/Some(2)` — code that is never reached because the whole event fails to parse first.
- **Change:** rename the value (the old name cannot be kept — one value, one name), add `STATE_CANCELLED = 3`, and add a cancelled arm emitting `AntigravityExecutionError { message: tsu.error.unwrap_or("Turn cancelled") }` then idle + sentinel, mirroring `event_processor.py:561-568`. Mirror in `src/wasm.rs:673-704`.

#### W3 — `models = 15` replaces `gemini_config` (breaking, L)
- **Upstream:** `upstream-0.1.9-localharness.proto:37-61`, `:83-117`. Builder: `local_connection.py:124-164` (`build_models_proto`).
- **Rust:** verified `proto/localharness.proto:28-31` still carries the `model_config` oneof, and `src/local.rs:663-665` sets `harness_config::ModelConfig::GeminiConfig(proto_gemini)`.
- **Change:** delete the oneof and the `GeminiConfig`/`GemmaConfig` messages; add `repeated ModelConfig models = 15` plus `ModelConfig{name=1, repeated ModelType types=2, oneof endpoint{gemini_api_endpoint=3, vertex_endpoint=4, gemma_endpoint=6, custom_endpoint=7}}`, `GeminiAPIEndpoint{base_url=1, map<string,string> http_headers=2, api_key=3, GeminiModelOptions options=4}`, `VertexEndpoint{base_url=1, http_headers=2, project=3, location=4, options=5}`, `GeminiModelOptions{thinking_level=1}`, `enum ModelType{UNSPECIFIED=0,TEXT=1,IMAGE=2}`. **Scope note:** `build_models_proto` (`local_connection.py:124-164`) only ever emits `gemini_api_endpoint` or `vertex_endpoint` and raises `ValueError` otherwise, so the Rust builder for the *local* strategy need only cover those two. `GemmaEndpoint` is **not** dead schema, though: `local_openai_connection.py:51` and the LiteRT strategy construct it directly (`local_openai_connection_test.py:48-49`, `litert_connection_test.py:341-342`). Keep the `gemma_endpoint = 6` arm in the regenerated proto — it is the wire shape an Ollama / LM Studio backend would need (N4, §6 decision 2).
- **Depends on:** W9 (image model moves here as a second `ModelConfig` with `types=[MODEL_TYPE_IMAGE]`, default `gemini-3.1-flash-lite-image`) and W19 (`enable_google_search`/`enable_url_context` have nowhere to go).

#### W4 / W12 — the handshake (breaking / high)
`local_connection.py:1162-1176` sends the init event and then does an **unconditional** `await ws.recv()`, parses an `OutputEvent`, and builds `initial_history` from `init_resp.initialize_conversation_response.history` (a repeated `StepUpdate`). `src/local.rs:678-682` sends and falls straight through to spawning the reader (`:685-734`); `src/wasm.rs:314-319` is identical. So our very first read fails today.

Two corrections to the obvious fix: (a) upstream **ignores** `init_resp.cascade_id` — `LocalConnection.conversation_id` returns `self._processor.main_trajectory_id`, set from the first `StepUpdate`'s `trajectory_id` (`event_processor.py:478-480`), so do not wire `cascade_id` into `learned_id`; fix the learning heuristic per C4 instead. (b) Arms 14/15 are conditional on `enabled_hooks` / `session_end_request`, neither of which we send — adding arm 13 alone is the minimum viable fix, and arms 14/15 belong with WP-8.

#### W5 — `StepUpdate` arms 33–36 (breaking, L)
Verified `proto/localharness.proto` ends the action block at `error = 32` then jumps to `request_text = 50`. Upstream: `mcp_tool=33` (0.1.3), `search_web=34` (0.1.4), `read_url_content=35` (0.1.6), `custom_tool=36` (0.1.7). `custom_tool` is the load-bearing one: `event_processor.py:462-474` states verbatim that the harness sends *both* a `StepUpdate` with `custom_tool` **and** a websocket `tool_call` event, suppresses the duplicate client-side, and keeps the `from_dict` path because no `tool_call` events are replayed during history resumption. So every custom-tool turn drops a `StepUpdate` today, and resumed sessions lose all custom-tool history.

#### W6 — MCP never reaches the wire (breaking, M)
`LocalConnectionStrategy.mcp_servers` exists (`src/local.rs:345`), is populated from the builder (`src/agent.rs:365,564,569`), and is validated against by the policy layer (`src/policy.rs:224-250`, which raises *"MCP policies were detected, but 'mcp_servers' was not …"*). A full grep of `src/` returns exactly those sites — it is never read inside `connect()`, and the verified `HarnessConfig` literal has no MCP field. `src/wasm.rs` has no such field at all. The 0.1.9 binary carries `protobuf:"bytes,14,rep,name=mcp_servers,json=mcpServers"`.

This is rated **breaking** rather than high because the SDK accepts MCP config, enforces MCP-scoped policies against it, documents it as working, and then tells the harness nothing — the user gets a security posture that is confidently wrong.

Add `McpServerConfig{name=1, oneof transport{stdio=2, http=3}, enabled_tools=4, disabled_tools=5, auth_provider_type=6, timeout_seconds=7}`, `McpStdioTransport{command=1, args=2, map<string,string> env=3}`, `McpHttpTransport{url=1, map<string,string> headers=2}`. Note `_to_mcp_server_proto` (`local_connection.py:638-666`) never sets `auth_provider_type`, so field 6 is optional for parity.

#### W8 — tool errors are reported as successes (high, S)
Upstream branches at `event_processor.py:778-805`: `if result.error is not None: ToolResponse(id=..., error_message=result.error)` with `response_json` deliberately unset. All six Rust construction sites instead build a successful response whose JSON happens to contain an `"error"` key: `src/local.rs:239-242` (`send_tool_response`), `:1129` (policy denial, literal `"{\"error\": \"Execution denied by hook policy\"}"`), `:1205-1206` (runtime failure), and `src/wasm.rs:768,851,1092`. The harness's error classification — and therefore its `ON_TOOL_ERROR` lifecycle — never fires.

#### W11 — no multimodal, no slash commands (high, L)
`proto/localharness.proto:356-361` declares `UserInput.Part` with only `text=1` and `media=2`; there is no `SlashCommand` message anywhere in the file. A repo-wide grep for `complex_user_input`/`ComplexUserInput`/`SlashCommand` in `src/` returns **two** hits, both `input_event::Event::UserInput(String)` (`src/local.rs:177`, `src/wasm.rs:1025`). The constraint is baked into the trait — `fn send(&self, content: &str)` at `src/connection.rs:27` with impls at `:128,:286` — so this is a public-API change, not a new method.

Upstream dispatch: `local_connection.py:294-320` (str → `user_input`; anything else → `complex_user_input`), `to_proto_input_content` at `:544-568` (SlashCommand → `slash_command`; Image/Document/Audio/Video → `Media`; `TypeError` otherwise), pinned by `local_connection_test.py:3385-3394`. `_sanitize_prompt` (`:219-229`) is applied **only** to text `Part`s, never to the plain-string path — so C8's sanitiser is needed only on the new path.

Rust already has the target types: `Content`/`ContentPrimitive` at `src/types.rs:843-905`, with `Content::from_file` — and **zero consumers outside `types.rs`**, despite `README.md:201-224` documenting them.

#### W19 — options with nowhere to go (medium, S)
`enable_url_context = 5` / `enable_google_search = 6` exist **only** in `upstream-0.1.1-localharness.proto:44-45`, as fields of the `GeminiConfig` message that W3 deletes. Zero occurrences in the 0.1.9 schema, zero in the 0.1.9 Python tree. Rust exposes both as public options (`src/types.rs:116-121`) and writes them (`src/local.rs:581-582`). They became **unrepresentable**, not relocated: deprecate them as documented no-ops with a warning on set, rather than trying to map them onto `ModelConfig`. The 0.1.9 answer is `BuiltinTools::SEARCH_WEB` / `READ_URL_CONTENT` (W13, A13).

#### W28 — regeneration hazard (low, M)
The 0.1.9 `FileDescriptorProto` reports `syntax='editions'`, `edition=1001`, `dependency=['google/antigravity/proto/content.proto']`. The rendered `.proto` text drops `optional` markers — that is a rendering artefact of editions' implicit explicit-presence, **not** the real schema. Upstream genuinely depends on presence (`usage_metadata.HasField("prompt_token_count")` at `event_processor.py:176-188`; `tsu.HasField("error")` at `:539/:554/:565`), and so do we: the step-type classifier (`src/local.rs:786-805`) and request dedup (`:771-780`) are built entirely on `.is_some()`. A naive proto3 regeneration that drops `optional` silently converts *absent* into *zero-valued present* and breaks both. Keep `syntax = "proto3"` and mark every singular field `optional`; prost-build 0.12 does not support editions.

### 4.2 Connection / event processing

`src/local.rs` is a faithful port of the 0.1.1 `local_connection.py` and has tracked none of the 0.1.2–0.1.9 rework.

#### C1 — ordered shutdown (high, S)
Upstream's `disconnect()` (`local_connection.py:407-455`, `_PROCESS_WAIT_TIMEOUT_SECONDS = 3*60` at `:53`) does six things in order: cancel processor tasks, cancel+await the reader, close the WebSocket (0.5 s timeout), **close stdin**, `wait(180)`, escalate to `terminate()` (1 s) then `kill()`. The tests spell out why: *"The Go harness monitors stdin for EOF. On EOF it runs cleanupAllAgents which persists trajectory state to disk. Without closing stdin, the trajectory is never saved"* (`local_connection_test.py:3110-3115`) and *"Killing it immediately would lose the trajectory"* (`:3126-3130`).

Ours is `src/local.rs:313-318`: lock, `proc.kill().await`, return. A SIGKILL runs no Go defers, so nothing under `app_data_dir` is written — which also pre-breaks W15's RESUME and W12's `initial_history`. `src/local.rs:54` already holds `child_stdin: Arc<Mutex<Option<ChildStdin>>>` and never uses it. `src/wasm.rs:1157-1160` is a bare `Ok(())`, which is correct for a transport with no subprocess but must still dispatch session-end (H1) and close the socket.

#### C3 — the sentinel protocol is lossy (high, M)
Upstream's loop `continue`s on `IDLE_SENTINEL` and re-evaluates `self.is_idle and self._processor.step_queue.empty()` at the loop head (`local_connection.py:338-359`); the regression it guards is `local_connection_test.py:295-341`. Ours returns immediately (`src/local.rs:117-122` and the duplicated blocking path at `:137-141`). Replay upstream's own scenario — idle, real step, idle — and the queue `[sentinel, step2, sentinel]` yields nothing after the first sentinel; upstream yields `"Step content after idle"`. Separately, `conn_is_idle.swap(true, SeqCst)` at `:1057` suppresses the sentinel for every idle after the first, whereas upstream enqueues one on every FULLY_IDLE (`event_processor.py:558-559`).

Fix: loop instead of returning; terminate only when `is_idle && rx.is_empty()`; `store` instead of `swap`; and replace the `step.id == "IDLE_SENTINEL"` magic string with a channel enum (`StepEvent::{Step, Idle, Close}`) so a real step cannot collide.

#### C4 — the idle state machine (high, M)
`src/local.rs:60-62` keeps `parent_idle` and `active_subagent_ids`, and `:1041-1057` requires `*p_idle && active_subs.is_empty()` — a direct port of `0.1.1/.../local_connection.py:888-915`. 0.1.9 deleted all of it: subagent trajectories `return` early (`event_processor.py:539-542`) and only the main trajectory drives `is_idle`.

Worse, `is_subagent` (`src/local.rs:1037`) depends on `learned_cascade`, which is only populated from a `StepUpdate` where `cascade_id == trajectory_id` (`:750-760`) — a condition upstream does **not** impose. `event_processor.py:478-480` is unconditional on the first `StepUpdate` carrying any `trajectory_id`. If no such matching StepUpdate has arrived (a resumed session, or a subagent trajectory reporting first), `is_subagent` is false for everything and a subagent's FULLY_IDLE ends the caller's turn. And `learned_id` is a `OnceLock` (`:52,713`) that `send()` never resets, whereas upstream calls `reset_for_turn()` (`event_processor.py:379-386`) from `send()` (`local_connection.py:302`).

Fix: `main_trajectory_id: Arc<Mutex<Option<String>>>`, set from the first non-empty `trajectory_id`, cleared in `send()` alongside `step_trackers`; delete `parent_idle`/`active_subagent_ids`; return early for non-main trajectories (logging `tsu.error` per `event_processor.py:538-541`).

#### C15 / W-media — supplemental media (medium, M)
All six `ToolResponse` sites hardcode `supplemental_media: Vec::new()`. Upstream's `_extract_media_from_result` (`event_processor.py:65-101`) recurses lists/dicts pulling out `Image/Document/Audio/Video`, substitutes `f"Returned {len(media)} media attachment(s)."` when the result was entirely media, and attaches each as `Media(mime_type, data, description)` — with the stated motivation that media must *"reach the model as supplemental media instead of opaque base64 in response_json"*. The `Media` proto message already exists on our side; only the extraction is missing. Note `Tool::call` returns a bare `serde_json::Value` (`src/tools.rs:29-32`), so there is no typed media object to detect — a reserved JSON shape or a `ToolResult.media` field is needed first.

### 4.3 Hooks

Upstream moved lifecycle hooks into the harness across 0.1.5–0.1.6: the harness calls back over the same WebSocket via `CallHookRequest`, driven by `HarnessConfig.enabled_hooks` computed from `_get_enabled_hooks()` (`local_connection.py:1004-1043`), and the SDK answers with `CallHookResponse` on `InputEvent` field 8. Our `HookRunner` is a faithful port of the 0.1.1 *client-side* design.

#### H1 — four hooks are documented but dead (breaking, M)
`grep -rn 'dispatch_pre_turn|dispatch_post_turn|dispatch_session_end|dispatch_on_compaction' src/ tests/ examples/` returns only the definitions (`src/hooks.rs:237,310,319,328`) and their own unit tests (`:501-746`). `src/conversation.rs` contains no reference to `hook` at all. The only `HookRunner` calls in `src/local.rs` are session_start (`:1249`), pre_tool_call (`:1009,:1103`), post_tool_call (`:914,:1168`), on_tool_error (`:921,:1160`) and interaction (`:960`); `src/wasm.rs` mirrors exactly that set.

This is a regression against **both** upstream versions. 0.1.1 already dispatched `pre_turn` in `send()` (`local_connection.py:507-517`), `post_turn` on the terminal model step (`:605-609`), `session_end` in `disconnect()` (`:686-690`) and compaction on `StepType.COMPACTION` (`:800-805`). 0.1.9 moved the first three onto the harness (`hook_router.py:130-171`) and kept compaction local (`event_processor.py:504-510`).

A user registering the `pre_turn` rate limiter that `docs/hooks.md:428-467` presents as a working example gets a hook that is never called and a turn that is never denied.

#### H2 — the hook channel (breaking, XL)
`grep -i hook proto/localharness.proto` returns zero matches. Needed: `enum LifecycleHook` with the **exact upstream value names** — `LIFECYCLE_HOOK_UNSPECIFIED=0 … LIFECYCLE_HOOK_ON_TOOL_ERROR=7` (protojson serialises enums by value name; the short forms would produce JSON the Go harness rejects), `CallHookRequest{request_id=1, name=2, LifecycleHook type=7, oneof args{pre_turn_args=3, post_turn_args=4, pre_tool_args=5, post_tool_args=6, on_tool_error_args=8}}`, `CallHookResponse{request_id=1, oneof result{pre_turn_result=2, pre_tool_result=3, empty_result=4, error_message=5, on_tool_error_result=6}}`, the five `*Args` and four `*Result` messages (`upstream-0.1.9-localharness.proto:512-579`), plus `OutputEvent.call_hook_request=14` / `InputEvent.call_hook_response=8`.

The router (`hook_router.py:76-313`) must: build a response keyed on `request_id`; dispatch through a 7-entry table; fall back to `empty_result` with a warning for unhandled types (`:299-305`); convert **any** hook failure into `resp.error_message = f"Hook failed: {e!r}"` (`:309-311`) so a broken hook cannot kill the reader; and **always** send exactly one response (`:313`) — even with no hook runner registered (`event_processor.py:412-423`), because an unanswered request blocks the harness's turn.

Implementation note: `std::panic::catch_unwind` cannot wrap an async handler (the future is not `UnwindSafe`, and a panic across an await point is not caught). Use `futures_util::FutureExt::catch_unwind` on `AssertUnwindSafe`, or run each handler in the spawned task and map both `Err(e)` and `JoinError::is_panic()` onto `ErrorMessage`.

**Ordering constraint:** `enabled_hooks` must not be emitted before the request handler exists. Shipping field 16 alone converts a silent no-op into a mid-turn deadlock.

#### H3 — no per-category registry (high, M)
`src/hooks.rs:206` is `Arc<RwLock<Vec<Arc<dyn DynHook>>>>` and `:225-227` pushes everything into it. Because `src/hooks.rs:16-101` gives *every* `Hook` method a default no-op, a registered hook implements all nine callbacks whether or not the author overrode them — there is no runtime signal for `_get_enabled_hooks`. Upstream's split base classes (`hooks.py:124-218`) and typed lists (`hook_runner.py:61-71`) exist precisely to make this discriminable.

Prefer option (b) — a `fn declares(&self) -> HookKinds` bitflags with an **empty** default — over splitting `Hook` into marker traits, which would break every example in `docs/hooks.md`. Defaulting to "all kinds" is not acceptable: it would over-declare `enabled_hooks` and make the harness round-trip hooks that do nothing.

#### H4 — inverted `on_tool_error` (high, S)
`src/local.rs:1160-1165`: `if res.allow { result.result = val; result.error = None; }` — the failure is erased and the hook's arbitrary `Value` is sent to the model as the tool's genuine output (`:1199-1216`), with the step downgraded to `StepStatus::Done` (`:1170-1173`). Upstream: *"The hook cannot fix or retry the tool call on its own"* (`hooks.py:178-194`); only a non-empty stripped `str` is honoured, producing `OnToolErrorResult{custom_error_message}` — the message's **only** field — and the tool still failed (`hook_router.py:279-290`).

This is the one place the port faithfully copied 0.1.1 (`local_connection.py:1207-1219` also substituted a recovery result) and 0.1.6+ narrowed the contract. `docs/hooks.md:516-559` actively teaches the wrong thing.

Change `Hook::on_tool_error` to return `Result<Option<String>, _>`; short-circuit on the first `Some(non-empty)`; replace `result.error` with the string while keeping `result.error.is_some()` true.

#### H5 — `HookContext` is dead (high, L)
`grep -rn HookContext src/` matches only `src/context.rs` (definition + its own tests) and a doc comment at `src/tool_context.rs:5`. No `Hook`/`DynHook` method takes a context; `HookRunner` has no `session_context`. Upstream threads it everywhere: `hooks.py:33-60`, every signature `run(self, context: HookContext, data: T)` (`hooks.py:70-115`), `hook_runner.py:73` (session), `:177` (fresh `TurnContext` per turn), `:213/:274/:297` (per-operation), and `hook_router.py:117-120,148-149,166-168,208-210,248-251,271-274` (turn context carried across the turn, cleared on POST_TURN) with `event_processor.py:395-399` as fallback. The scoping contract is pinned by `hook_runner_test.py:166-183`: a child reads parent keys; a parent cannot read child keys.

This is the mechanism by which a `pre_tool_call` and its `post_tool_call` share state. We have no substitute.

#### H7 / S2 — gating is both narrow and open (high)
Two independent defects at the same call site.

*Narrow:* the only write to `pending_builtin_tool_calls` is `src/local.rs:1016`, inside `if is_tool_conf_new` (`:998`) and only when `allow` is true. Since `PolicyEnforcer` **is** a `Hook::pre_tool_call` (`src/policy.rs:373`), any built-in the harness executes without raising a confirmation bypasses every registered policy — including the `workspace_only` DENY rules, which target read-only tools that are least likely to be confirmed. Upstream 0.1.9 decouples the two entirely: `handle_tool_confirmation_request` *"Auto-accepts unconditionally. Pre-tool gating is handled by FirePreToolHook → CallHookRequest → HookRouter._handle_pre_tool"* (`event_processor.py:666-677`), asserted by `local_connection_test.py:2905-2955` (accepted == True even with a DenyAll hook).

Honest scoping: upstream's own test at `local_connection_test.py:2906-2911` says *"The legacy ToolConfirmation path only fires when no hooks are registered."* Since we declare no `enabled_hooks`, a 0.1.9 harness takes exactly that legacy path — so built-ins are not currently unpoliced. What is genuinely lost is MCP/search_web/read_url_content coverage (they never reach the extractor at all), `PreToolResult.reason`, argument rewriting, and all non-pre-tool lifecycle events.

*Open:* `src/local.rs:1103` is `.map_or(true, |res| res.allow)` and `:1006-1012` initialises `let mut allow = true;` overwritten only `if let Ok(res)`. `src/hooks.rs:257` uses `?`, so one erroring hook aborts the chain — and if that hook is registered before the `PolicyEnforcer`, the entire policy system is skipped for that call. The `Err` path is reachable: `src/policy.rs:457-460` returns `Err` for an ASK_USER policy with no handler, and that validation is bypassed at startup (S3). Upstream never converts a failure into ALLOW (`policy.py:712-730,745-758`; `hook_router.py:309-311`).

Fix the open half by making failure unrepresentable: `dispatch_pre_tool_call` catches per-hook `Err` and returns `HookResult{allow: false, message: format!("Internal policy error: {e}")}`, so all four call sites become correct by construction and the reason survives into the denial.

### 4.4 Policy and safety

The 9-bucket priority model, `_matches_target`, first-match-wins short-circuiting and the MCP builders are all structurally correct against upstream. Everything around them has drifted.

#### S1 — workspace escape (breaking, M)
Upstream `_is_path_in_workspace` runs `_secure_normalize_path` = `pathlib.Path(path).resolve()` on **both** target and workspace (`policy.py:441-449,487-489`), collapsing `..` and resolving symlinks, fails closed on `OSError` (`:490-492`), then compares `pathlib` **parts** component-wise with casefolding (`:494-506`). `policy_test.py:844-855,866-891` pin both.

Ours (verified above) builds `Path::new(path_str)` and calls `target_path.starts_with(ws_path)` on the raw wire string. Compiled and run:

- `/allowed/workspace/../../etc/passwd`.starts_with(`/allowed/workspace`) → **true** (inside)
- `/allowed/workspace/./../secret` → **true** (inside)
- `/tmp/workspace-evil/x` vs `/tmp/workspace` → false (Rust's component-wise `starts_with` does handle the prefix attack)

No symlink resolution exists anywhere in `src/policy.rs`, so a symlink inside the workspace pointing at `/etc` is treated as inside. This is the only mechanism enforcing the workspace sandbox and it is bypassable with a literal `..` in a model-controlled `file_path`.

Fix: `secure_normalize_path` mirroring `resolve(strict=False)` — canonicalize the longest existing ancestor, re-append the non-existent tail with `..`/`.` popped (new files must still normalize) — plus `is_path_in_workspace` that normalizes both sides, returns `false` on any `io::Error`, and compares `Components` element-wise. Preserve the existing passing cases in `src/policy.rs:861-875` and the `workspace-evil` rejection.

#### S3 — `enforce()` is never called (high, XS)
Verified: `src/agent.rs:293` is `PolicyEnforcer::new(final_policies, Vec::new())`, wrapped in `if !final_policies.is_empty()`. `policy::enforce()` (`src/policy.rs:222-254`) implements both upstream guards and derives `server_names`, and `grep` shows it is called **only** from `policy.rs`'s own `#[cfg(test)]` block. Upstream: `agent.py:105-110`.

Consequences: (a) with an empty `server_names`, `parse_mcp_tool` (`:329-338`) always returns `None`, so `is_mcp` is always false and every `server/tool` policy is inert — though this is unobservable until W6 lands; (b) the ASK_USER-handler validation is downgraded from a config-time error to a runtime `Err` that then fails **open** via S2. Fix: `policy::enforce(final_policies, Some(&self.config.mcp_servers))?`.

#### S4 — `allow_all()` opts out of the sandbox (high, XS)
`src/agent.rs:256-260` scans for a wildcard `Approve` policy named `"allow_all"` — exactly what `policy::allow_all()` (`src/policy.rs:108-116`) produces — and `:262` gates the entire workspace block on `!has_allow_all`. Upstream's `_apply_workspace_policies` (`local_connection_config.py:114-133`) is an unconditional model-validator with no such exemption, and its docstring explicitly states that `policies=[policy.allow_all()]` is the sanctioned way to get shell access **and** that file tools remain workspace-restricted. The bucket ordering makes this well-defined on both sides (specific DENY = bucket 0, global APPROVE = bucket 8). Since `allow_all()` is what `src/lib.rs:29-31`, `README.md` and every example recommend, the default Rust posture is strictly less sandboxed than upstream.

Fix: delete the special case; always prepend; filter out any incoming policy named `workspace_only` first (upstream `:116-118`) so re-application is idempotent. If an opt-out is wanted, make it explicit (`workspaces(vec![])`), not an inferred side effect of a policy name.

#### S5 — no wire-path normalization (high, M)
`WIRE_PATH_ARGUMENT_KEYS = {"path","file_path","directory_path","TargetFile"}` and `normalize_wire_path` (`local_connection_config.py:61-84`) turn `file:///a/b` into a native path via `url2pathname` and `cns://cell/rest` into `/cns/cell/rest`. Applied in three places: to tool args in place + `canonical_path` derivation (`hook_router.py:63-74,191-200` and `event_processor.py:255-271`), and to configured workspaces before they become policy roots (`:124-127`). Pinned by `event_processor_test.py:33-52` and `hook_router_test.py:900-951`.

Rust has none of it — grep for `file://`/`cns`/`normalize` across `src/` returns nothing. `extract_builtin_tool_call` assigns raw proto strings (`src/local.rs:1319,1347,1358,1370,1381,1394`; duplicated at `src/wasm.rs:1189-1260`), and `src/agent.rs:263-278` pushes raw workspace strings into `workspace_only`. Concrete effect: for `file:///tmp/ws/foo.py`, `Path::is_absolute()` is false so `src/policy.rs:181-183` returns "outside" and **every file tool is denied**. Fail-closed here, but it breaks the SDK on any harness that emits URIs, and a differently-shaped URI could mis-bucket the other way.

**This is a regression, not drift:** 0.1.1 already had `normalize_wire_path` (`local_connection.py:215-230`) and applied it at `:262-280,:1076-1088`. The port dropped it.

#### S8 — tool identifier casing (medium, M)
Upstream's public tool identifiers are the lowercase wire names (`run_command`, `view_file`, `list_directory`), mapped from proto field names by `PROTO_FIELD_TO_SDK_NAME` before every hook dispatch (`hook_router.py:184`, pinned by `hook_router_test.py:882-896`: `invoke_subagent` → `start_subagent`). We invented UPPERCASE (`src/types.rs:196-208`, `src/local.rs:1304-1401`, `src/policy.rs:141,153`).

Adversarially: Rust is **internally consistent** — the string the extractor produces is the string `confirm_run_command` targets, so `policy::deny("RUN_COMMAND")` works end to end, and there is no wire impact (tool names are never serialised; `CapabilitiesConfig` maps to `HarnessSideTools` booleans). What genuinely breaks is (a) anyone porting a Python policy verbatim, and (b) `docs/hooks.md:305,357,375`, which ship lowercase examples that resolve to ALLOW under the trailing wildcard.

The forward-looking argument is decisive regardless: once `PreToolArgs` arrives, tool names come off the wire as proto field names and must be mapped. Land the rename together with `src/policy.rs:166-173`, every doc, and `examples/policies.rs` in one breaking change, and keep a `from_wire_name()`.

### 4.5 Tools

#### T1 — `ToolContext` never wired (high, S)
`grep -rn "ToolContext|call_with_context|needs_context|set_context" --include=*.rs .` returns hits only in `src/tools.rs` (trait signatures) and `src/tool_context.rs`. `ToolRunner` (`src/tools.rs:112-115`) holds only the tool map — no context field, no `set_context`. `process_tool_calls` (`:147-150`), the sole dispatcher (called from `src/local.rs:1151` and `src/wasm.rs:790`), unconditionally calls `tool.call(...)`. `Agent::start` (`src/agent.rs:297-300`) only registers. Upstream wires it at `agent.py:136-140` after the conversation exists.

The fix is straightforward — `ToolRunner` derives `Clone` over `Arc` fields, so a context set after `Conversation::new` is visible to the clone handed to `LocalConnectionStrategy` at `src/agent.rs:362`, and `Conversation::connection()` (`src/conversation.rs:72`) already returns the `AnyConnection` that `ToolContext::new` (`src/tool_context.rs:33`) takes. The reason this went unnoticed is X19: no example or test uses it, and `examples/custom_tools.rs:53-55` hand-rolls an `Arc<Mutex<HashMap<..>>>` to do exactly the job `ToolContext` exists for.

#### T3 — the invented web-search fallback (medium, S)
`src/tools.rs:168` intercepts `call.name == "google_search" || call.name == "web_search"` *ahead of* the unknown-tool arm and calls `builtin_web_search` (`:233-293`), which spawns `python3 -c <inline scraper>` fetching `https://html.duckduckgo.com/html/?q=...` and regex-parsing the HTML. Upstream's only behaviour for an unregistered tool is `ToolResult(name=tc.name, error=f"Unknown tool: '{tc.name}'")` (`tool_runner.py:361-364`).

This is not "stale upstream behaviour" — it was **never** upstream, at any version. It is an undeclared network egress path and a hidden `python3` runtime dependency in what `README.md` Option A explicitly sells as a pure-Rust install, and it shadows the real harness-side `search_web` tool. `README.md:418` and `docs/tools.md:226-231` document it as a feature.

#### T2 / W17 — subagents (high, L)
`grep -rn -i subagent src/` yields only `BuiltinTools::StartSubagent`, the boolean `SubagentsConfig{enabled}` (`src/local.rs:632-634`), and internal id tracking. Upstream `SubagentConfig` carries `name, description, system_instructions, capabilities, tools` (`types.py:804-834`) with `SubagentCapabilities` and a mutually-exclusive validator (`:785-802`); `_build_custom_subagents_protos` (`local_connection.py:884-936`) defaults capabilities to `read_only()`, warns and drops `START_SUBAGENT` for nested subagents, and **raises** when a subagent tool is not registered on the main agent. Port all three validations. Also note upstream's own toggle is `getattr(cfg, 'enable_subagents', True) and START_SUBAGENT in active_tools` (`:840-845`) where we gate on `active_tools` alone.

### 4.6 Agent / Conversation / Triggers

#### A2 — per-turn vs cumulative usage (high, XS)
`src/conversation.rs:307` is `usage_metadata: self.total_usage().await`, and `total_usage` returns `state.cumulative_usage` accumulated across the whole session (`:116-118,172-181`, reset only by `clear_history`). Upstream's `ChatResponse.usage_metadata` returns `_last_turn_usage` (`types.py:1024-1027`), which `send()` resets every turn (`conversation.py:133`). `conversation_test.py:902-924` asserts per-turn sums; a sibling test asserts `None` when no step reported usage. We already track `turn_usage` correctly (`:182-189`, exposed at `:121-123`) — this is a one-line swap, plus `Option<UsageMetadata>` to match upstream's `| None`.

#### A3 — workspaces never reach the harness (high, XS)
`src/agent.rs:263-268` computes a cwd fallback, but that binding is consumed only by the workspace-policy block (`:277-281`) and dropped. What is handed to the connection is `self.config.workspaces.clone().unwrap_or_default()` (`:360`, and `:315` on wasm) — an **empty** Vec by default. `src/local.rs:588-595` then emits an empty `HarnessConfig.workspaces`. Upstream's field is `default_factory=lambda: [os.getcwd()]` (`local_connection_config.py:107`) and is passed straight through (`:323`). So the harness has no declared workspace root while our policy layer behaves as if cwd were one — the two halves disagree.

#### A1 / A10 — triggers (high / medium)
`TriggerRunner::start` (`src/triggers.rs:64-74`) calls `crate::spawn_task(...)` per trigger and discards the handle; the struct stores only `triggers: Vec<Arc<dyn DynTrigger>>`. There is no `stop`, no `is_running`, no already-started guard (upstream raises `RuntimeError` at `trigger_runner.py:61-62`). `Agent<Started>::stop()` (`src/agent.rs:422-425`) never touches `self.state.trigger_runner`, which is assigned at `:333/:380` and never read. Upstream enters the runner on the `AsyncExitStack` (`agent.py:129-134`) so teardown cancels it.

The proposed `Vec<JoinHandle>` will not compile: `crate::spawn_task` (`src/lib.rs:84-96`) returns `()` and has no wasm equivalent (`any_spawner::Executor::spawn_local`). Use a `tokio_util::sync::CancellationToken` / `watch` channel that the trigger future selects on — the only approach that works uniformly across both cfgs.

Separately, `Trigger::run` takes the raw `AnyConnection` (`src/triggers.rs:11-32`), so a trigger can call `send_halt_request`, `send_tool_response`, `send_question_response`, `send_tool_confirmation` and `disconnect`. `src/trigger_helpers.rs:23-28` shows only `send_trigger_notification` is needed. Provenance correction: 0.1.1 **already** wrapped the connection in a `TriggerContext` whose sole public method is `send()` (`0.1.1/triggers/triggers.py:28-50`); 0.1.5 only re-typed the constructor to a `TriggerConnection` Protocol. This is an original port omission, not upstream drift.

#### A4 — the interactive loop (high, L)
`src/interactive.rs` is 90 lines: read stdin, `agent.chat(...)`, print. `grep -n 'impl Hook' src/interactive.rs` returns nothing. Upstream's `run_interactive_loop(config, agent_class)` (`utils/interactive.py:344-374`) takes the **config**, appends an `AskQuestionHook` if absent (`:361-363`), runs `_upgrade_policies_list` (`:320-341`, rewriting `RUN_COMMAND` + DENY + `when is None` into `ask_user(..., handler=ask_user_handler, name=p.name or "interactive_confirm")`), then constructs and enters the agent, and drives `receive_steps()` inside a `Spinner` (`:376-411`, added 0.1.2).

Four consequences: no interaction hook means `src/local.rs:959-962` answers every question `unanswered`; the default `confirm_run_command(None)` hard-DENY on `RUN_COMMAND` is never upgraded, so shell commands cannot be approved interactively; no per-step feedback (we call `chat()`, which drains); no `agent_class` extension point. The signature must change to take the config (or a builder) — an already-started agent structurally cannot do any of this.

### 4.7 WASM, tooling, docs

#### X1 — the fork (breaking, L)
`src/wasm.rs` is a hand-copied fork of `src/local.rs` frozen at 0.1.1: ~150 duplicated lines of harness-config construction (`src/local.rs:516-676` vs `src/wasm.rs:150-311`), a duplicated `extract_builtin_tool_call` (`:1165-1272`), a duplicated reader loop and duplicated `ToolResponse` sites. Every wire finding in §4.1 therefore has two fix sites, and the first-pass audit flagged only the `local.rs` one.

Extract a target-agnostic `build_harness_config` and a shared step/tool-call mapper. **Placement constraint:** `src/lib.rs:68-71` gates `pub mod local` on `cfg(not(target_arch = "wasm32"))` and `pub mod wasm` on `cfg(any(target_arch = "wasm32", test))`, so the shared code must live in a new neutral module (e.g. `src/harness_config.rs`), not in either file.

#### X3 — CI cannot catch any of this (high, M)
`.github/workflows/ci.yml` runs fmt, clippy and `cargo test --all-targets --all-features` on ubuntu-latest. (a) `--all-targets` excludes doctests, and `grep -rn include_str src/` is empty, so `README.md` and `docs/*.md` are never compiled. (b) No wasm target step, despite `skills/.../SKILL.md:29-31` prescribing `cargo check --target wasm32-wasip1`; `src/wasm.rs` only ever builds as a host test. (c) Root `Cargo.toml` has no `[workspace]` and `examples/leptos_axum` / `examples/leptos_ssr_axum` declare their own, so none of the three directory examples is ever compiled — an API change like `LocalConnectionStrategy::new`'s signature is not caught. (d) `--all-features` is a no-op (no `[features]`). (e) `publish.yml` runs the same weak gate before `cargo publish`. (f) `python3-websockets` is installed and nothing uses it.

The single highest-leverage addition is a scheduled **upstream-drift job**: fetch `https://pypi.org/pypi/google-antigravity/json`, compare `info.version` against a committed `UPSTREAM_VERSION` and the pin in `scripts/install_harness.sh`, download the wheel, decode the serialized descriptor out of `localharness_pb2.py`, and diff the message/field/enum-value set against `proto/localharness.proto`. That check would have caught `STATE_IDLE`→`STATE_FULLY_IDLE`, the field-2 removal and the `STATE_TERMINAL_ERROR` deletion mechanically.

#### X5 — the version pin (medium, XS — but read the ordering note)
`scripts/install_harness.sh:7` is `VERSION="0.1.1"`; `README.md:28-38` offers that script (Option A) and `pip install google-antigravity` (Option B, now 0.1.9). `src/agent.rs:619-666` happily picks up either from `./bin`, `PATH`, or python3 site-packages.

**Do not bump the pin first.** The 0.1.1 pin is currently the only reason the SDK works at all; bumping it in isolation trips every breaking finding above. Correct sequence: add the README incompatibility warning **now**; bump `VERSION` in the same commit as the proto regeneration; add a `HARNESS_VERSION` const plus a startup check so a mismatch fails loudly instead of hanging.

### 4.8 Types and model configuration

This subsystem was audited separately (the first pass over it died mid-run). 56 findings, 7 breaking. It is the detail WP-4 is implemented from, so the wire shapes are given exactly.

#### The 0.1.9 type graph

`models.py`, re-exported through `types.py:32-38` and `__init__.py`:

```
DEFAULT_MODEL                  = "gemini-3.6-flash"              models.py:35   (was 3.5-flash through 0.1.7)
DEFAULT_IMAGE_GENERATION_MODEL = "gemini-3.1-flash-lite-image"   models.py:36   (was -flash-image-preview through 0.1.6)
ThinkingLevel   = minimal | low | medium | high | extra_high     models.py:44-63  (extra_high added 0.1.7)
ModelType       = text | image                                   models.py:66-70
ModelEndpoint   { base_url, http_headers, validate_endpoint() }  models.py:73-82   (abstract)
  GeminiAPIEndpoint { api_key, options: GeminiModelOptions }     models.py:91-106
  VertexEndpoint    { project, location, options }               models.py:109-128
GeminiModelOptions { thinking_level }                            models.py:85-88
ModelTarget     { name: str|None, types=[TEXT], endpoint|None }  models.py:131-138
RetryConfig     { api_retry, model_output_retry } + .benchmark() types.py:355-417
```

`LocalAgentConfig` keeps the shorthand fields `model`, `models`, `api_key`, `vertex`, `project`, `location` (`local_connection_config.py:169-174`) and normalises them into `models` in a post-init validator (`:297-301`).

#### The exact wire (`HarnessConfig.models`, field 15, protojson camelCase)

| Case | Emitted |
|---|---|
| no config at all | `[{"name":"gemini-3.6-flash","types":["MODEL_TYPE_TEXT"],"geminiApiEndpoint":{}}, {"name":"gemini-3.1-flash-lite-image","types":["MODEL_TYPE_IMAGE"],"geminiApiEndpoint":{}}]` — two entries, endpoint present but **empty** (`local_connection_test.py:3523-3532`) |
| `api_key="k"`, `model="m"` | the same two entries, `"geminiApiEndpoint":{"apiKey":"k"}` on **both** (`:1128-1144`) |
| `vertex=True, project, location` | `"vertexEndpoint":{"project":"p","location":"l"}` on both (`:1146-1161`) |
| separate image model | explicit `ModelTarget(types=[IMAGE])`; the default **text** model is then auto-appended *after* it (`:3544-3554`) |
| thinking level | `"geminiApiEndpoint":{"options":{"thinkingLevel":"high"}}`; `options` is omitted entirely when every field is `None` (`local_connection.py:140-146`) |
| OpenAI / Ollama-compatible | `gemmaEndpoint:{baseUrl}` — emitted by `LocalOpenAIConnectionStrategy`, not by `build_models_proto` (`local_openai_connection.py:44-57`) |
| bare strategy, no models | `models` **empty** — a valid state; the backend then chooses everything (`:1118-1126`, `local_connection.py:1060-1061`) |

Two easy traps: `VertexEndpoint.options` is field **5**, not 4; and `ModelTarget.name` is `str | None`, serialised as `""` — an empty name is meaningful, not an error (`:1213-1224`).

#### The merge algorithm (`_merge_models_list`, `local_connection_config.py:268-296`)

Concatenate **explicit `models`** → **shorthand `model`** → **defaults**, in that order. Then take the union of `ModelType`s already present and append each default model *only* if none of its types is already covered. Dedupe is by **ModelType, never by name** — two TEXT models are legal. The shorthand endpoint (`_build_shorthand_endpoint`: `VertexEndpoint` if `vertex` else always `GeminiAPIEndpoint(api_key)`) attaches to the shorthand model and to the defaults, but **never** to explicitly-supplied `models` entries — an explicit entry with `endpoint=None` raises (`local_connection.py:1060-1073`).

#### Environment variables

Upstream reads more than we do, and reads them in different places: `GEMINI_API_KEY` only inside `GeminiAPIEndpoint.validate_endpoint` as a **presence check** (`models.py:101`) — it is never copied onto the wire, so an env-only setup emits `"geminiApiEndpoint": {}` and the harness picks the key up from the inherited process environment; `GOOGLE_CLOUD_PROJECT` / `GOOGLE_CLOUD_LOCATION` hydrate a `VertexEndpoint` (`models.py:116-124`, 0.1.7); `GOOGLE_GENAI_USE_VERTEXAI` / `GOOGLE_GENAI_USE_ENTERPRISE` (`"true"`/`"1"`) flip `vertex` (`local_connection_config.py:206-211`, 0.1.7). Rust reads only `GEMINI_API_KEY` (`src/local.rs:396`) plus a non-upstream `ANTIGRAVITY_API_KEY` (`src/wasm.rs:100`), and copies the resolved key into the proto (`:415,:566`).

#### Two defaults that are less safe than upstream

- **CT-009 (high).** `CapabilitiesConfig` derives `Default`, which `src/agent.rs:205-237` and `src/local.rs:609-624` both read as *all ten* built-in tools — including `RunCommand`, `EditFile` and `CreateFile`. Upstream's default is `CapabilitiesConfig(enabled_tools=BuiltinTools.read_only())` (`connection.py:52-56` in 0.1.9, **and identically at 0.1.1** `connection.py:43-47`). So `Agent::builder().allow_all().build()` — the crate-level doc example at `src/lib.rs:29-32` — starts with unrestricted write and shell tools where the Python equivalent starts read-only. This is an original port divergence, not drift, and it compounds S1 and S4.
- **CT-032 (medium).** `HookResult` derives `Default`, and Rust's derived `bool` default is `false`, so `HookResult::default()` — and every `HookResult { message, ..Default::default() }` — **denies**. Upstream's default is `allow=True` (`types.py:715-726`, same at 0.1.1). The crate's own test at `src/types.rs:1087-1091` documents the wrong behaviour as if it were intended.

#### `read_only()` — what is and is not broken

`BuiltinTools::read_only()` (`src/types.rs:213-221`) returns four tools; upstream's has included `FINISH` since 0.1.1 and gained `READ_URL_CONTENT` in 0.1.6. The stronger reading — that `AgentBuilder::read_only()` therefore denies the `finish` tool and the agent cannot terminate — does **not** hold on the current code: `finish` never becomes a `ToolCall` (`src/local.rs` maps it to `StepType::Finish` at `:789` and never routes it through `extract_builtin_tool_call`), so the `deny_all()` prefix built at `src/agent.rs:592-604` never sees it. What is real today is narrower: `has_write_tools` (`src/agent.rs:239-240`) is permanently true, because `Finish` is always in `active_tools` and never in `read_only()`.

But the reason `finish` is not a `ToolCall` is itself a port regression: upstream's `_BUILTIN_TOOL_PROTO_FIELDS` has mapped `FINISH: "finish"` since 0.1.1 (`local_connection.py:105-116`), so upstream *does* policy-evaluate it and we simply dropped the arm. Fix the extractor and the `read_only()` list together — fixing only the extractor would make every read-only agent unable to terminate. Both are planned in `docs/fix-plan-current-defects.md` (WI-5, WI-40).

#### Findings

**Breaking**

| ID | What | Where in Rust | Effort |
|---|---|---|---|
| CT-011 | McpServerConfig never reaches the wire — builder accepts servers, LocalConnectionStrategy stores them, nothing emits them | `src/agent.rs:48-49, 563-572 (mcp_server/mcp_servers builders); src/agent.rs:365; src/loc…` | L |
| MC-01 | ModelTarget / ModelEndpoint / GeminiModelOptions type graph is entirely absent from src/types.rs | `src/types.rs:41-116 (ModelEntry/ModelConfig/GeminiConfig) — no ModelTarget, no endpoint …` | L |
| MC-02 | ModelType (TEXT/IMAGE) discriminator missing — Rust cannot declare an image model on the wire | `src/types.rs:62-95 — ModelConfig encodes purpose structurally as two named slots (`defau…` | S |
| MC-03 | src/types.rs GeminiConfig is a 0.1.1 type whose backing proto message no longer exists | `src/types.rs:98-122 (pub struct GeminiConfig); src/agent.rs:23 + 467-470; src/local.rs:3…` | L |
| MC-04 | ModelEntry / ModelConfig{default,image_generation} / GenerationConfig were deleted upstream in 0.1.4; Rust still models the world this way | `src/types.rs:33-36 (GenerationConfig), :41-60 (ModelEntry + Default), :64-95 (ModelConfi…` | M |
| MC-08 | The explicit/shorthand/default model merge algorithm does not exist in Rust — only one model is ever configured and the image model is never sent as a model | `src/local.rs:565-586 and src/wasm.rs:201-222 build exactly one config object from `gemin…` | M |
| MC-09 | CapabilitiesConfig.image_model writes GenerateImageToolConfig.model_name, a field reserved upstream since 0.1.4 | `src/types.rs:239-241 (`pub image_model: Option<String>`); emitted at src/local.rs:655-65…` | S |

**High**

| ID | What | Where in Rust | Effort |
|---|---|---|---|
| CT-001 | Content / ContentPrimitive / Media are dead types: multimodal prompts cannot be sent at all | `src/types.rs:841-905 (ContentPrimitive, Content, Content::from_file); src/agent.rs:403 (…` | L |
| CT-002 | SlashCommand / BuiltinSlashCommandName (0.1.2) missing entirely — a Rust user cannot send /plan | `src/types.rs:843-861 (ContentPrimitive has only Text and Media); proto/localharness.prot…` | M |
| CT-003 | BuiltinTools emits SCREAMING_SNAKE identifiers where upstream uses lowercase snake_case tool names | `src/types.rs:163-211 (#[serde(rename = "CREATE_FILE")] ... and as_str() returning "CREAT…` | M |
| CT-004 | BuiltinTools enum missing ASK_QUESTION, SEARCH_WEB (0.1.4) and READ_URL_CONTENT (0.1.6) | `src/types.rs:163-194 (10 variants only); src/local.rs:597-608 and src/agent.rs:208-233 a…` | M |
| CT-007 | CapabilitiesConfig.image_model retained and emitted on GenerateImageToolConfig.model_name, a field the harness reserved in 0.1.4 | `src/types.rs:239-241 (pub image_model: Option<String>); src/local.rs:655-658 and src/was…` | S |
| CT-009 | Default capabilities are all-tools-enabled; upstream defaults an agent to read-only tools | `src/types.rs:225-232 (#[derive(Default)] with enabled_tools/disabled_tools = None); src/…` | S |
| CT-013 | SubagentConfig / SubagentCapabilities (0.1.5) absent — static subagents cannot be declared | `src/types.rs (no such types); src/agent.rs:18-50 (AgentConfig has no subagents field); p…` | L |
| MC-05 | DEFAULT_MODEL is two releases stale: gemini-3.5-flash vs gemini-3.6-flash | `src/types.rs:12 `pub const DEFAULT_MODEL: &str = "gemini-3.5-flash";` (used at src/types…` | XS |
| MC-07 | ThinkingLevel is missing the EXTRA_HIGH variant added in 0.1.7, and `rename_all = "lowercase"` cannot produce "extra_high" | `src/types.rs:18-29 (`#[serde(rename_all = "lowercase")] pub enum ThinkingLevel { Minimal…` | XS |
| MC-11 | Vertex validation is wrong: Rust accepts an API-key-only Vertex config, upstream requires both project AND location | `src/local.rs:404-413` | S |
| MC-12 | GOOGLE_CLOUD_PROJECT / GOOGLE_CLOUD_LOCATION env vars are never read | `src/local.rs:388-413 reads only GEMINI_API_KEY (:396); src/wasm.rs:93-106 reads ANTIGRAV…` | XS |
| MC-15 | No base_url / http_headers support — custom, proxied and OpenAI-compatible endpoints are impossible | `src/local.rs:567 `base_url: None,` and src/wasm.rs:203 `base_url: None,` — hardcoded; sr…` | M |
| MC-17 | AgentBuilder exposes the removed model surface (.gemini_config/.api_key/.default_model) and none of the current one | `src/agent.rs:467-479 — only `gemini_config(GeminiConfig)`, `api_key(impl Into<String>)`,…` | M |

**Medium**

| ID | What | Where in Rust | Effort |
|---|---|---|---|
| CT-005 | BuiltinTools::read_only() diverges from upstream: FINISH (present since 0.1.1) and READ_URL_CONTENT (0.1.6) missing | `src/types.rs:213-221 (read_only() -> vec![FindFile, ListDir, ViewFile, SearchDir]); src/…` | XS |
| CT-006 | BuiltinTools missing the nondestructive() / all_tools() / file_tools() / none() constructors | `src/types.rs:196-222 (only as_str() and read_only())` | S |
| CT-008 | CapabilitiesConfig missing enable_subagents | `src/types.rs:225-242 (CapabilitiesConfig has enabled_tools, disabled_tools, compaction_t…` | XS |
| CT-010 | McpServerConfig::Sse variant still exists and is documented, but upstream deleted McpSseServer in 0.1.2 | `src/types.rs:264-280 (McpServerConfig::Sse { name, url, headers, enabled_tools, disabled…` | XS |
| CT-012 | McpServerConfig missing stdio env (0.1.4), timeout_seconds (0.1.3), auth_provider_type, and the name regex validation | `src/types.rs:245-316` | M |
| CT-014 | Media has no per-category MIME validation: Image, Document, Audio and Video are the same type | `src/types.rs:817-838 (struct Media + `pub type Image = Media; pub type Document = Media;…` | M |
| CT-015 | Four MIME type strings are wrong and the 0.1.9 audio additions are missing | `src/types.rs:672 (Javascript => "application/javascript"), 681 (Xml => "application/xml"…` | S |
| CT-018 | StepType::THINKING (0.1.5) missing, and local.rs never classifies a thinking-only step | `src/types.rs:394-414 (StepType has TextResponse, ToolCall, SystemMessage, Compaction, Fi…` | XS |
| CT-019 | StepStatus::TerminalError and STATE_TERMINAL_ERROR=5 are dead — removed upstream in 0.1.3 | `src/types.rs:468-470 (StepStatus::TerminalError); src/local.rs:824 (Some(5) => StepStatu…` | S |
| CT-020 | UsageMetadata uses non-optional i32 counters, conflating "not reported" with an explicit zero | `src/types.rs:377-391 (prompt_token_count: i32, candidates_token_count: i32, total_token_…` | S |
| CT-023 | ToolCall and ToolResult missing server_name (0.1.6); ToolResult missing exception | `src/types.rs:347-358 (ToolCall: id, name, args, canonical_path); src/types.rs:361-374 (T…` | M |
| CT-024 | ToolCall.id and ToolCall.args are required in Rust but optional/defaulted upstream | `src/types.rs:347-358 (pub id: String; pub args: Value — neither Option nor #[serde(defau…` | M |
| CT-025 | ToolExecutionError (0.1.9) missing from the error surface | `src/error.rs:17-34 (AntigravityError has only Connection, Execution, Validation); src/ty…` | S |
| CT-026 | AntigravityCancelledError (0.1.2) and ChatResponse::cancel() (0.1.2) missing | `src/error.rs:17-34; src/types.rs:600-610 (ChatResponse is a plain struct); src/connectio…` | M |
| CT-027 | ChatResponse.usage_metadata reports cumulative session usage, not the turn's usage | `src/types.rs:600-610 (ChatResponse.usage_metadata: UsageMetadata); src/conversation.rs:3…` | XS |
| CT-028 | SessionContinuationMode (0.1.7) missing from types and from HarnessConfig | `src/types.rs (absent); src/agent.rs:42-43 (only conversation_id); proto/localharness.pro…` | M |
| CT-029 | RetryConfig / ModelAPIRetryConfig / ModelOutputRetryConfig (0.1.9) missing | `src/types.rs (absent); src/agent.rs:18-50; proto/localharness.proto:26-41` | M |
| CT-030 | DEFAULT_MODEL and DEFAULT_IMAGE_GENERATION_MODEL are stale by two releases | `src/types.rs:12 (DEFAULT_MODEL = "gemini-3.5-flash"); src/types.rs:15 (DEFAULT_IMAGE_GEN…` | XS |
| CT-031 | ThinkingLevel missing EXTRA_HIGH (0.1.7), and its serde rename_all would mis-encode it | `src/types.rs:18-29 (#[serde(rename_all = "lowercase")] enum ThinkingLevel { Minimal, Low…` | XS |
| CT-032 | HookResult::default() denies where upstream's default allows | `src/types.rs:546-553 (#[derive(Default)] on HookResult); src/types.rs:1087-1091 (test as…` | XS |
| MC-06 | DEFAULT_IMAGE_GENERATION_MODEL is stale: gemini-3.1-flash-image-preview vs gemini-3.1-flash-lite-image | `src/types.rs:15 `pub const DEFAULT_IMAGE_GENERATION_MODEL: &str = "gemini-3.1-flash-imag…` | XS |
| MC-10 | RetryConfig / ModelAPIRetryConfig / ModelOutputRetryConfig are completely missing from the Rust SDK | `no occurrence of RetryConfig anywhere in /home/user/antigravity-sdk-rust/src or /home/us…` | M |
| MC-13 | GOOGLE_GENAI_USE_VERTEXAI / GOOGLE_GENAI_USE_ENTERPRISE do not switch the Rust SDK to Vertex | `src/types.rs:105-106 (`#[serde(default)] pub vertex: bool`) — a plain bool with no tri-s…` | XS |
| MC-16 | GeminiConfig.enable_google_search / enable_url_context are dead — the message that carried them no longer exists and upstream Python never set them | `src/types.rs:116-121 (pub enable_google_search / enable_url_context on GeminiConfig); em…` | S |
| MC-19 | ModelTarget.name is optional upstream and the model list may legitimately be empty; Rust forces a non-empty name and always sends exactly one model | `src/types.rs:41-49 (`ModelEntry.name: String`, non-optional, defaulted to DEFAULT_MODEL …` | S |
| MC-21 | Docs, examples, skills and integration tests all teach the removed 0.1.1 model API | `docs/agent.md:113, :141, :620-665, :695; docs/connections.md:146, :164-171, :206-209, :2…` | M |

**Low**

| ID | What | Where in Rust | Effort |
|---|---|---|---|
| CT-016 | from_bytes (0.1.5) has no Rust equivalent | `src/types.rs:863-905 (Content only offers text(), media(), from_file(), as_text())` | S |
| CT-017 | Content::from_file uses a hand-rolled extension table instead of MIME inference, accepting and rejecting different file sets than upstream | `src/types.rs:878-896 (from_file), 920-956 (mime_from_extension)` | S |
| CT-021 | UsageMetadata has no addition operator (0.1.8) — accumulation is open-coded in Conversation | `src/types.rs:377-391 (no impl std::ops::Add); src/conversation.rs:172-186` | XS |
| CT-022 | Rust proto narrows UsageMetadata counters to int32 where the harness declares uint64 | `proto/localharness.proto:453-459 (optional int32 for all five fields)` | S |
| CT-033 | SystemInstructionSection.title has no default and the builder rejects a bare string | `src/types.rs:125-131 (SystemInstructionSection { content: String, title: String }); src/…` | S |
| CT-034 | CustomSystemInstructions.Part.template (SystemInstructionTemplate + Arg) absent from the Rust proto | `proto/localharness.proto:77-85 (Part oneof has only text = 1); src/types.rs:134-138 (Cus…` | XS |
| CT-035 | ToolOutputTruncation (HarnessConfig field 18) is absent from the Rust proto; upstream Python does not expose it either | `proto/localharness.proto:26-41 (HarnessConfig ends at field 12); src/types.rs (no such t…` | S |
| MC-14 | GEMINI_API_KEY is copied into the proto rather than left empty, and src/wasm.rs invents an ANTIGRAVITY_API_KEY env var upstream has never had | `src/local.rs:389-397 and :566 (`api_key: Some(api_key)`); src/wasm.rs:93-106 and :202` | S |
| MC-18 | Model-config-to-proto logic is duplicated between src/local.rs and src/wasm.rs (refactor guidance for the MC-01/MC-08 rewrite, not an independent parity gap) | `src/local.rs:565-586 vs src/wasm.rs:201-222 (identical ProtoGeminiConfig construction); …` | S |
| MC-20 | GeminiModelOptions must be omitted when every option is None — a naive Rust port would emit an empty options object | `no equivalent — src/local.rs:569-580 writes `thinking_level: Option<String>` directly on…` | XS |

MC-01…MC-04, MC-08, MC-19 and MC-20 are the substance of WP-4 and should be read as its specification. CT-011 (MCP never reaching the wire) is the same defect as W6, found independently from the types side. CT-009 and CT-032 belong in WP-3 with the other security-posture fixes — neither is blocked on the migration.

---

## 5. Work plan

Effort: S ≤ 2 days · M ≤ 1 week · L ≤ 2 weeks · XL > 2 weeks.

> **Prerequisites for everything else: WP-1 and WP-2.** No wire-level fix can be written or verified without a regenerated schema and a mock that speaks 0.1.9. WP-3 is the only substantial package with no dependency on either, and it should be started in parallel on day one because it closes two breaking security findings.

---

### WP-1 — Regenerate the proto; make deserialization tolerant and loud
**Blocked by:** none. **Size:** M. **Prerequisite for:** WP-2, WP-4…WP-10.

**Goal.** `proto/localharness.proto` describes the 0.1.9 schema, generated by script rather than by hand, and a schema surprise produces an error rather than a hang.

**Files.** `proto/localharness.proto`, `build.rs`, new `scripts/gen_proto.py`, `docs/upstream-parity.md`.

**Work.**
1. Write `scripts/gen_proto.py`: read the serialized `FileDescriptorProto` out of the wheel's `localharness_pb2.py` and render `.proto` text. Keep `syntax = "proto3"` and mark **every singular field `optional`** (W28) — prost-build 0.12 does not support editions, and dropping presence silently breaks the `.is_some()`-driven classifier and dedup.
2. Regenerate. This lands the schema half of W2, W4, W5, W6, W7, W8, W9, W10, W11, W13–W17, W20, W22, W25–W27, and removes W23's `STATE_TERMINAL_ERROR = 5`.
3. W24: do **not** vendor `content.proto`. Drop `Struct arguments = 4` from `ToolCall` and `Struct response = 4` from `ToolResponse`, and delete the local `Struct`/`Field`/`Value`/`ListValue`/`NullValue`. Neither SDK populates those fields (`event_processor.py:696` reads `arguments_json`; we set `response: None`), and with `ignore_unknown_fields` any stray object is skipped.
4. `build.rs`: add `.ignore_unknown_fields()` (W1). Emit `RUSTC_VERSION` via `cargo:rustc-env` so `language_version` stops being `"unknown"` (W20).
5. `src/local.rs:1229-1235` / `src/wasm.rs:869`: on parse failure, push `Err(...)` onto `step_tx` **and** log, so drift surfaces to the caller.

**Verify.**
- New `tests/wire_compat.rs`: feed raw 0.1.9 JSON fixtures — `initializeConversationResponse`, `trajectoryStateUpdate` with `STATE_FULLY_IDLE` and `error`, `stepUpdate.customTool`, `stepUpdate.searchWeb`, `generateImage.aspectRatio` — through `serde_json::from_str::<OutputEvent>` and assert each parses to the expected variant.
- A negative fixture with a fabricated unknown field asserts the event still parses (`ignore_unknown_fields` works) and a fabricated unknown **enum variant** asserts a surfaced `Err`, not a silent drop.
- `cargo build` fails loudly at every struct literal that must change — that is the intended forcing function for WP-4 and beyond.

---

### WP-2 — Rebuild the mock harness and the test rig
**Blocked by:** WP-1. **Size:** M. **Prerequisite for:** WP-4, WP-5, WP-6, WP-8.

**Goal.** Nothing can be verified against a mock that certifies the 0.1.1 format.

**Files.** `src/bin/mock_localharness.rs`, `tests/integration_tests.rs`, `src/wasm.rs` (in-file mock, X12), new `tests/fixtures/`.

**Work.** Closes X2, X12, and the test half of W23.
1. Reply to `InitializeConversationEvent` with `{"initializeConversationResponse":{"cascadeId":…,"history":[…]}}` — currently `mock_localharness.rs:74-77` reads and discards it, which will deadlock any client that adopts the mandatory recv.
2. `STATE_IDLE` → `STATE_FULLY_IDLE` (`:143-150`); add a prompt-triggered `STATE_CANCELLED` path and a `trajectoryStateUpdate.error` path.
3. Replace the `STATE_TERMINAL_ERROR` branch (`:100-108`) with `STATE_ERROR` + an `error` action carrying `httpCode`; update `tests/integration_tests.rs:176-233` accordingly.
4. Answer `{"sessionEndRequest":true}` with `{"sessionEndResponse":true}`; add a prompt-triggered `callHookRequest` frame so WP-8's response path is exercisable.
5. Assert `clientInfo.os`/`osVersion` and `InputConfig.env` on the handshake so WP-5's new fields are regression-tested.
6. Prefer fixtures captured from upstream's `local_connection_test.py` / `event_processor_test.py` over hand-written JSON, so the rig is anchored to upstream rather than to itself.
7. Replace `src/wasm.rs:1536-1591`'s closed Rust→Rust loop with the same fixtures.

**Verify.** The existing integration tests pass against the new mock; a deliberately-0.1.1 fixture now **fails**.

---

### WP-3 — Security hardening (no wire dependency — start immediately)
**Blocked by:** none. **Size:** M.

**Goal.** Close the two breaking security findings and the four high ones that need no proto work.

**Files.** `src/policy.rs`, `src/agent.rs`, `src/local.rs`, `src/wasm.rs`, `src/hooks.rs`, new `src/wire_path.rs`, `docs/policy.md`.

**Closes.** S1, S2, S3, S4, S5, S7, S12, S13, S14, S15, S16, S17, C20, N8, CT-005, CT-006, CT-009, CT-032.

**Work.**
1. **S1:** `secure_normalize_path` + `is_path_in_workspace` with component-wise comparison, fail-closed on `io::Error`, optional case-insensitivity probe. Rewrite the `is_outside_workspace` closure.
2. **S2:** make `dispatch_pre_tool_call` return a fail-closed `HookResult` on per-hook `Err` (no `?`), then fix all four call sites; carry `res.message` into the denial (H10).
3. **S3:** `policy::enforce(final_policies, Some(&self.config.mcp_servers))?`. Note the enclosing block already returns `Err(anyhow!(...))` at `src/agent.rs:287`, so this compiles in place.
4. **S4:** delete the `has_allow_all` gate; always prepend `workspace_only`; dedup by policy name.
5. **S5:** `src/wire_path.rs` with `WIRE_PATH_ARGUMENT_KEYS` and `normalize_wire_path`; apply to workspaces (`LocalConnectionStrategy::new` and `src/agent.rs:263-278`) and to args + `canonical_path` in **both** extractors.
6. **S7/S12:** add `BuiltinTools::file_tools()`, `nondestructive()`, `all_tools()`; add `Finish` to `read_only()`; drive `workspace_only` from `file_tools()`.
7. **S14/S16/C20/S13/S15/S17** as tabled. `docs/policy.md` covers the 9-bucket table, the identifier spelling the matcher actually accepts, `workspace_only` semantics, and the `server/tool` target form.

**Verify.** Port `policy_test.py:832-901` (path scoping incl. symlink resolution and structural containment), `event_processor_test.py:33-52` (three `normalize_wire_path` cases), and add a fail-closed test asserting an erroring hook DENIES. A regression test asserts `Agent::builder().allow_all()` still produces a `workspace_only` prefix.

---

### WP-4 — Model configuration and the shared `HarnessConfig` builder
**Blocked by:** WP-1. **Size:** L.

**Goal.** Stop emitting removed fields; unify the two divergent config builders.

**Files.** new `src/harness_config.rs`, `src/local.rs`, `src/wasm.rs`, `src/types.rs`, `src/agent.rs`.

**Closes.** W3, W9, W18, W19, X1 (config half), X7 (constants), X10, N1, N2, N5, N7, and the model-config block of §4.8: MC-01…MC-06, MC-08…MC-21, CT-029, CT-030, CT-031.

**Work.**
1. Extract `build_harness_config(&…) -> HarnessConfig` into a target-agnostic module (see the `src/lib.rs:68-71` cfg constraint) and delete the duplicate in `src/wasm.rs:150-311`.
2. Build `models` from `GeminiConfig.models` — one `ModelConfig` per target, `types=[TEXT]` or `[IMAGE]`, endpoint chosen by `vertex` — mirroring `build_models_proto`. Route `capabilities_config.image_model` here as an IMAGE-typed entry (W9).
3. Separate `app_data_dir` from `save_dir` (W18); default to `~/.gemini/antigravity` via `dirs::home_dir()`, no `/tmp` fallback, absolute-path validated; keep `save_dir` for `InputConfig.storage_directory` only.
4. Deprecate `enable_google_search`/`enable_url_context` as no-ops with a warning (W19).
5. `DEFAULT_MODEL = "gemini-3.6-flash"`, `DEFAULT_IMAGE_GENERATION_MODEL = "gemini-3.1-flash-lite-image"` (X7); delete the explicit `.default_model(...)` calls from examples/docs so they inherit the constant.
6. Migrate `tests/` off `GeminiConfig`/`GenerationConfig`/`ModelEntry` and off exhaustive `CapabilitiesConfig` literals (X10).

**Verify.** Mock asserts the init JSON contains `models` with the expected name/type/endpoint and contains **no** `geminiConfig`, `gemmaConfig` or `modelName`. Unit test for the Vertex vs Gemini-API endpoint branch.

---

### WP-5 — Turn lifecycle and idle state machine
**Blocked by:** WP-1, WP-2. **Size:** L.

**Goal.** A turn against a 0.1.9 harness starts, streams, errors, cancels and terminates correctly.

**Files.** `src/local.rs`, `src/wasm.rs`, `src/types.rs`, `src/error.rs`, `src/conversation.rs`.

**Closes.** W2 (behaviour), W7, W10 (args), C2, C3, C4, C5, C6, C7, C10, C14, C16, C17, C18, C19, W21, W23 (code), S9 (generic args conversion), plus the extraction arms for W5's four new actions.

**Work.**
1. FULLY_IDLE / CANCELLED / `tsu.error` handling per `event_processor.py:538-568`, including the subagent early-return.
2. `main_trajectory_id: Arc<Mutex<Option<String>>>` set from the first non-empty `trajectory_id`; delete `parent_idle`/`active_subagent_ids`; clear it and `step_trackers` in `send()`.
3. Restructure the `receive_steps` unfold: `StepEvent` enum instead of the magic sentinel id; terminate only on `is_idle && rx.is_empty()`; emit a sentinel on every FULLY_IDLE; initial state idle.
4. `client_cancelled: AtomicBool` + `AntigravityError::Cancelled`; `Connection::cancel()`; yield the error once at each termination point.
5. `stderr_lines: Arc<Mutex<VecDeque<String>>>` (cap 100) + `disconnecting: AtomicBool`; surface `AntigravityConnectionError` with the stderr tail on unexpected close.
6. `StepTracker::update_state` clears `handled_requests` on leaving `WAITING_FOR_USER`; move the state guard to the call site.
7. Replace the hand-written `json!` blocks in `extract_builtin_tool_call` with a generic message→`Value` conversion (S9) and add arms for `mcp_tool`, `search_web`, `read_url_content`, `custom_tool` (with the upstream de-duplication against the live `tool_call` event), plus `aspect_ratio`.
8. `is_receiving` guard; `connect_async_with_config` with `max_message_size: None`; `StepType::Thinking`; `TARGET_MODEL` → Unknown; question-index alignment + error-fallback answers.
9. Apply every change to `src/wasm.rs` — or, better, extract the shared reader logic as part of X1.

**Verify.** Mock drives: normal turn; turn ending in `trajectoryStateUpdate.error`; cancelled turn; idle → step → idle (asserting the post-idle step is yielded, per `local_connection_test.py:295-341`); a 5 MiB tool result round-trip (`:673-691`); a subagent trajectory going idle first, asserting the parent turn does **not** end; two `questions_request`s on the same step index. Assert `is_idle` is true immediately after connect.

---

### WP-6 — Handshake, session continuation, ordered shutdown
**Blocked by:** WP-1, WP-5. **Size:** M.

**Files.** `src/local.rs`, `src/wasm.rs`, `src/conversation.rs`, `src/connection.rs`, `src/agent.rs`, `src/types.rs`.

**Closes.** W4 (arm 13), W12, W14, W15, W20, C1, C8, C9, C11, C12, C13, A8, A9.

**Work.**
1. After sending the init event, `ws_read.next().await` with a timeout; parse the `InitializeConversationResponse`; map `history` through the same StepUpdate→Step mapping; store as `initial_history` on the connection; expose via `Connection::initial_history()`; seed `Conversation::new(conn, history)` replaying compaction indices and cumulative usage. On failure kill the child and surface the harness stderr, as upstream does. **Do not** take `cascade_id` — upstream ignores it.
2. `SessionContinuationMode` on `AgentConfig` + builder + strategy + field 19; reject RESUME without a `conversation_id`; validate `conversation_id` (≥32, `[a-zA-Z0-9-]`).
3. `env` on `AgentConfig`/strategy → `InputConfig.env` (field 5, on the prost **binary** handshake, so map encoding is exercised) **and** `Command::envs()` merged over the existing SHELL/PATH base.
4. `ClientInfo.os`/`os_version`; `save_dir` temp default; `DebugConfig` with `apply_logging`.
5. Ordered `disconnect()`: `disconnecting` flag → close WS (500 ms) → take and drop the retained `child_stdin` → `timeout(180s, proc.wait())` → `start_kill` + 1 s → `kill()`. Hoist `PROCESS_WAIT_TIMEOUT_SECONDS`.
6. `sanitize_prompt` (text parts only); 127.0.0.1 fallback with stderr in the connect error.

**Verify.** Mock returns a two-step history and the test asserts `conversation.history()` contains both before any prompt. A test asserts stdin is closed **before** the process is waited on and that the WS closes first (mirroring `local_connection_test.py:3108-3185`). Env round-trip assertion on the decoded `InputConfig`. Relative-path `app_data_dir` and short `conversation_id` produce config errors.

---

### WP-7 — Tool runner correctness
**Blocked by:** WP-1 (for `error_message`); otherwise independent. **Size:** M.

**Files.** `src/tools.rs`, `src/tool_context.rs`, `src/agent.rs`, `src/local.rs`, `src/wasm.rs`, `src/types.rs`, `examples/custom_tools.rs`.

**Closes.** W8, T1, T3, T4, T5, T6, T7, T8, T9, T10, T11, C15, H11, X19, N3, N6.

**Work.**
1. `ToolRunner::set_context`; branch on `needs_context()`; call it from `Agent::start` after the `Conversation` exists, on both the native and wasm branches.
2. Split every `ToolResponse` construction: `error_message` set and `response_json` unset when the tool failed (W8) — six sites.
3. Delete the `google_search`/`web_search` interception and `builtin_web_search` (T3); unknown tools fall through to `Unknown tool: '{name}'`.
4. `coerce_against_schema(parameters_json_schema, args)` applied before dispatch, retaining the original value on parse failure (T4); `arguments_json` absent → `{}` (T5).
5. `server_name` on `ToolCall`/`ToolResult`; `#[serde(skip)] exception: Option<Arc<anyhow::Error>>` on `ToolResult`; `ToolExecutionError{message, tool_name, server_name}` in `src/error.rs` passed to `on_tool_error` (H11) — prefer changing the trait parameter type over `downcast_ref`.
6. `register()` returns `Result`; `IndexMap` (or a parallel order Vec) for deterministic wire order; `join_all` for batches.
7. `_extract_media_from_result` equivalent populating `supplemental_media` (C15).
8. Rewrite `RecordFruitTool` in `examples/custom_tools.rs` to use `needs_context`/`call_with_context` + `ctx.update_state`, and add an integration test that drives `process_tool_calls` with a context set and asserts the state round-trips (X19).

**Verify.** Port `tool_runner_test.py:274-336` (coercion), `:382-397` (unknown tool), `:459-478` and `:481-518` (exception preservation, mixed batch), `:67-103` (duplicate registration). Mock asserts a failing tool produces `errorMessage` and **no** `responseJson`.

---

### WP-8 — Harness-side hook channel
**Blocked by:** WP-1, WP-3, WP-5, WP-6. **Size:** XL.

**Files.** `proto/localharness.proto` (via WP-1's generator), new `src/hook_router.rs`, new `src/state.rs`, `src/hooks.rs`, `src/context.rs`, `src/tool_context.rs`, `src/local.rs`, `src/wasm.rs`, `docs/hooks.md`.

**Closes.** H1, H2, H3, H4, H5, H6, H7, H8, H9, H12, H13, H15, H16, H17, H18, H19, W4 (arms 14/15), S6, S10.

**Work, in this order — the ordering is load-bearing:**
1. **H3 first.** `HookKinds` bitflags via `fn declares(&self) -> HookKinds` (default empty); typed accessors; `has_hooks()`; `enabled_hooks()`; reject a hook declaring nothing.
2. **Shared `StateStore`** (`src/state.rs`) with `parent`, reentrant lock, `get_state(key, default)`, `set_state`, `update_state` (parent-read → local-write, non-deadlocking), `lock()` + RAII guard. Rebuild `HookContext` and `ToolContext` on it (H13); drop `ToolContext::is_idle`/`send` (T9).
3. **H5:** add `context: &HookContext` to every `Hook`/`DynHook` method; session context on `HookRunner`; `TurnContext` in `dispatch_pre_turn`; `OperationContext` per tool call / interaction / compaction.
4. **H4/H8/H9/H12/H15/H16:** correct the contracts — `on_tool_error` returns `Option<String>` and never clears the error; `post_tool_call` fires for every completed tool *in addition to* `on_tool_error`; a failing `on_tool_error` hook yields `allow=false, "Error recovery failed: …"`; subagent completion dispatches `post_tool_call`.
5. **H2:** `src/hook_router.rs` with `handle(req)`, the 7-entry table, the always-answer guarantee, the `empty_result` fallback, and error containment. Wire the `CallHookRequest` arm into both reader loops, spawned so the reader never blocks.
6. **Only then** emit `enabled_hooks = 16` from `HookRunner::enabled_hooks()`, and reduce the `tool_confirmation_request` arm to an unconditional accept (H7). Flipping that before step 5 would remove the only working gate.
7. `session_end_request`/`session_end_response` gated on registered session-end hooks, awaited in `disconnect()` before teardown; **also** dispatch `dispatch_session_end` unconditionally so the client-side hook fires on both transports (H1).
8. `ToolCall.server_name` from `PreToolArgs`/`ActionMcpTool`; delete `parse_mcp_tool` and `server_names` (S6). Async predicates/handlers (S10).
9. Rewrite `docs/hooks.md` (H19).

**Verify.** Port `hook_router_test.py` wholesale — it is the best specification available: `:169-200` (pre-turn prompt unmarshalling), `:321-360` (post-tool with error), `:605-660` (`ToolExecutionError` with `server_name`), `:779-821` (denial reason), `:865-951` (name mapping and path normalization). Plus `hook_runner_test.py:166-183` (context scoping) and `:365-382` (error containment). Mock issues a `callHookRequest` and asserts exactly one `callHookResponse` with the matching `request_id`, including the no-router `emptyResult` case.

---

### WP-9 — Capability surface completion
**Blocked by:** WP-1, WP-4. **Size:** L.

**Files.** `src/types.rs`, `src/agent.rs`, `src/local.rs`, `src/wasm.rs`, `src/harness_config.rs`.

**Closes.** W6, W13, W16, W17, W21, W26, W27, A13, T2, T11, S8, CT-003, CT-004, CT-008, CT-010, CT-011, CT-012, CT-013.

**Work.** MCP servers on the wire (field 14) with stdio `env`, `timeout_seconds` and both transports; `BuiltinTools` gains `AskQuestion`/`SearchWeb`/`ReadUrlContent` and `all_tools()` replaces the three hardcoded lists; `search_web`/`read_url_content` configs; `user_questions.enabled` gated on `AskQuestion`; `enable_subagents`; `SubagentConfig`/`SubagentCapabilities` + `custom_subagents = 17` with all three upstream validations; `RetryConfig` (emit only when a sub-message is populated, per `local_connection.py:116-121`); `tool_output_truncation`; the lowercase tool-identifier rename with `from_wire_name()` (S8) landed together with `src/policy.rs`, docs and examples.

**Verify.** Mock asserts `mcpServers`, `searchWeb`, `readUrlContent`, `customSubagents`, `retryConfig` appear in the init JSON exactly when configured. Port `local_connection_test.py:1771-1790` (both subagent-disable paths), `:1842-1973` (retry matrix), `:3716` (MCP), and the subagent-tool-not-registered error.

---

### WP-10 — Public API surface
**Blocked by:** WP-5, WP-6, WP-8. **Size:** L. **This is the release-breaking package** — see §6.

**Files.** `src/connection.rs`, `src/conversation.rs`, `src/agent.rs`, `src/types.rs`, `src/triggers.rs`, `src/trigger_helpers.rs`, `src/interactive.rs`, `src/local.rs`, `src/wasm.rs`.

**Closes.** W11, W22, A2, A4, A5, A6, A7, A10, A11, A12, A14, A16, A17, A18, A19, C19 (surface), S11, H6 (prompt plumbing), H14.

**Work.** `Content`/`ContentPrimitive` + `SlashCommand` threaded through `send`/`chat` down to `complex_user_input`; `Connection` gains `cancel`, `wait_for_idle`, `wait_for_wakeup`, `initial_history`, `debug_config`, and moves the three tool-result methods behind a `pub(crate)` / sealed trait; `Conversation` gains `cancel`, `get_last_structured_output`, in-flight drain, per-turn `usage_metadata` and per-turn `steps`; `TriggerContext` narrowing + `TriggerRunner::stop` via a cancellation token + `every(callback)` + `on_file_change` behind a `file-watch` feature + `FileChange` re-exported from `triggers`; `run_interactive_loop(config)` with `Spinner`, `AskQuestionHook`, `ToolConfirmationHook`, `ask_user_handler` and `upgrade_policies_list`; empty-prompt validation; `UsageMetadata` → `u64`; deprecations for `register_*` and `last_turn_usage` (the latter **only after** A2 lands).

**Verify.** Port `conversation_test.py:902-924` (per-turn usage), `:971-1035` (back-to-back drain), `agent_test.py:117-135` (empty prompt), `local_connection_test.py:3333/3360/3385` (media and slash-command parts), `interactive_test.py:282-296` (policy upgrade). Mock asserts a media prompt arrives as `complexUserInput` with the right `Part` shapes.

---

### WP-11 — Tooling, CI, docs, examples, skills
**Blocked by:** partially independent; the doc rewrites follow the packages they describe. **Size:** M.

**Closes.** X3, X4, X5, X6, X7 (fan-out), X8, X9, X11, X13, X14, X15, X16, X17, X18, T12, S17, H19, A19 (docs), W25–W27 (parity notes).

**Work.** The drift-detection CI job (§4.7); `cargo test --doc`; `cargo check --target wasm32-wasip1`; a matrix step building each `examples/*/Cargo.toml`; bump `VERSION` to 0.1.9 **in the same commit as WP-1** plus a `HARNESS_VERSION` const and a startup mismatch check; add the README `protoc` prerequisite (or vendor protoc); fix `install_harness.sh` to extract via `mktemp -d`, verify `digests.sha256`, and drop the never-published `macosx_10_9_x86_64` branch; rewrite the platform table (Windows **is** supported since 0.1.4); delete every SSE mention and the `mcp_config.json` narrative; fix the `{server}_{tool}` vs `{server}/{tool}` contradiction in `docs/mcp.md:174` vs `:278`; add `docs/policy.md`; update `ARCHITECTURE.md:78-131` and `docs/connections.md:355` for the new handshake; add a `WasmConnectionStrategy::new` (or `#[non_exhaustive]`) so `docs/connections.md:204-219` stops being a semver trap; fix the `file:///Volumes/...` skill links and `plugin.json`'s license/author; settle wasip1 vs wasip2; add an `otel` feature + `src/otel.rs` (X9, or explicitly decline — see §6).

**Verify.** The drift job passes against 0.1.9 and fails against a synthetic 0.1.10 descriptor. `cargo test --doc` passes. `cargo check --target wasm32-wasip1` passes. A fresh container running only the README instructions builds and runs `hello_world`.

---

## 6. Decisions needed from the maintainer

1. **One breaking release or staged?** WP-10 changes `Connection::send`, the `Hook` trait, `BuiltinTools` string values (S8), `UsageMetadata` field types (W22), `ToolRunner::register`'s return type (T7), `Trigger::run`'s parameter (A10) and `run_interactive_loop`'s signature (A4). Staging them across 0.x releases means several breaking releases in a row; batching means one large one. **Recommendation: batch WP-10 into a single 0.2.0**, and ship WP-1…WP-9 as 0.1.x fixes that are wire-breaking but source-compatible.
2. **Model backends beyond Gemini/Vertex (N4).** Upstream ships two extra strategies as top-level exports since 0.1.6 — `LiteRTAgentConfig` (on-device) and `LocalOpenAIAgentConfig` (Ollama / LM Studio / any OpenAI-compatible endpoint) — and both build `GemmaEndpoint` directly rather than through `build_models_proto`. Three options: keep the `gemma_endpoint`/`custom_endpoint` arms in the schema but emit neither (cheap, matches today's capability); add the LocalOpenAI strategy only (moderate — it is a base-URL swap plus a different `_build_harness_config`, and it is the one users actually ask for); or port LiteRT too (XL, needs the on-device server lifecycle). **Recommendation: schema now, LocalOpenAI as a follow-up, LiteRT out of scope.**
3. **Is the WASM path at parity?** Today it is a frozen fork, no CI target builds it, no example constructs it, and it doubles the cost of every wire fix. Options: (a) extract shared code and keep parity (the plan assumes this — X1 is embedded in WP-4/WP-5); (b) put it behind a feature flag and stop claiming parity; (c) delete it. This decision changes WP-4/WP-5 sizing materially.
4. **Harness version pinning policy.** Options: pin exactly and fail loudly on mismatch (safest, needs `HARNESS_VERSION` + a startup check); accept a range; or follow latest and rely on the drift job. Whichever is chosen, `install_harness.sh` and the README must stop disagreeing — and the pin must not move before WP-1 lands.
5. **Harness-side hooks: adopt or decline?** WP-8 is XL and is the largest single item. Declining means client-side approximations forever, no `PRE_TOOL` argument rewriting, no harness-driven `PRE_TURN`/`POST_TURN`/`ON_SESSION_*`, and permanent divergence in what gets gated. If declining, H1 must still be fixed locally (0.1.1 already did) and `docs/hooks.md` must say so plainly.
6. **`ignore_unknown_fields` is a deliberate divergence.** Upstream's `json_format.Parse` is strict; we propose lenient because our descriptor will always lag. Accept the divergence, or commit to a release cadence tight enough that strictness is safe?
7. **OTel (X9).** Upstream ships an `otel` extra and `utils/otel.py`, but that module is unwired — nothing else in the 0.1.9 package references it. Mirror it (L), or declare it out of scope and document that?
8. **`policy::deny("run_command")` vs `deny("RUN_COMMAND")` (S8).** Renaming to upstream's lowercase values is the parity answer but breaks every existing Rust policy and example. Alternative: normalise both sides through a canonical-name function and keep the uppercase spelling as an accepted alias. Pick one before WP-9.
9. **Removing the DuckDuckGo fallback (T3).** It is documented in the README and has no upstream equivalent. Delete outright, or keep behind an explicitly-named non-default feature with the `python3` dependency documented?
10. **`ChatResponse.steps` (A19).** Upstream has no such field. Slice it to the current turn (recommended), or keep session-wide and document the divergence?

---

## 7. Corrections to the first-pass audit

The previous `docs/upstream-parity.md` contained four errors that this document supersedes:

- *"Our proto matches 0.1.1 exactly."* It does not. `ClientInfo` did not exist at 0.1.1 (`upstream-0.1.1-localharness.proto:7-11` shows `InputConfig` with only `storage_directory`/`port`/`bind_address`) — it is a 0.1.2 back-port. And `UsageMetadata` was `uint64` in **every** release including 0.1.1, so our `int32` is an original transcription error, not drift (W20, W22).
- *"The harness silently ignores `geminiConfig`, discarding the model settings."* It does not ignore it. The 0.1.9 harness fails the init frame with `unknown field "geminiConfig"` and closes the socket — the SDK cannot connect at all (§2.1, W3).
- *"No code changes here — this PR is the plan."* Still true of the plan itself, but the first-pass note also implied the migration was mostly a proto refresh plus some feature ports. It is not: 24 of the 202 verified findings are breaking, and several — the workspace-escape bug (S1), fail-open pre-tool gating (S2), four never-dispatched hooks (H1), unwired `ToolContext` (T1), unsent workspaces (A3) — are defects in this crate today that have nothing to do with upstream drift.
- *"`ToolResult.timeout_seconds`."* `timeout_seconds` appears only on `BaseMcpServerConfig` (`0.1.9/types.py:420-433`) and as `McpServerConfig` proto field 7. It has never been on `ToolResult` (T11).
- *"The `get`/`set` → `get_state`/`set_state` rename is already done."* True for `src/tool_context.rs:57-73` only. `HookContext` (`src/context.rs:44-64`) is still on the 0.1.1 `get`/`set` API and has no `update_state`/`lock` (H13).

---

## 8. How this audit was produced

Reproducible from a clean checkout.

1. **Fetch every upstream release.** For `V` in `0.1.1 … 0.1.9`: `pip download google-antigravity==$V --no-deps -d …`, then unzip each wheel into `scratchpad/ag/$V/`. Each contains `google/antigravity/**`, including the `*_test.py` unit tests — those are the highest-value evidence in the whole tree and should be read alongside every implementation file.
2. **Decode the harness schema.** Each wheel ships `google/antigravity/proto/localharness_pb2.py` with a serialized `FileDescriptorProto`. Parse it directly (do not trust the rendered text — see W28: 0.1.9 reports `syntax='editions'`, `edition=1001`, and the renderer drops `optional` markers) and render readable `.proto`. Rendered outputs live at `scratchpad/ag/upstream-0.1.9-localharness.proto`, `upstream-0.1.9-content.proto`, `upstream-0.1.1-localharness.proto`, plus per-version dumps `lh-0.1.1.txt … lh-0.1.9.txt` used to date every field's introduction and removal.
3. **Cross-check against the shipped binary.** `strings google/antigravity/bin/localharness | grep -E 'name=<field>|STATE_'` reveals the Go struct tags and enum value names actually compiled in. This is what confirmed `gemini_config` is absent, `mcp_servers`/`models`/`aspect_ratio` are present, and only `STATE_FULLY_IDLE` exists.
4. **Confirm the platform matrix from PyPI.** `https://pypi.org/pypi/google-antigravity/json` gives the per-release wheel list (which is how X13/X14 were settled) and per-file `digests.sha256` (which `install_harness.sh` should be using).
5. **Run eight parallel subsystem audits** — proto/wire, connection/local, hooks, policy/safety, tools, agent/conversation/triggers, wasm/tooling/docs — each required to cite `file:line` on both sides.
6. **Adversarially verify every finding.** A second pass re-read each cited line and tried to refute the claim. This downgraded, rescoped or outright refuted roughly a fifth of the first-pass findings (e.g. `tool_output_truncation` and `defer_loading` are not behaviour gaps because upstream never sets them either; the `.genai.Struct` move has nil blast radius; `read_only()` omitting FINISH does **not** break `finish` today because FINISH is never dispatched as a `ToolCall`). Verifier corrections are folded into the entries above.
7. **Spot-check by compiling.** Where a claim hinged on Rust semantics — `Path::starts_with` with `..`, pbjson's `NumberDeserialize` accepting quoted 64-bit integers — a minimal program was compiled and run rather than reasoned about.
8. **Reproduce against the real binary.** `scripts/probe_harness.py <unpacked-wheel>` drives the shipped `localharness` through its handshake with hand-written init JSON, which is how §2.1 was established. The inbound half of §2.2 is a throwaway test in `tests/` that feeds captured 0.1.9 frames to `serde_json::from_str::<OutputEvent>`; WP-1 turns those fixtures into a permanent `tests/wire_compat.rs`. Note `build.rs` needs `protoc` on `PATH` (`apt-get install protobuf-compiler`) — see X6.

**To re-run against 0.1.10+:** repeat steps 1–4, diff `lh-0.1.9.txt` against the new dump, and re-read the `*_test.py` diff for the affected modules. The CI drift job proposed in WP-11 automates steps 1–4 and step 6's mechanical half.