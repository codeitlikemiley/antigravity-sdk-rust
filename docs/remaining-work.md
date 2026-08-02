# Remaining work

Executable backlog, derived from `docs/upstream-parity.md` (the 0.1.9 migration)
and `docs/fix-plan-current-defects.md` (defects against the pinned harness).
Every row is a unit of work that can be picked up on its own once its blockers
are clear. Item IDs match the two plans — read the corresponding section there
before starting one.

**Status as of 2026-08-02.** Phase A is complete except A2's `is_idle` flip (C2) and A5's wasm half. 16 items landed: WI-1…WI-8 and WI-14 (merged in
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
| WP-6 | Core landed; remainder landed as A5 except the wasm handshake read and `DebugConfig` | S | — |
| WP-2 tail | `session_end` reply; a `callHookRequest` branch so WP-8 is exercisable; assert `clientInfo.os`/`env` on the handshake; replace the wasm in-file mock's closed Rust→Rust loop with real fixtures | S | — |
| **WP-5** | **Turn lifecycle and idle state machine** — now the blocker for a real turn completing: `STATE_CANCELLED`, `TrajectoryStateUpdate.error`, the sentinel protocol, main-trajectory tracking, cancel support | L | WP-1, WP-2 |
| WP-4 | Model configuration public API — `ModelTarget` / `ModelEndpoint` replacing `GeminiConfig`; the wire shape is already correct, this is the type graph and the env-var routing | L | WP-1 |
| WP-7 | Tool runner correctness: `error_message` on the wire, argument coercion, `ToolContext` wiring, media extraction | M | WP-1 |
| WP-9 | Capability surface: MCP servers on the wire, `search_web`/`read_url_content`, custom subagents, retry config, tool-name casing | L | WP-1, WP-4 |
| WP-8 | Harness-side hook channel: `LifecycleHook`, `CallHookRequest`/`Response`, the router, `enabled_hooks`. Largest single item; contains the `Hook` trait break | XL | WP-1, WP-5, WP-6 |
| WP-10 | Public API surface: multimodal prompts, slash commands, `Connection` trait changes, trigger narrowing | L | WP-5, WP-6, WP-8 |
| WP-11 | Tooling, CI, docs, examples, skills — **the drift job landed**; the rest has been kept current batch by batch | M | — |

**WP-3** (security hardening) is complete — it landed as WI-1…WI-8 in #8.

---

## 2. Fix plan, release 0.1.15 — source-compatible

Twelve items, none blocking each other except where noted.

| ID | What | Files | Size |
|---|---|---|---|
| C2 | A freshly connected connection reports `is_idle == true`. **Blocked on the connect-time race**, not on C3 — the loop restructure removed the first-poll hazard, but a caller polling `receive_steps()` before the reader sees `STATE_RUNNING` still gets an empty stream. Upstream's API is send()-then-receive; ours does not promise that. Tried twice, reverted twice; `NOTE` at both sites | `local.rs`, `wasm.rs` | S |
| ~~A5~~ | **Done** — `send` drains the previous turn into history first, but only once a turn has actually been sent: a fresh connection reports not-idle, and draining there would block on a stream with nothing to deliver | `conversation.rs`, `connection.rs` | M |
| ~~hook-dispatch~~ | **Done** — `src/hook_dispatch.rs` | `hook_dispatch.rs` (new) | XS |
| ~~H1a~~ | **Done** — `gate_turn` runs before any state is touched, so a denied turn leaves the connection untouched; a hook that errors denies, matching S2 | `hook_dispatch.rs`, both transports | S |
| ~~H12~~ | **Done** — a non-main trajectory going idle dispatches `post_tool_call` for `START_SUBAGENT`, carrying the subagent's last model text (or its trajectory id). `examples/subagents.rs` now fires | `local.rs`, `wasm.rs` | S |
| N3 | Structured per-tool results for harness-executed built-ins | `tool_output.rs` (new), both transports | M |

---

## 3. Fix plan, release 0.2.0 — breaking

Twenty items. Per the maintainer decision recorded in the fix plan §8.3, these
batch into one release, and wire-neutral upstream API corrections are in scope.

**Security and correctness first:**

| ID | What | Size |
|---|---|---|
| ~~S2~~ | **Done** — `HookRunner::gate_tool_call` is the single decision point at both transports and both call sites; a hook that errors denies and the model is told the gate could not decide | S |
| ~~H4~~ | **Done** — `on_tool_error` returns `Option<String>`: it rewords the failure the model is shown and cannot clear it | M |
| ~~T1+T9~~ | **Done** — `Agent::start` attaches a `ToolContext` built on a new `WeakConnection`, so context-aware tools work and the context does not keep the session alive | S |
| ~~A1~~ | **Done** — `stop`/`is_running`/double-start guard, and `Agent::stop` stops triggers before disconnecting | M |
| ~~A10~~ | **Done** — `TriggerContext::send` is the whole surface | S |

**The rest:**

| ID | What | Size |
|---|---|---|
| ~~T6~~ | **Done** — a policy predicate can now tell `github/create_issue` from a local one, and a hook can route failures without parsing prose | S |
| ~~H11~~ | **Done** — `error::ToolExecutionError`, carried on `ToolResult::exception` | XS |
| ~~tool-wire~~ | **Done** — `src/tool_wire.rs`; the six drifted `ToolResponse` sites became one | S |
| ~~T5~~ | **Done** — absent or empty `arguments_json` is an empty object, not null | XS |
| ~~W8~~ | **Done** — and `error_message` is finally set, so a failed tool no longer reaches the harness looking like a success | S |
| ~~docs-on-tool-error~~ | **Done** — `docs/hooks.md`, both skill references and the skill's hooks example | S |
| ~~H1b~~ | **Done** — dispatched at the terminal user-facing model step | S |
| ~~H1d+H16~~ | **Done** — dispatched on the compaction step, which is what the hook receives | XS |
| ~~A2~~ | **Done** — per-turn and `Option`; the session total stays on `Conversation::total_usage` | XS |
| ~~T7+T8~~ | **Done** — duplicate names error from `Agent::start()`; the registry is a `Vec`, so order is registration order | S |
| ~~T4~~ | **Done** — `src/coerce.rs`; only unambiguous conversions, so a real type error still reads as one | M |
| ~~T10~~ | **Done** — the batch joins, and the registry lock is released before any tool body runs | S |
| ~~X19~~ | **Done** — `examples/custom_tools.rs` has a session-state tool; unit tests cover the injection and the dead-session case | S |
| ~~A11~~ | **Done** — `every(interval, callback)` plus `every_notification` for the fixed-message case; both reject a zero interval | S |
| ~~finish-extractor~~ | **Done** — `FINISH` classifies as a tool call, so the one call that ends a turn is finally visible to policies and hooks | XS |

---

## 4. Conflict-pass items — fix plan §9

Ten defects found while checking the plan against itself. All live today, none
blocked on the migration.

| ID | What | Size |
|---|---|---|
| ~~wait-for-idle~~ | **Done** — `Connection::wait_for_idle` is watch-backed, not a poll loop; also on `Conversation` | S |
| ~~harness-crash-diagnostics~~ | **Done** — landed with A4 | S |
| predicate-args-fidelity | **`diff_block` done.** Remaining: `SEARCH_DIR`/`RUN_COMMAND` args still carry non-proto result keys (`output`, `combined_output`, `exit_code`) that a predicate cannot rely on before execution | XS |
| ~~single-consumer-receive-steps~~ | **Done** — the connection hands out one live stream at a time and a second subscriber gets an error rather than half the steps. The claim is released when the stream drops, so the per-turn call still works | XS |
| ~~ask-question-builtin~~ | **Done** — `BuiltinTools::AskQuestion` (`ASK_QUESTION`), in `all_tools()`, and `user_questions.enabled` now follows it. Behaviour change: a caller passing `enabled_tools` explicitly must include it to keep the question panel, where before it was on unconditionally | XS |

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
| ~~A3~~ | Cancellation — **done**. `Conversation::cancel()` sets a `cancel_requested` flag on the connection; the reader converts the harness's plain `STATE_FULLY_IDLE` into `AntigravityError::Cancelled`, and `send()` clears the flag so it cannot leak into the next turn. Covered by `test_cancel_surfaces_cancelled_error` | S | A cancelled turn is distinguishable from a completed one |
| ~~A4~~ | Turn-level errors — **done**. The stderr reader keeps a 20-line tail; when the socket closes before idle, the stream yields `harness connection closed before the turn finished` with those lines attached. Covered by `test_harness_crash_surfaces_stderr_tail` | S | A turn that fails server-side surfaces an error instead of ending silently |
| A5 | WP-6 remainder — **done on the native transport**: `Conversation::seed_history()` seeded from the handshake reply (turn boundaries recovered from user-sourced steps), `AgentConfig::env` on `InputConfig.env`, a per-conversation temp `save_dir` default, and a 127.0.0.1 fallback for hosts where `localhost` resolves to ::1 first. **Remaining**: the same seeding on wasm (its reader loop is already running when the reply arrives, so `connect()` has nothing to hand back — it needs local.rs's blocking handshake read), and `DebugConfig`, which is not in the 0.1.9 proto and needs an upstream reading before it is designed | M | A resumed conversation starts with its history |

**After Phase A the SDK should complete a real turn against a 0.1.9 harness.**
That is the milestone worth cutting a release around.

### Phase B — the non-breaking release (0.1.15)

| # | Batch | Items | Size |
|---|---|---|---|
| B1 | Policy ergonomics | S15, N8 | XS |
| ~~B2~~ | Hook plumbing module — **done**. H1c (`session_end` from `disconnect`) and H9 (contain an erroring `on_tool_error`) already landed earlier in this branch; H9's shape changed again with H4 | `hook-dispatch`, H1c, H9 | S |
| ~~B3~~ | Turn hooks — **done** | H1a (`pre_turn` with deny semantics), H12 | S |
| ~~B4~~ | Conversation drain — **done** | A5 + `wait-for-idle` | M |
| B5 | Structured tool results | N3 | M |
| ~~B6~~ | Small correctness — **done** | question-answer index mismatch, `single-consumer-receive-steps`, `ask-question-builtin`, `agent-input-validation`, `step-error-and-ws-limits` | S |
| B7 | WP-2 tail (**CI subset done**; `session_end` reply + `callHookRequest` branch remain) | `session_end` reply, `callHookRequest` branch, handshake assertions; `cargo check --target wasm32`, `cargo test --doc`, build the directory examples | M |

**The CI half of B7 landed early**, after A1 shipped to `src/local.rs` only and
nothing caught the missing `src/wasm.rs` half. CI now compiles the wasm target,
the doctests and the three directory examples — it found a genuine wasm-only
break on its first run.

### Phase C — capability surface — **complete**

| # | Batch | Items | Size |
|---|---|---|---|
| ~~C1~~ | Model types — **done**. `ThinkingLevel` uses per-variant renames: `rename_all = "lowercase"` would have emitted `extrahigh` | `ModelTarget` / `ModelEndpoint` / `GeminiModelOptions`, `ThinkingLevel::ExtraHigh` | M |
| ~~C2~~ | Model resolution — **done**. Explicit → shorthand → defaults, deduped by model type and never by name; an explicit target without an endpoint is an error | M |
| ~~C3~~ | Model environment — **done**. `GOOGLE_GENAI_USE_VERTEXAI`/`_USE_ENTERPRISE` select Vertex, `GOOGLE_CLOUD_PROJECT`/`_LOCATION` hydrate it, and an env-only key stays off the wire | S |
| ~~C4~~ | MCP on the wire — **done**. `mcp_server(...)` was a no-op: the builder accepted servers, both strategies stored them, and nothing emitted them, so the model never saw an MCP tool | M |
| C5 | Retry + truncation | `RetryConfig`, `ToolOutputTruncation` | S |
| ~~C6~~ | New built-ins — **done**. `SEARCH_WEB`/`READ_URL_CONTENT` are `BuiltinTools`, gate their harness configs, and classify as tool calls so policies see them. `read_only()` gains `READ_URL_CONTENT`, matching upstream 0.1.6 | M |
| ~~C7~~ | Subagents — **done**, with all three validations: read-only default capabilities, `START_SUBAGENT` dropped with a warning, and an unregistered tool name is an error | L |

### Phase D — the breaking release (0.2.0) — **complete**

Batch **all** of Phase D into a single release; the audit's §6 decision 1 and the
maintainer's §8.3 decisions both assume one break, not several.

| # | Batch | Items | Size |
|---|---|---|---|
| ~~D1~~ | Fail-closed gating — **done** | S2 | S |
| ~~D2~~ | Tool result shape — **done** | T6, H11, `tool-wire`, T5, W8 | M |
| ~~D3~~ | `on_tool_error` contract — **done** | H4 + the documents that teach the old behaviour | M |
| ~~D4~~ | Tool runner — **done** | T7+T8, T4, T10, `finish-extractor` | M |
| ~~D5~~ | `ToolContext` — **done** | T1+T9, `tool-context-state-atomicity`, X19 | M |
| ~~D6~~ | Triggers — **done** | A10, A1, A11 | M |
| ~~D7~~ | Per-turn response — **done** | A2, `chatresponse-per-turn-steps` | XS |
| ~~D8~~ | Remaining hook signatures — **done** | H1b, H1d+H16 | S |

> **Before D8, settle the `HookContext` question.** The maintainer chose to ship
> the `Hook` break now and accept a second one later (§8.3), so D8's release
> notes must **not** claim the trait is settled — they must say a further break
> is expected. Pulling `HookContext`'s signature half into D8 would avoid that
> second break; that remains an open option.

### Phase E — hooks and the rest

| # | Batch | Items | Size |
|---|---|---|---|
| ~~E1~~ | Hook kind registry — **done**. `HookKinds` is opt-in and covers exactly the seven `LifecycleHook` members; `on_interaction`/`on_compaction` have none, so they stay local-only | M |
| ~~E2~~ | Shared state store — **done**. `src/state.rs`; both contexts delegate. The two stores stay separate data, deliberately | M |
| E3 | Hook context threading | H5 — the context parameter on every `Hook` method | L |
| E4 | Hook proto + router | H2 — `CallHookRequest`/`Response`, the 7-entry table, always-answer guarantee | L |
| E5 | Turn on `enabled_hooks` | Emit field 16; reduce the confirmation arm to an unconditional accept | S |
| E6 | Public API surface | WP-10: multimodal prompts, slash commands, `Connection` trait | L |
| E7 | Docs, examples, drift job — **the drift job is done** (`scripts/check_upstream_drift.py`, weekly + advisory on PRs, verified against the live 0.1.9 release). Docs and examples have been updated batch by batch alongside the code | WP-11 | M |

> **E5 must be last in Phase E.** Emitting `enabled_hooks` before the router
> exists converts a silent no-op into a mid-turn deadlock: the harness blocks
> waiting for a `CallHookResponse` nothing can send.

### Suggested cut points

- **After Phase A** — the SDK works against a current harness. Cut `0.1.15-rc`.
- **After Phase B** — ship `0.1.15`.
- **After Phase C** — feature parity on configuration; still non-breaking.
- **After Phase D** — ship `0.2.0`, one break.
- **Phase E** — `0.3.0`, or fold D8 into it if `HookContext` is adopted.
