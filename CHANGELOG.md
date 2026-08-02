# Changelog

## 0.2.0

Migrates the wire format from upstream 0.1.1 to **0.1.9** and clears the defect
backlog recorded in `docs/upstream-parity.md` and
`docs/fix-plan-current-defects.md`.

This release **breaks source compatibility**. The breaks are batched here
deliberately so downstream code adapts once — including both `Hook` breaks, the
second of which was pulled forward for exactly that reason. See
[On the two `Hook` breaks in this release](#on-the-two-hook-breaks-in-this-release).

### Why the wire changes matter

`proto/localharness.proto` had been hand-transcribed from 0.1.1 and had drifted
eight releases. It is now generated from the descriptor embedded in the upstream
wheel (`scripts/gen_proto.py`). The transport is protojson, which matches on
field and enum **names**, so a rename is as fatal as a renumber and fails
silently: the frame is dropped as unknown rather than rejected.

Two examples this fixed, both of which the suite was previously certifying as
working:

- `HarnessConfig.gemini_config` was **removed upstream in 0.1.4**. The harness
  rejected the frame and closed the socket. The crate now sends the
  `repeated ModelConfig models` list that replaced it.
- `STATE_IDLE` was renamed `STATE_FULLY_IDLE` in 0.1.9. The idle event was
  discarded as an unknown variant, so a turn never ended.

CI now regenerates the proto from the live upstream wheel on a schedule and
fails if the checked-in schema disagrees, so the next rename surfaces as a
build failure rather than a silent hang.

### Breaking changes

**Hooks**

- Every `Hook` method takes a `&HookContext` as its final parameter, giving
  hooks access to session state and conversation metadata.
- `post_turn` receives `&str` (the response text), not `&ChatResponse`.
- `on_compaction` receives `&Step`, not `&str`.
- `on_tool_error` returns `Result<Option<String>>` — a replacement *message*.
  It could previously return a value that **cleared** the error, which reported
  a failed tool to the model as a genuine success.
- **`pre_tool_call` now fails closed.** A hook that returns `Err` denies the
  call; it was previously treated as "no objection" and the tool ran, so any
  hook bug was an open gate. A hook with a tolerable failure mode must catch it
  and return `allow: true` explicitly.
- Implementors may declare which kinds they handle via `declares()`, which is
  what drives the harness-side `enabled_hooks`.

**Tools**

- `ToolCall` gains `server_name`; `ToolResult` gains `server_name` and a
  structured `exception`.
- Registering two tools with the same name is now an error rather than a silent
  overwrite; registry order is preserved.
- Model-supplied arguments are coerced against the tool's declared JSON Schema.
  Only unambiguous conversions are performed, so a real type error still reads
  as one.
- `ToolContext` is constructed and injected. It previously existed but was
  never built, so context-aware tools did not work at all.
- Absent or empty `arguments_json` is `{}` rather than an error.

**Types and responses**

- `ChatResponse.usage_metadata` is `Option<UsageMetadata>` and reports **this
  turn**, not the session total. The running total remains on
  `Conversation::total_usage`.
- `ChatResponse.steps` carries the turn, not the whole session.
- `UsageMetadata`'s counters are `u64`, matching the harness, which has declared
  `uint64` since 0.1.1.
- `BuiltinTools::AskQuestion` (`ASK_QUESTION`) exists and drives
  `user_questions.enabled`, which was hardcoded on. **A caller passing an
  explicit `enabled_tools` list must add `ASK_QUESTION` to keep the question
  panel.**
- `BuiltinTools::read_only()` includes `FINISH`; an agent that cannot finish
  cannot terminate a turn or emit structured output.

**Triggers**

- `Trigger::run` takes a one-method `TriggerContext` instead of the full
  connection.
- `every()` invokes a callback and rejects a non-positive interval.
- `TriggerRunner::stop` exists and `Agent::stop` calls it. Triggers previously
  outlived the agent.

**Connection**

- `receive_steps()` is single-consumer. Two live streams shared one receiver and
  each took roughly half the steps, silently; a second subscriber now gets an
  error. The claim is released when the stream is dropped, so the per-turn call
  still works.
- Prompts accept multimodal parts and slash commands.

**Policy**

- Workspace scoping is applied **unconditionally**, including alongside
  `allow_all()` — which upstream documents as the way to get autonomous shell
  access *while* file tools stay scoped. The opt-out is `workspaces(vec![])`,
  not a policy.
- Generated MCP policy names use `approve_`, not `allow_`.

### Added

- Session resumption: the handshake reply is read, and `Conversation` is seeded
  with the replayed history on both transports.
- `SessionContinuationMode` on the config and builder, with upstream's RESUME
  validation.
- Cancellation: `Conversation::cancel()`. The harness answers a halt with an
  ordinary idle, so a caller-initiated halt is tracked client-side and surfaces
  as `AntigravityError::Cancelled` rather than looking like a completed turn.
- Harness-crash diagnostics: the last 20 stderr lines are retained and attached
  when the socket closes mid-turn. A crash previously ended the stream in
  silence.
- The harness-side hook channel: `CallHookRequest`/`Response`, the router, and
  `enabled_hooks`.
- Model configuration: `ModelTarget` / `ModelEndpoint` / `GeminiModelOptions`,
  the explicit → shorthand → default merge, and `GOOGLE_GENAI_USE_VERTEXAI` /
  `GOOGLE_CLOUD_PROJECT` / `GOOGLE_CLOUD_LOCATION` routing. The explicit list is
  `GeminiConfig::model_targets`, since `models` was already the crate's
  shorthand — a deliberate divergence from upstream's naming.
- MCP servers on the wire, with stdio `env` and `timeout_seconds`.
- `search_web` and `read_url_content`; named custom subagents.
- `RetryConfig` and `ToolOutputTruncation`.
- `Conversation::wait_for_idle`; `send` drains the previous turn into history.
- `safe_defaults()`, `workspace_only_for()`, and policy group composition via
  `AgentBuilder::policy_groups`.
- `docs/policy.md`.

### Fixed

- **`scripts/install_harness.sh` installed harness 0.1.1**, whose wire format
  this SDK no longer speaks — a turn against it never ends, because `STATE_IDLE`
  was renamed and protojson drops the unknown variant. It now installs 0.1.9,
  and the drift job fails if that pin ever disagrees with the version the proto
  was generated from.

- **Workspace sandbox escape.** Containment is decided after resolution — `..`
  is collapsed and symlinks are followed before comparison — and resolution
  failure is treated as outside. It fails closed.
- The workspace root no longer falls back to `/tmp/.gemini/antigravity` when
  `HOME` is unset.
- `Agent::start` routes through `policy::enforce()`; it previously bypassed it.
- Question answers are matched to the question actually asked, by index.
- Websocket message size caps are lifted; tool results and file contents
  routinely exceed tungstenite's defaults, and hitting the cap killed the
  connection mid-turn.
- A subagent going idle no longer ends the caller's turn.
- Steps queued behind an idle event are no longer dropped.
- A fresh connection reports idle, so send-then-receive cannot race.
- `on_session_end` hooks are dispatched on disconnect; they previously never ran.
- A failing `on_tool_error` hook is contained.
- Shutdown is ordered — stdin is closed first, so the harness runs its cleanup
  and persists the trajectory instead of being killed outright.
- `EDIT_FILE` policy predicates can see `diff_block`, so a rule can inspect the
  change and not only the path.
- Tool-call arguments carry arguments, not post-execution results.
- The removed DuckDuckGo/`python3` scraper.

### On the two `Hook` breaks in this release

`Hook` is broken **twice** here, deliberately, so that downstream code adapts
once rather than across two releases:

1. Signatures — `post_turn` takes `&str`, `on_compaction` takes `&Step`,
   `on_tool_error` returns `Result<Option<String>>`.
2. A `&HookContext` parameter on all nine methods.

The second was originally planned for a later release, which would have broken
every implementation a second time. It was pulled forward instead. No further
`Hook` break is on the roadmap — though this is a pre-1.0 crate and that is not
a stability guarantee.

### Out of scope

`DebugConfig` has no field in the 0.1.9 proto and is dropped rather than
invented.
