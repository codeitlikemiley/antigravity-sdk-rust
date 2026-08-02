# Remaining work

Executable backlog, derived from `docs/upstream-parity.md` (the 0.1.9 migration)
and `docs/fix-plan-current-defects.md` (defects against the pinned harness).
Every row is a unit of work that can be picked up on its own once its blockers
are clear. Item IDs match the two plans — read the corresponding section there
before starting one.

**Status as of 2026-08-02.** 12 items landed: WI-1…WI-8 and WI-14 (merged in
#8), WP-1 and the core of WP-2 (open in #9).

> **The SDK still cannot connect to a 0.1.9 harness.** Everything landed so far
> fixes what we send and what we can parse. The harness's mandatory first frame,
> `initialize_conversation_response`, is still never read, so a real connection
> stalls at the handshake. **WP-6 is the single unblocker** — see §1.

---

## 1. Migration — `docs/upstream-parity.md`

| ID | What | Size | Blocked by |
|---|---|---|---|
| **WP-6** | **Read the handshake frame; session continuation mode; ordered shutdown.** The one that makes a real connection possible. `scripts/probe_harness.py` documents the exact wire shapes. | M | — |
| WP-2 tail | `session_end` reply; a `callHookRequest` branch so WP-8 is exercisable; assert `clientInfo.os`/`env` on the handshake; replace the wasm in-file mock's closed Rust→Rust loop with real fixtures | S | — |
| WP-5 | Turn lifecycle and idle state machine: `STATE_CANCELLED`, `TrajectoryStateUpdate.error`, the sentinel protocol, main-trajectory tracking, cancel support | L | WP-1, WP-2 |
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
| S13 | `policy::safe_defaults(handler)` | `policy.rs` | XS |
| S15 | MCP builder `when`/`name` options; `allow_` → `approve_` auto-name | `policy.rs` | XS |
| N8 | `IntoPolicies` so policy groups compose in the builder | `policy.rs`, `agent.rs` | XS |
| C5 | `StepTracker` clears `handled_requests` on leaving WAITING — a re-asked question currently deadlocks the harness | `local.rs`, `wasm.rs` | XS |
| C2 | A freshly connected connection reports `is_idle == true` | `local.rs`, `wasm.rs` | XS |
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

1. **WP-6** — the connection unblocker.
2. **WP-5** — so turns terminate correctly against a real harness.
3. **WP-2 tail** — cheap, and makes WP-8 exercisable.
4. **§2 (0.1.15)** as a shippable non-breaking release.
5. **WP-4 + WP-7 + WP-9** — the capability surface.
6. **WP-8** — largest, and it contains the `Hook` trait break.
7. **§3 (0.2.0)** batched with WP-10, so downstream breaks once.
8. **WP-11** — the drift-detection CI job is what stops this happening again.

One standing caveat: CI still compiles neither the wasm target nor the docs, so
every `src/wasm.rs` mirror in this backlog is unverified. The cheap subset of
WP-11 is worth landing early for that reason alone.
