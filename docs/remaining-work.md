# Remaining work

Executable backlog, derived from `docs/upstream-parity.md` (the 0.1.9 migration)
and `docs/fix-plan-current-defects.md` (defects against the pinned harness).
Every row is a unit of work that can be picked up on its own once its blockers
are clear. Item IDs match the two plans — read the corresponding section there
before starting one.

**Status as of 2026-08-02.** 16 items landed: WI-1…WI-8 and WI-14 (merged in
#8); WP-1, the core of WP-2, the core of WP-6 and C5 (open in #9).

> **The handshake now works.** `connect()` reads
> `initialize_conversation_response`, sends `session_continuation_mode`, and
> shuts down in order so the harness persists its trajectory. What remains
> before a real 0.1.9 turn completes is **WP-5** — the idle state machine still
> tracks `parent_idle` + `active_subagent_ids` from 0.1.1, and
> `TrajectoryStateUpdate.error` and `STATE_CANCELLED` are unhandled.

**Landed in #9 beyond WP-1/WP-2:** the handshake read with replayed history
exposed as `LocalConnection::initial_history()`; `SessionContinuationMode` on
the config and builder (field 19) with upstream's RESUME validation; ordered
shutdown (close stdin → wait 3min → escalate); `StepTracker` dedup cleared on
leaving WAITING_FOR_USER.

**WP-6 remainder:** seed `Conversation` from `initial_history`; `env`
passthrough; `DebugConfig`; `save_dir` temp default; prompt control-character
sanitization; 127.0.0.1 connect fallback.

---

## 1. Migration — `docs/upstream-parity.md`

| ID | What | Size | Blocked by |
|---|---|---|---|
| WP-6 | Core landed. Remainder: seed `Conversation` from the replayed history, `env` passthrough, `DebugConfig`, `save_dir` default, prompt sanitization, 127.0.0.1 fallback | S | — |
| WP-2 tail | `session_end` reply; a `callHookRequest` branch so WP-8 is exercisable; assert `clientInfo.os`/`env` on the handshake; replace the wasm in-file mock's closed Rust→Rust loop with real fixtures | S | — |
| **WP-5** | **Turn lifecycle and idle state machine** — now the blocker for a real turn completing: `STATE_CANCELLED`, `TrajectoryStateUpdate.error`, the sentinel protocol, main-trajectory tracking, cancel support | L | WP-1, WP-2 |
| WP-4 | Model configuration public API — `ModelTarget` / `ModelEndpoint` replacing `GeminiConfig`; the wire shape is already correct, this is the type graph and the env-var routing | L | WP-1 |
| WP-7 | Tool runner correctness: `error_message` on the wire, argument coercion, `ToolContext` wiring, media extraction | M | WP-1 |
| WP-9 | Capability surface: MCP servers on the wire, `search_web`/`read_url_content`, custom subagents, retry config, tool-name casing | L | WP-1, WP-4 |
| WP-8 | Harness-side hook channel: `LifecycleHook`, `CallHookRequest`/`Response`, the router, `enabled_hooks`. Largest single item; contains the `Hook` trait break | XL | WP-1, WP-5, WP-6 |
| WP-10 | Public API surface: multimodal prompts, slash commands, `Connection` trait changes, trigger narrowing | L | WP-5, WP-6, WP-8 |
| WP-11 | Tooling, CI, docs, examples, skills; the upstream-drift detection job | M | — |

**WP-3** (security hardening) is complete — it landed as WI-1…WI-8 in #8.

---

## 2. Fix plan, release 0.1.15 — source-compatible

Twelve items, none blocking each other except where noted.

| ID | What | Files | Size |
|---|---|---|---|
| S15 | MCP builder `when`/`name` options; `allow_` → `approve_` auto-name | `policy.rs` | XS |
| N8 | `IntoPolicies` so policy groups compose in the builder | `policy.rs`, `agent.rs` | XS |
| C2 | A freshly connected connection reports `is_idle == true`. **Blocked on the connect-time race**, not on C3 — the loop restructure removed the first-poll hazard, but a caller polling `receive_steps()` before the reader sees `STATE_RUNNING` still gets an empty stream. Upstream's API is send()-then-receive; ours does not promise that. Tried twice, reverted twice; `NOTE` at both sites | `local.rs`, `wasm.rs` | S |
| A5 | `Conversation::send` drains the previous turn into history | `conversation.rs`, `connection.rs` | M |
| H9 | Contain an erroring `on_tool_error` hook instead of aborting the chain | `hooks.rs` | XS |
| H1c | Dispatch `session_end` from `disconnect()` | `local.rs`, `wasm.rs` | XS |
| hook-dispatch | New target-neutral module for shared hook plumbing | `hook_dispatch.rs` (new) | XS |
| H1a | Dispatch `pre_turn` from `Connection::send`, with upstream deny semantics | `hook_dispatch.rs`, both transports, `conversation.rs` | S |
| H12 | Dispatch `post_tool_call` on subagent completion | `local.rs`, `wasm.rs` | S |
| N3 | Structured per-tool results for harness-executed built-ins | `tool_output.rs` (new), both transports | M |

---

## 3. Fix plan, release 0.2.0 — breaking

Twenty items. Per the maintainer decision recorded in the fix plan §8.3, these
batch into one release, and wire-neutral upstream API corrections are in scope.

**Security and correctness first:**

| ID | What | Size |
|---|---|---|
| S2 | Make pre-tool gating **fail closed by construction** — it currently fails **open** at both transports when a hook errors | S |
| H4 | Narrow `on_tool_error`: never clear the error, never downgrade the step. It currently reports a failed tool to the model as a genuine success | M |
| T1+T9 | Construct and inject the `ToolContext` — it is never constructed today, so context-aware tools do not work at all | S |
| A1 | `TriggerRunner::stop` + double-start guard; `Agent::stop` calls it. Triggers currently outlive the agent | M |
| A10 | Narrow the trigger surface to a one-method `TriggerContext` | S |

**The rest:**

| ID | What | Size |
|---|---|---|
| T6 | `ToolCall.server_name`; `ToolResult.{server_name, exception}` | S |
| H11 | `ToolExecutionError{message, tool_name, server_name, source}` | XS |
| tool-wire | New ungated module both transports delegate to | S |
| T5 | Absent/empty `arguments_json` → `{}` | XS |
| W8 | Route all six `ToolResponse` constructions through one function | S |
| docs-on-tool-error | Rewrite the six documents that teach the removed behaviour | S |
| H1b | Dispatch `post_turn`; parameter → `&str` | S |
| H1d+H16 | Dispatch `on_compaction`; parameter → `&Step` | XS |
| A2 | `ChatResponse.usage_metadata` → per-turn, `Option<UsageMetadata>` | XS |
| T7+T8 | Reject duplicate tool names; insertion-ordered registry | S |
| T4 | Coerce model-supplied arguments against the tool's JSON Schema | M |
| T10 | Execute a tool-call batch concurrently | S |
| X19 | Exercise context-aware tools in an example and tests | S |
| A11 | `every()` invokes a callback and rejects a non-positive interval | S |
| finish-extractor | Map `StepUpdate.finish` to a `FINISH` `ToolCall` so it is policy-evaluated | XS |

---

## 4. Conflict-pass items — fix plan §9

Ten defects found while checking the plan against itself. All live today, none
blocked on the migration.

| ID | What | Size |
|---|---|---|
| question-answer-index-mismatch | Answers are written at the wrong index whenever a question is not multiple-choice; a hook returning more responses than questions indexes out of bounds | XS |
| wait-for-idle | `Connection` has no `wait_for_idle`; A5 ships an unsound poll loop without it | S |
| harness-crash-diagnostics | A harness crash ends the step stream silently and the captured stderr is discarded | S |
| predicate-args-fidelity | Policy predicate args drop `ActionEditFile.diff_block` and inject non-proto keys | S |
| single-consumer-receive-steps | Concurrent `receive_steps()` calls silently split the stream | XS |
| chatresponse-per-turn-steps | `ChatResponse.steps` carries whole-session history | XS |
| ask-question-builtin | `BuiltinTools` missing `ASK_QUESTION`; `user_questions.enabled` hardcoded | XS |
| tool-context-state-atomicity | Add `update_state` before X19 publishes a read-modify-write race as the flagship example | XS |
| policy-docs-and-tool-name-selectors | `docs/policy.md` is referenced by the plan but created by nothing; shipped policy examples can never match | S |
| step-error-and-ws-limits | `ActionError.error_message` is never read; WS message size at tungstenite defaults | XS |
| agent-input-validation | Empty prompts and unvalidated `conversation_id` accepted where upstream rejects | XS |

---

## 5. Sequencing

Before starting anything in §2 or §3, read **fix plan §8** — seven items edit
`Agent::start`, six edit `process_tool_calls`, and the ordering rules there are
load-bearing.

Recommended order:

1. ~~WP-6 core~~ — done. **WP-5** is now the blocker for a turn completing.
2. **WP-6 remainder** — small, and it finishes session resumption.
3. **WP-2 tail** — cheap, and makes WP-8 exercisable.
4. **§2 (0.1.15)** as a shippable non-breaking release.
5. **WP-4 + WP-7 + WP-9** — the capability surface.
6. **WP-8** — largest, and it contains the `Hook` trait break.
7. **§3 (0.2.0)** batched with WP-10, so downstream breaks once.
8. **WP-11** — the drift-detection CI job is what stops this happening again.

One standing caveat: CI still compiles neither the wasm target nor the docs, so
every `src/wasm.rs` mirror in this backlog is unverified. The cheap subset of
WP-11 is worth landing early for that reason alone.

---

## 6. Batched delivery plan

The L and XL rows above are too large to pick up in one sitting, and two of
them (WP-5, WP-8) restructure code both transports share. This breaks them into
batches sized to **one commit, one review, tree green at the end**. Batches are
ordered; within a batch the items must land together.

Each batch names its **done when** so it can be verified without re-reading the
plans.

### Phase A — finish the connection (the critical path)

| # | Batch | Items | Size | Done when |
|---|---|---|---|---|
| ~~A1~~ | ~~Main-trajectory tracking~~ — **done** | Replace `parent_idle` + `active_subagent_ids` with `main_trajectory_id` set from the first non-empty `trajectory_id`; return early for non-main trajectories; clear it in `send()` | S | A subagent going idle no longer ends the caller's turn; the `OnceLock` learning heuristic is gone |
| A2 | Sentinel restructure — **loop half done**; the `StepEvent` enum and C2 remain | `StepEvent::{Step, Idle, Close}` enum replacing the `"IDLE_SENTINEL"` magic id; loop instead of returning on first idle; `store` not `swap`; then flip the initial `is_idle` to `true` | M | Upstream's idle → step → idle scenario yields the post-idle step; `test_wasm_connection_integration_mock` still passes |
| A3 | Cancellation | `STATE_CANCELLED` arm, `Connection::cancel()`, `AntigravityError::Cancelled` | S | A cancelled turn is distinguishable from a completed one |
| A4 | Turn-level errors | `TrajectoryStateUpdate.error` (field 4); `ActionError.error_message` fallback; harness-crash stderr tail | S | A turn that fails server-side surfaces an error instead of ending silently |
| A5 | WP-6 remainder | Seed `Conversation` from `initial_history`; `env` passthrough; prompt sanitization; `save_dir` temp default; 127.0.0.1 fallback; `DebugConfig` | M | A resumed conversation starts with its history |

**After Phase A the SDK should complete a real turn against a 0.1.9 harness.**
That is the milestone worth cutting a release around.

### Phase B — the non-breaking release (0.1.15)

| # | Batch | Items | Size |
|---|---|---|---|
| B1 | Policy ergonomics | S15, N8 | XS |
| B2 | Hook plumbing module | `hook-dispatch`, H1c, H9 | S |
| B3 | Turn hooks | H1a (`pre_turn` with deny semantics), H12 | S |
| B4 | Conversation drain | A5 + `wait-for-idle` (the latter is a prerequisite, not optional) | M |
| B5 | Structured tool results | N3 | M |
| B6 | Small correctness | question-answer index mismatch, `single-consumer-receive-steps`, `ask-question-builtin`, `agent-input-validation`, `step-error-and-ws-limits` | S |
| B7 | WP-2 tail (**CI subset done**; `session_end` reply + `callHookRequest` branch remain) | `session_end` reply, `callHookRequest` branch, handshake assertions; `cargo check --target wasm32`, `cargo test --doc`, build the directory examples | M |

**The CI half of B7 landed early**, after A1 shipped to `src/local.rs` only and
nothing caught the missing `src/wasm.rs` half. CI now compiles the wasm target,
the doctests and the three directory examples — it found a genuine wasm-only
break on its first run.

### Phase C — capability surface

| # | Batch | Items | Size |
|---|---|---|---|
| C1 | Model types | `ModelTarget` / `ModelEndpoint` / `GeminiModelOptions`, `ThinkingLevel::ExtraHigh` | M |
| C2 | Model resolution | The explicit → shorthand → default merge algorithm; per-target endpoint validation | M |
| C3 | Model environment | `GOOGLE_GENAI_USE_VERTEXAI`, `GOOGLE_CLOUD_PROJECT` / `_LOCATION`; stop copying the API key onto the wire | S |
| C4 | MCP on the wire | `McpServerConfig` proto + `mcp_servers` field 14; stdio `env`, `timeout_seconds` | M |
| C5 | Retry + truncation | `RetryConfig`, `ToolOutputTruncation` | S |
| C6 | New built-ins | `search_web`, `read_url_content` configs and their step actions | M |
| C7 | Subagents | `SubagentConfig` / `SubagentCapabilities` → `custom_subagents` field 17, with upstream's three validations | L |

### Phase D — the breaking release (0.2.0)

Batch **all** of Phase D into a single release; the audit's §6 decision 1 and the
maintainer's §8.3 decisions both assume one break, not several.

| # | Batch | Items | Size |
|---|---|---|---|
| D1 | Fail-closed gating | S2 | S |
| D2 | Tool result shape | T6, H11, `tool-wire`, T5, W8 | M |
| D3 | `on_tool_error` contract | H4 + the six documents that teach the old behaviour | M |
| D4 | Tool runner | T7+T8, T4, T10, `finish-extractor` | M |
| D5 | `ToolContext` | T1+T9, `tool-context-state-atomicity`, X19 | M |
| D6 | Triggers | A10, A1, A11 | M |
| D7 | Per-turn response | A2, `chatresponse-per-turn-steps` | XS |
| D8 | Remaining hook signatures | H1b, H1d+H16 | S |

> **Before D8, settle the `HookContext` question.** The maintainer chose to ship
> the `Hook` break now and accept a second one later (§8.3), so D8's release
> notes must **not** claim the trait is settled — they must say a further break
> is expected. Pulling `HookContext`'s signature half into D8 would avoid that
> second break; that remains an open option.

### Phase E — hooks and the rest

| # | Batch | Items | Size |
|---|---|---|---|
| E1 | Hook kind registry | H3 — `declares() -> HookKinds`, the prerequisite for `enabled_hooks` | M |
| E2 | Shared state store | `StateStore`, rebuilding `HookContext` and `ToolContext` on it (H13) | M |
| E3 | Hook context threading | H5 — the context parameter on every `Hook` method | L |
| E4 | Hook proto + router | H2 — `CallHookRequest`/`Response`, the 7-entry table, always-answer guarantee | L |
| E5 | Turn on `enabled_hooks` | Emit field 16; reduce the confirmation arm to an unconditional accept | S |
| E6 | Public API surface | WP-10: multimodal prompts, slash commands, `Connection` trait | L |
| E7 | Docs, examples, drift job | WP-11 | M |

> **E5 must be last in Phase E.** Emitting `enabled_hooks` before the router
> exists converts a silent no-op into a mid-turn deadlock: the harness blocks
> waiting for a `CallHookResponse` nothing can send.

### Suggested cut points

- **After Phase A** — the SDK works against a current harness. Cut `0.1.15-rc`.
- **After Phase B** — ship `0.1.15`.
- **After Phase C** — feature parity on configuration; still non-breaking.
- **After Phase D** — ship `0.2.0`, one break.
- **Phase E** — `0.3.0`, or fold D8 into it if `HookContext` is adopted.
