# Fix plan — defects in this crate today

Companion to `docs/upstream-parity.md`. That document audits the whole crate
against upstream 0.1.9 and plans the wire migration (WP-1 … WP-11). **This
document plans only the subset that is wrong right now, against the harness
version this crate actually pins**, and is implementable without touching the
wire format.

Harness pin: **0.1.1** (`scripts/install_harness.sh:7`, `VERSION="0.1.1"`).
Crate version at time of writing: **0.1.14** (`Cargo.toml:3`).

---

## 1. What this covers and what it does not

### 1.1 Covers

Defects that are all four of:

1. wrong against the **currently pinned 0.1.1** harness — shippable now;
2. still correct after the 0.1.9 migration lands, or with the post-migration
   note recorded per item;
3. applied to **both** transports where relevant (`src/local.rs` native and
   `src/wasm.rs`, its hand-copied fork);
4. buildable in context — every signature below was checked against the real
   trait definitions, the `Send`/object-safety constraints, and the lint table
   at `Cargo.toml:41-71` (`clippy::all` = deny, `unwrap_used`/`expect_used`/
   `panic` = deny, `unsafe_code` = forbid, `missing_debug_implementations` =
   warn).

### 1.2 Does not cover

Everything gated on the 0.1.9 wire migration: WP-1 (proto regeneration),
WP-2 (mock harness rebuild), WP-8 (harness-side hook channel), W2
(`STATE_IDLE` → `STATE_FULLY_IDLE`), W3 (`gemini_config` → `models`), W5/W6
(new StepUpdate arms, MCP on the wire), W13/A13 (`search_web`,
`read_url_content`), S8 (lowercase tool-name rename), S6 (wire-supplied
`ToolCall.server_name`), H2 (`CallHookRequest`), C1 (ordered shutdown),
C3/C4 (idle sentinel and cascade-id heuristic), A12 (`on_file_change`),
A19 (`ChatResponse.steps` slicing), X1's full `src/harness_config.rs`
extraction, W18/WP-4 (`app_data_dir` vs `save_dir` on the wire).

Nothing planned below has to be undone by any of those. §7 states the
interaction item by item.

### 1.3 Security-relevant

These change what an agent will refuse to do. They are the reason to ship
first.

| Fix | Why it is a security defect |
|---|---|
| **S1** | `src/policy.rs:186` decides workspace containment with `Path::starts_with`, which is purely lexical. A model-supplied `file_path` of `<ws>/../../etc/passwd` is reported INSIDE the workspace. Verified by compiling: `Path::new("/allowed/workspace/../../etc/passwd").starts_with("/allowed/workspace") == true`. Symlinked ancestors are likewise never resolved. `workspace_only` is the only mechanism enforcing the sandbox. |
| **S2** | Pre-tool gating converts hook failure into ALLOW at five sites (`src/hooks.rs:257`, `src/local.rs:1005-1013`, `src/local.rs:1103`, `src/wasm.rs:644-651`, `src/wasm.rs:742`). Reachable today via `src/policy.rs:457-460`. A panicking async hook deadlocks the harness instead. |
| **S3** | `src/agent.rs:293` builds `PolicyEnforcer::new(final_policies, Vec::new())` directly, so `server_names` is permanently empty, every `server/tool` and `server/*` policy is inert, and both of upstream's fail-closed startup guards are skipped. |
| **S4** | `src/agent.rs:256-262` exempts any policy set containing `allow_all()` from workspace scoping — the exact posture `src/lib.rs:29-31`, `README.md:76,334,359,388` and eight examples recommend. Upstream's `_apply_workspace_policies` is unconditional and its docstring says so verbatim. |
| **C20** | `src/agent.rs:271` falls back to `HOME` or `/tmp`, so with `HOME` unset the agent adds `/tmp/.gemini/antigravity` — world-writable — to its own workspace allow-list. Exploitable once S1 resolves symlinks on the workspace side. |
| **S7** | `find_file` carries a `canonical_path` (`src/local.rs:1320`) but is not scoped, so it can enumerate any directory while `list_directory` cannot. |
| **T3** | `src/tools.rs:168-293` intercepts `google_search`/`web_search` and spawns `python3 -c` to scrape `https://html.duckduckgo.com`. Undeclared subprocess execution and undeclared network egress in a crate whose README sells a pure-Rust install. Exists in no upstream version. |
| **S16** | Loosening, listed here because it is inseparable from S1: relative paths are currently denied unconditionally, which is stricter than upstream but is implemented by the same line S1 replaces. |

### 1.4 Regressions against upstream 0.1.1 — the port dropped or inverted behaviour that already existed

| Fix | Evidence |
|---|---|
| **S5** | `normalize_wire_path` shipped at 0.1.1 `local_connection.py:215-222`, applied at `:265-270`, `:1078-1080` and `:1418`. The Rust port has none of it; `grep -rn 'file://\|cns\|normalize' src/` returns nothing relevant. |
| **S12** | `FINISH` has been in `read_only()` since 0.1.1 (`types.py:249-262`). `src/types.rs:214-221` omits it. |
| **S13** | `safe_defaults` exists at 0.1.1 `policy.py:371-384`. No Rust equivalent. |
| **S14** | The `has_mcp_servers` disjunct is in the 0.1.1 startup guard (`agent.py:112-124`). |
| **S15** | `_mcp_policies` took keyword-only `when`/`name` at 0.1.1 `policy.py:135-201`. |
| **H1a/H1b/H1c/H1d** | 0.1.1 dispatches `pre_turn` (`local_connection.py:496-517`), `post_turn` (`:596-610`), `session_end` (`:681-690`) and `on_compaction` (`:800-806`). All four Rust dispatchers exist and have **no production caller**. |
| **H12** | 0.1.1 dispatches `post_tool_call` on subagent trajectory idle (`local_connection.py:906-919`), fed by `_subagent_responses` (`:813-825`), cleared per turn (`:506`). |
| **T1** | 0.1.1 wires the `ToolContext` at `agent.py:176-180`. `grep -rn 'set_context'` finds no Rust caller; `ToolContext` is unreachable code. |
| **T5** | `json.loads(tool_call.arguments_json or "{}")` at 0.1.1 `local_connection.py:1151`. Rust does `unwrap_or(Value::Null)`. |
| **T6** | `ToolResult.exception` at 0.1.1 `types.py:527`, populated at `tool_runner.py:308-313`, consumed at `local_connection.py:1195`. |
| **T7/T8** | Duplicate-name `ValueError` at 0.1.1 `tool_runner.py:164-165`; `_tools` is an insertion-ordered Python dict. |
| **T10** | `asyncio.gather` at 0.1.1 `tool_runner.py:315`. |
| **A1** | `TriggerRunner.stop`/`is_running`/double-start guard at 0.1.1 `trigger_runner.py:74-113`. |
| **A10** | `TriggerContext` (one method, `send`) at 0.1.1 `triggers.py:28-50`. Rust hands triggers the whole `AnyConnection`. |
| **A11** | `every(interval, callback)` at 0.1.1 `helpers.py:39-72`. Rust takes a fixed string. |
| **C5** | `_StepTracker.update_state` clears `handled_requests` at 0.1.1 `local_connection.py:61-69`. |
| **C2** | `_is_idle.set()` / `_parent_idle = True` at construction, 0.1.1 `local_connection.py:448-459`. |
| **A5** | `send()` drains the previous turn at 0.1.1 `conversation.py:125-134`. |
| **A2** | `usage_metadata` is per-turn at 0.1.1 `types.py:964-967`. |
| **A3** | `workspaces` defaults to `[os.getcwd()]` and is passed to the strategy at 0.1.1 `local_connection_config.py:54,185`. |
| **new-finish-extractor** | `_BUILTIN_TOOL_PROTO_FIELDS` includes `FINISH: "finish"` at 0.1.1 `local_connection.py:105-116`. |
| **N3** | `_extract_tool_result` returns per-tool structured models at 0.1.1 `local_connection.py:142-207`; `connections/local/types.py` exists at 0.1.1. This **corrects the audit**, which framed N3 as 0.1.9-only. |

### 1.5 Wrong in both 0.1.1 and 0.1.9 (drift, not port regression)

- **S1** — 0.1.1 had no `_secure_normalize_path`; the hardening arrived later
  (0.1.9 `policy.py:437-506`). Rust is worse than either, because
  `Path::starts_with` does not even do the lexical work `pathlib` does.
- **H4** — 0.1.1 genuinely substituted a recovery result
  (`local_connection.py:1207-1219`), so the port copied it faithfully. Upstream
  narrowed it in 0.1.6. But the `StepStatus::Done` downgrade at
  `src/local.rs:1170-1176` is a **Rust-only invention**: 0.1.1 emits no
  synthetic terminal step at all.
- **T4** — `_coerce_args` arrived in 0.1.8. Not a regression; a real usability
  defect (`examples/custom_tools.rs:88-91` breaks on a string-typed `count`).
- **N6** — the `Agent` path already validates (`src/agent.rs:199-203`,
  pinned by `tests/integration_tests.rs:86-104`). **The audit's claim that
  "Rust silently lets enabled_tools win" is false for that path**; the real gap
  is the direct-strategy path (`src/local.rs:610-623`, `src/wasm.rs:245-260`).

### 1.6 Findings rejected

| Finding | Verdict |
|---|---|
| **H8** — "`post_tool_call` never sees a failed ToolResult; upstream fires both" | **Wrong against the pinned harness.** 0.1.1 `local_connection.py:1207-1227` is the identical `if tool_error / elif not result.error` mutual exclusion, and it is unchanged through 0.1.5 (`:1362-1382`), the last release with client-side tool hooks. The Rust is a faithful port. The audit's claim describes 0.1.9's `HookRouter`, which arrives free with WP-8. Do not act now. |
| **H7 (narrow half)** — "built-in `post_tool_call`/`on_tool_error` only fire when a `tool_confirmation_request` happened first" | **Not a defect against 0.1.1.** Upstream writes `_pending_builtin_tool_calls` in exactly one place, inside `_handle_tool_confirmation_request` (`local_connection.py:1107-1120`), and reads it in exactly one place (`:827-859`). The confirmation request *is* the only pre-tool gate that exists before `CallHookRequest` (0.1.5/0.1.6). Any "fix" now would have to synthesise information the 0.1.1 wire does not carry. |
| **S7's audit text** — "`workspace_only` should be reduced to upstream's three file tools, plus `FIND_FILE` which does carry a path" | **Internally contradictory and half wrong.** `BuiltinTools.file_tools()` is `[VIEW_FILE, CREATE_FILE, EDIT_FILE]` — three entries, `FIND_FILE` **not** among them — identically at 0.1.1 `types.py:294-307` and 0.1.9 `types.py:274-287`. Adopting it verbatim would *remove* the `LIST_DIR`/`SEARCH_DIR` denials this crate ships. See §3 WI-6 for the resolution. |
| **N6's audit text** — "Rust silently lets `enabled_tools` win" | False for the `Agent` path (see §1.5). The fix is retargeted at the strategy paths. |
| **S15's audit text** — the `allow_` → `approve_` naming difference is "user-visible in deny messages" | **False.** APPROVE policies return `HookResult{allow: true, message: String::new()}` (`src/policy.rs:417-422`); an APPROVE policy's name never reaches a user-facing string. Fix it for parity, not severity. |

---

## 2. Summary table

`api_break` values: **none** · **behaviour** (source-compatible, changes what
the agent does) · **breaking** (will not compile).

### Release 0.1.15 — security and correctness, source-compatible

| ID | One line | Files | api_break | Effort |
|---|---|---|---|---|
| **A3** | Resolve workspace roots once; send them to the harness instead of `[]` | `src/workspace.rs` (new), `src/lib.rs`, `src/agent.rs` | behaviour | XS |
| **S5** | `normalize_wire_path` — `file://`/`cns://` → native paths, in both extractors, both `connect()`, and the workspace list | `src/wire_path.rs` (new), `src/lib.rs`, `src/local.rs`, `src/wasm.rs`, `src/workspace.rs`, `src/policy.rs` | none | M |
| **S1+S16** | `secure_normalize_path` + component-wise `is_path_in_workspace`, fail closed; relative paths resolve against cwd | `src/path_safety.rs` (new), `src/lib.rs`, `src/policy.rs`, `Cargo.toml` | behaviour | M |
| **C20** | `app_data_dir`: expand `~`, require absolute, drop the `/tmp` fallback | `src/path_safety.rs`, `src/agent.rs` | behaviour | XS |
| **S12+N6** | `BuiltinTools::{read_only(+FINISH), all_tools, file_tools, path_scoped_tools}`; `CapabilitiesConfig::{validate, active_tools}` | `src/types.rs`, `src/agent.rs`, `src/local.rs`, `src/wasm.rs` | behaviour | S |
| **S7** | `workspace_only` routed through `path_scoped_tools()`; add `workspace_only_for` | `src/types.rs`, `src/policy.rs` | none | XS |
| **S4** | Apply `workspace_only` unconditionally; delete the `allow_all()` exemption; idempotent re-application | `src/agent.rs` | behaviour | S |
| **S3+S14** | Route `Agent::start` through `policy::enforce()`; add the `has_mcp_servers` guard term | `src/agent.rs` | none | XS |
| **S13** | `policy::safe_defaults(handler)` | `src/policy.rs` | none | XS |
| **S15** | MCP builder `when`/`name` options; `allow_` → `approve_` auto-name | `src/policy.rs` | none | XS |
| **N8** | `IntoPolicies` — let policy groups compose in the builder and in `enforce()` | `src/policy.rs`, `src/agent.rs` | behaviour | XS |
| **C5** | `StepTracker::update_state` clears `handled_requests` on leaving WAITING | `src/local.rs`, `src/wasm.rs` | behaviour | XS |
| **C2** | A freshly connected connection reports `is_idle == true` | `src/local.rs`, `src/wasm.rs` | behaviour | XS |
| **A5** | `Conversation::send` drains the previous turn into history | `src/conversation.rs`, `src/connection.rs` | behaviour | M |
| **T3** | Delete the `google_search`/`web_search` → `python3` → DuckDuckGo fallback | `src/tools.rs`, `README.md`, `docs/tools.md`, `.github/workflows/ci.yml` | none | XS |
| **H9** | Contain an erroring `on_tool_error` hook instead of aborting the chain | `src/hooks.rs` | behaviour | XS |
| **H1c** | Dispatch `session_end` from `disconnect()` on both transports | `src/local.rs`, `src/wasm.rs` | behaviour | XS |
| **hook-dispatch** | `src/hook_dispatch.rs` — target-neutral module for shared hook plumbing | `src/hook_dispatch.rs` (new), `src/lib.rs` | none | XS |
| **H1a** | Dispatch `pre_turn` from `Connection::send`, with upstream deny semantics | `src/hook_dispatch.rs`, `src/local.rs`, `src/wasm.rs`, `src/conversation.rs` | behaviour | S |
| **H12** | Dispatch `post_tool_call` on subagent trajectory completion | `src/local.rs`, `src/wasm.rs` | behaviour | S |
| **N3** | Structured per-tool results for harness-executed built-ins | `src/tool_output.rs` (new), `src/lib.rs`, `src/local.rs`, `src/wasm.rs` | behaviour | M |

### Release 0.2.0 — breaking

| ID | One line | Files | api_break | Effort |
|---|---|---|---|---|
| **S2** | Make pre-tool gating fail closed by construction (infallible dispatcher + shared gate) | `src/hooks.rs`, `src/local.rs`, `src/wasm.rs` | breaking | S |
| **T6** | `ToolCall.server_name`; `ToolResult.{server_name, exception}`; constructors | `src/types.rs`, `src/tools.rs`, `src/local.rs`, `src/wasm.rs`, `src/conversation.rs`, `src/hooks.rs`, `src/policy.rs` | breaking | S |
| **H11** | `ToolExecutionError{message, tool_name, server_name, source}` | `src/error.rs`, `src/types.rs`, `src/hooks.rs` | breaking | XS |
| **tool-wire** | `src/tool_wire.rs` — the ungated module both transports delegate to | `src/tool_wire.rs` (new), `src/lib.rs` | none | S |
| **T5** | Absent/empty `arguments_json` → `{}`; malformed → error result | `src/tool_wire.rs`, `src/local.rs`, `src/wasm.rs` | none | XS |
| **W8** | Route all six `ToolResponse` constructions through one function | `src/tool_wire.rs`, `src/local.rs`, `src/wasm.rs` | none | S |
| **H4** | Narrow `on_tool_error` to a message transform; never clear the error, never downgrade the step | `src/types.rs`, `src/hooks.rs`, `src/tool_wire.rs`, `src/local.rs`, `src/wasm.rs` | breaking | M |
| **docs-on-tool-error** | Rewrite the six documents that teach the removed behaviour | `docs/hooks.md`, `skills/**`, `.github/workflows/ci.yml` | none | S |
| **H1b** | Dispatch `post_turn` on the terminal user-facing model step; parameter → `&str` | `src/hooks.rs`, `src/hook_dispatch.rs`, `src/local.rs`, `src/wasm.rs` | breaking | S |
| **H1d+H16** | Dispatch `on_compaction`; parameter → `&Step` | `src/hooks.rs`, `src/local.rs`, `src/wasm.rs` | breaking | XS |
| **A2** | `ChatResponse.usage_metadata` → `Option<UsageMetadata>`, per turn not cumulative | `src/types.rs`, `src/conversation.rs`, `src/interactive.rs` | breaking | XS |
| **T7+T8** | Reject duplicate tool names; insertion-ordered registry | `src/tools.rs`, `src/types.rs`, `src/agent.rs`, `src/local.rs`, `src/wasm.rs` | breaking | S |
| **T1+T9** | Construct and inject the `ToolContext` via a `WeakConnection`; delete `is_idle`/`send` | `src/connection.rs`, `src/tool_context.rs`, `src/tools.rs`, `src/agent.rs` | breaking | S |
| **T4** | Coerce model-supplied arguments against the tool's JSON Schema | `src/coerce.rs` (new), `src/lib.rs`, `src/tools.rs` | none | M |
| **T10** | Execute a tool-call batch concurrently; release the registry lock first | `src/tools.rs` | none | S |
| **X19** | Exercise context-aware tools in an example, a unit test and an end-to-end test | `examples/custom_tools.rs`, `src/tools.rs`, `src/bin/mock_localharness.rs`, `tests/integration_tests.rs` | none | S |
| **A10** | Narrow the trigger surface to a one-method `TriggerContext` | `src/triggers.rs`, `src/trigger_helpers.rs`, `src/connection.rs` | breaking | S |
| **A1** | `TriggerRunner::{stop, is_running}` + double-start guard; `Agent::stop` calls it | `src/triggers.rs`, `src/agent.rs` | breaking | M |
| **A11** | `every()` invokes a callback and rejects a non-positive interval | `src/trigger_helpers.rs` | breaking | S |
| **finish-extractor** | Map `StepUpdate.finish` to a `FINISH` `ToolCall` so it is policy-evaluated | `src/local.rs`, `src/wasm.rs`, examples | behaviour | XS |

### Declined

| ID | Verdict |
|---|---|
| **H8** | Faithful port of 0.1.1 through 0.1.5. Arrives free with WP-8. Add an explanatory comment only. |
| **H7-narrow** | Structurally forced by the 0.1.1 wire. Closes at WP-8 step 6. Add an explanatory comment only. |

---

## 3. Work items

Each item is one reviewable commit that leaves the tree green
(`cargo build`, `cargo test --all-targets`, `cargo clippy --all-targets
--all-features -- -D warnings`, and `cargo check --target wasm32-unknown-unknown`
where the wasm fork is touched).

### 3.0 New module layout

`src/lib.rs:68-71` gates `pub mod local` on `cfg(not(target_arch = "wasm32"))`
and `pub mod wasm` on `cfg(any(target_arch = "wasm32", test))`, so the two
transports cannot import from each other. Every piece of shared logic below
therefore needs an **ungated** home. Seven new modules, created by the commit
that first needs each:

| Module | Visibility | Created by | Contents |
|---|---|---|---|
| `src/workspace.rs` | `pub` | WI-1 | `default_workspaces`, `resolve` |
| `src/wire_path.rs` | `pub` | WI-2 | `WIRE_PATH_ARGUMENT_KEYS`, `normalize_wire_path`, `normalize_path_args`, `canonical_path_from_args` |
| `src/path_safety.rs` | `pub` | WI-3 | `secure_normalize_path`, `is_path_in_workspace`, `is_case_insensitive`; WI-4 adds `home_dir`, `expand_home`, `default_app_data_dir` |
| `src/hook_dispatch.rs` | private `mod` | WI-18 | `RecvState`, `denied_turn_step`, `is_turn_terminal_step` |
| `src/tool_output.rs` | `pub` | WI-22 | `ToolOutput` + seven result structs, `extract_tool_output` |
| `src/tool_wire.rs` | `pub(crate) mod` | WI-25 | `parse_tool_arguments`, `tool_result_to_response_json`, `build_tool_response`, `tool_execution_error`, `finish_tool_result`, `dispatch_builtin_tool_error` |
| `src/coerce.rs` | private `mod` | WI-34 | `coerce_args`, `coerce_value`, `coerce_to_type` |

**Placement rule, stated once because it is easy to get wrong.**
`src/lib.rs:67` is `pub mod hooks;`, `:68` is the bare attribute
`#[cfg(not(target_arch = "wasm32"))]`, `:69` is `pub mod local;`, `:70` is
`#[cfg(any(target_arch = "wasm32", test))]`, `:71` is `pub mod wasm;`.
An unattributed `mod` declaration inserted **between `:68` and `:69`** silently
reattaches the cfg to the wrong item and breaks the wasm build. Insert new
declarations either on a new line 68 (before the attribute) or in the
un-gated block at `:73-78` alongside `pub mod policy;` / `pub mod types;`.

**Lint traps that apply to every new module.** `pub(crate) fn` inside a private
`mod` trips `clippy::redundant_pub_crate` (nursery) → hard failure under CI's
`-D warnings`; use plain `pub` inside a private module. Every new type needs
`#[derive(Debug)]` or a hand-written impl
(`missing_debug_implementations = "warn"`, `Cargo.toml:43`). No `unwrap`,
`expect`, `panic` or slicing anywhere — all three are `deny`
(`Cargo.toml:52-55`). Test modules may opt out with
`#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`, matching
the existing convention at `src/hooks.rs:338-344` and `src/types.rs:995-999`.

---

### WI-1 (A3) — Resolve workspace roots once, and give them to both halves

**Defect.** `src/agent.rs:263-268` computes the cwd fallback *inside* the
`if !has_allow_all` block at `:262`; the binding is consumed only by the policy
prepend at `:277-281` and dropped. What reaches the transport is
`self.config.workspaces.clone().unwrap_or_default()` —
`src/agent.rs:360` (native, 6th positional arg) and `src/agent.rs:315` (wasm,
`workspaces:` field) — an **empty** `Vec` whenever the caller never called
`.workspaces(...)`. `src/local.rs:588-595` then builds `proto_workspaces` from
that empty Vec and `:670` puts `HarnessConfig.workspaces = []` on the wire;
`src/wasm.rs:224-231`, `:306` is identical. The client-side policy layer
enforces `workspace_only([cwd, app_data_dir])` while the harness is told there
is no workspace at all.

**Upstream.** 0.1.1 `local_connection_config.py:54`
(`workspaces: list[str] = pydantic.Field(default_factory=lambda: [os.getcwd()])`),
`:144-150` (same field feeds `_apply_workspace_policies`), `:185`
(`workspaces=self.workspaces` passed to the strategy), `:142` (docstring:
"Users who want truly unrestricted access should set ``workspaces=[]``").

**New file `src/workspace.rs`:**

```rust
//! Workspace-root resolution shared by every transport.

/// Returns the default workspace list used when the caller configured none.
/// Mirrors upstream `local_connection_config.py:54`.
/// Returns an empty Vec when the cwd cannot be read (wasm32-unknown-unknown,
/// deleted cwd) — the same fail-soft the inline code at `src/agent.rs:264-267`
/// already has.
pub fn default_workspaces() -> Vec<String> {
    std::env::current_dir()
        .map_or_else(|_| Vec::new(), |cwd| vec![cwd.to_string_lossy().into_owned()])
}

/// Resolves the effective workspace roots for an agent configuration.
///
/// `None` (never configured) -> `[cwd]`. `Some(v)` -> exactly `v`, including
/// `Some(vec![])`, which upstream documents as the opt-out
/// (`local_connection_config.py:142`).
pub fn resolve(configured: Option<&Vec<String>>) -> Vec<String> {
    configured.map_or_else(default_workspaces, Clone::clone)
}
```

**`src/lib.rs`:** add `pub mod workspace;` in the un-gated block at `:73-78`.

**`src/agent.rs`:**

- Insert one resolution immediately after the `has_write_tools` computation
  (after `:240`), before the policy section:
  ```rust
  // Single resolution point: the policy layer and the harness must agree.
  // Upstream local_connection_config.py:54 + :144-150 + :185.
  let workspaces = crate::workspace::resolve(self.config.workspaces.as_ref());
  ```
- Delete `:263-268` (the inline `unwrap_or_else` fallback).
- `:277` becomes `let mut allowed_paths = workspaces.clone();` — it is followed
  by `allowed_paths.push(app_data_dir)`, so it must clone, not move.
- Keep the `if !workspaces.is_empty()` guard at `:270`; it is upstream's
  `if self.workspaces:` (`local_connection_config.py:144`).
- `:315` (wasm branch): `workspaces: workspaces.clone(),`
- `:360` (native branch, 6th positional arg): `workspaces.clone(),`

Clone at both branches even though they are cfg-exclusive: it keeps the code
identical on both targets and costs one `Vec<String>` per session.

**Do not touch** the `has_allow_all` gate at `:256-262` here — that is WI-9.
WI-1 must be correct whichever way WI-9 goes, and it is, because the resolution
now happens above the gate.

**Call sites.** `src/workspace.rs` (new) · `src/lib.rs:73-78` · `src/agent.rs`
insert after `:240` · `src/agent.rs:263-268` (delete) · `src/agent.rs:277` ·
`src/agent.rs:315` · `src/agent.rs:360` · `docs/agent.md:117` (document the
`[cwd]` default and the `vec![]` opt-out) · `README.md` wherever `workspaces`
is described as defaulting to empty.

**Tests.** `src/workspace.rs::tests::default_workspaces_returns_cwd` ·
`::resolve_none_is_cwd` · `::resolve_some_empty_stays_empty` (pins upstream's
documented opt-out, `local_connection_config.py:142`) ·
`::resolve_some_explicit_is_passthrough`. No upstream test exists — upstream
gets this from the pydantic `default_factory`; cite `:54` in the test comment.
Verify at the builder that `.workspaces(vec![])` stores `Some(vec![])` and is
not normalised away — `src/agent.rs:497-500` is `self.config.workspaces =
Some(workspaces)`, so it is not.

**Behaviour change.** Every user who never called `.workspaces(...)` now sends
`[cwd]` to the harness instead of `[]`. Client-side policy behaviour is
unchanged (it already used the same fallback). Migration note: to restore the
previous wire payload, call `.workspaces(vec![])` — which is also upstream's
documented way to ask for unrestricted access.

**Risk.** Behavioural, not compile-level. If a user's cwd is not where their
files are, the harness's workspace-scoped behaviour changes under them. This is
exactly upstream 0.1.1's behaviour. `current_dir()` fails on
wasm32-unknown-unknown, so the wasm default stays empty — same as today, no
regression, but it means the wasm policy layer and the wasm harness now agree
on `[]` rather than disagreeing.

---

### WI-2 (S5) — `normalize_wire_path`, applied everywhere a path crosses the wire

**Defect.** The Rust port dropped `normalize_wire_path` entirely. Both
extractors assign raw proto strings to `args` and `canonical_path`:
`src/local.rs:1320,1347,1358,1368,1385,1395` and the hand-copied fork at
`src/wasm.rs:1189,1212,1223,1233,1250,1260`. Both `connect()` implementations
push raw workspace strings into `HarnessConfig.workspaces`
(`src/local.rs:588-595`, `src/wasm.rs:224-231`), and `src/agent.rs` pushes raw
strings into `policy::workspace_only`.

Concrete failure: for `file:///tmp/ws/foo.py`, `Path::is_absolute()` is false,
so `src/policy.rs:181-183` returns "outside" and **every file tool is denied**.
After WI-3 lands, `secure_normalize_path` would join it onto cwd producing
`<cwd>/file:/tmp/ws/foo.py` — still denied, still nonsense. Fail-closed either
way, but the SDK is unusable against any harness that emits URIs, and hooks
receive unusable paths. Corroborated in-tree:
`examples/agent_server/src/main.rs:300-307` contains a system-prompt paragraph
begging the model not to emit `file://` paths.

**Upstream.** 0.1.1 `local_connection.py:215-222` (`normalize_wire_path`,
`file:` only), applied at `:265-270`, `:1078-1080`, `:1418`. `cns://` added in
0.1.3 (`:233-235`). 0.1.9: `local_connection_config.py:63-66`
(`WIRE_PATH_ARGUMENT_KEYS`) + `:68-84`, `hook_router.py:63-74`
(`_normalize_path_args`) and `:191-200` (canonical_path, first match, `break`),
`event_processor.py:261-268`, `local_connection_config.py:124-127`
(workspaces). Tests: `event_processor_test.py:35-53`;
`hook_router_test.py:900-951`.

**New file `src/wire_path.rs`.** No new dependencies — the crate has neither
`url` nor `percent-encoding`, and adding one for ~60 lines is not worth the
wasm32 build-surface risk.

```rust
/// Tool-call argument keys that carry wire-format URIs. Mirrors upstream's
/// WIRE_PATH_ARGUMENT_KEYS (local_connection_config.py:63-66). Upstream uses a
/// frozenset, so ITS iteration order is arbitrary; this ordered slice is a
/// deliberate divergence that makes canonical_path derivation deterministic.
/// It is unobservable for built-ins: no built-in tool's args carry two of these.
pub const WIRE_PATH_ARGUMENT_KEYS: [&str; 4] =
    ["path", "file_path", "TargetFile", "directory_path"];

pub fn normalize_wire_path(path: &str) -> String {
    match split_uri(path) {
        Some((scheme, _netloc, p)) if scheme == "file" => url2pathname(p),
        // urlparse("cns://el-d/home/x").netloc == "el-d", .path == "/home/x"
        Some((scheme, netloc, p)) if scheme == "cns" => format!("/cns/{netloc}{p}"),
        _ => path.to_string(),
    }
}

/// Minimal urllib.parse.urlparse split: (lowercased scheme, netloc, path).
/// Returns None when there is no RFC-3986 scheme, so plain paths pass through.
fn split_uri(s: &str) -> Option<(String, &str, &str)> {
    let colon = s.find(':')?;
    let scheme = s.get(..colon)?;
    let mut cs = scheme.chars();
    let first = cs.next()?;
    if !first.is_ascii_alphabetic() { return None; }
    if !cs.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.') {
        return None;
    }
    let rest = s.get(colon + 1..)?;
    let rest = rest.split_once('#').map_or(rest, |(a, _)| a); // urlparse strips fragment
    let rest = rest.split_once('?').map_or(rest, |(a, _)| a); // ... and query
    let (netloc, path) = match rest.strip_prefix("//") {
        Some(r) => r.find('/').map_or((r, ""), |i| (&r[..i], &r[i..])),
        None => ("", rest),
    };
    Some((scheme.to_ascii_lowercase(), netloc, path))
}

/// urllib.request.url2pathname. POSIX: unquote(). Windows (nturl2path): also
/// swaps separators and strips the leading slash before a drive letter.
fn url2pathname(p: &str) -> String {
    let decoded = percent_decode(p);
    #[cfg(windows)] {
        let s = decoded.replace('/', "\\");
        let b = s.as_bytes();
        if b.len() >= 3 && b[0] == b'\\'
            && (b[1] as char).is_ascii_alphabetic() && b[2] == b':' {
            return s.get(1..).unwrap_or("").to_string();
        }
        s
    }
    #[cfg(not(windows))] { decoded }
}

/// Byte-level percent decoding, then lossy UTF-8 — multi-byte characters are
/// encoded per byte, so decoding must not be done per char. Invalid `%` escapes
/// pass through literally, matching Python's unquote("%zz") == "%zz".
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0usize;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) =
                ((b[i + 1] as char).to_digit(16), (b[i + 2] as char).to_digit(16))
            {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Rewrites every WIRE_PATH_ARGUMENT_KEYS entry of a JSON object in place.
/// Non-objects and non-string values are left untouched. Mirrors
/// hook_router.py:63-74.
pub fn normalize_path_args(args: &mut serde_json::Value) {
    let Some(map) = args.as_object_mut() else { return };
    for key in WIRE_PATH_ARGUMENT_KEYS {
        if let Some(serde_json::Value::String(s)) = map.get(key) {
            if !s.is_empty() {
                let n = normalize_wire_path(s);
                map.insert(key.to_string(), serde_json::Value::String(n));
            }
        }
    }
}

/// First non-empty WIRE_PATH_ARGUMENT_KEYS string, in slice order.
/// Mirrors hook_router.py:193-200.
pub fn canonical_path_from_args(args: &serde_json::Value) -> Option<String> {
    let map = args.as_object()?;
    WIRE_PATH_ARGUMENT_KEYS.iter().find_map(|k| match map.get(*k) {
        Some(serde_json::Value::String(s)) if !s.is_empty() => Some(s.clone()),
        _ => None,
    })
}
```

Borrow note on `normalize_path_args`: the `map.get(key)` read must end before
the `map.insert`. As written the `if let` borrow ends at the closing brace of
the inner `if`, which NLL accepts; if it trips, bind `let owned = s.clone();`
first.

**Verified outputs** (compiled scratch binary):

| Input | Output |
|---|---|
| `file:///dev/shm/workspace/foo.py` | `/dev/shm/workspace/foo.py` (`event_processor_test.py:35-40`) |
| `cns://el-d/home/user/workspace/kittens.md` | `/cns/el-d/home/user/workspace/kittens.md` (`:42-47`) |
| `/tmp/clean-path` | `/tmp/clean-path` (`:49-53`) |
| `file:///tmp/my%20dir/a%2Bb.py` | `/tmp/my dir/a+b.py` |
| `file:///tmp/caf%C3%A9/x.py` | `/tmp/café/x.py` |
| `file://localhost/tmp/x` | `/tmp/x` (netloc dropped, mirroring upstream) |
| `file:///tmp/a?q=1` | `/tmp/a` |
| `relative/path.py`, `C:/win/path`, `""` | unchanged |

**Application site 1 — both extractors.** Refactor `src/local.rs:1293-1411` and
`src/wasm.rs:1164-1276` so every arm builds only `name` and `args`, and a single
tail does normalization and canonical-path derivation:

```rust
fn extract_builtin_tool_call(step_update: &StepUpdate) -> Option<ToolCall> {
    let traj_id = step_update.trajectory_id.clone().unwrap_or_default();
    let step_idx = step_update.step_index.unwrap_or(0);
    let id = format!("{traj_id}_{step_idx}");
    let (name, mut args): (&str, serde_json::Value) =
        if step_update.invoke_subagent.is_some() {
            ("START_SUBAGENT", serde_json::json!({
                "prompt": step_update.request_text.clone().unwrap_or_default() }))
        } else if let Some(ref fd) = step_update.find_file {
            ("FIND_FILE", serde_json::json!({
                "directory_path": fd.directory_path, "query": fd.query }))
        } else if /* ... one arm per existing branch, bodies copied verbatim ... */ {
            unreachable!()
        } else {
            return None;
        };
    crate::wire_path::normalize_path_args(&mut args);
    let canonical_path = crate::wire_path::canonical_path_from_args(&args);
    Some(ToolCall { id, name: name.to_string(), args, canonical_path })
}
```

This reproduces today's `canonical_path` assignments **exactly**, which is the
property that makes the refactor safe — verified arm by arm:

| Tool | args keys | Derived `canonical_path` | Today |
|---|---|---|---|
| `START_SUBAGENT` | `prompt` | `None` | `None` |
| `FIND_FILE` | `directory_path`, `query` | `directory_path` | `fd.directory_path` |
| `RUN_COMMAND` | `command_line`, `working_dir`, `combined_output`, `exit_code` | `None` | `None` (`working_dir` is deliberately NOT a wire-path key, matching upstream) |
| `VIEW_FILE` | `file_path`, `start_line`, `end_line` | `file_path` | `view.file_path` |
| `CREATE_FILE` | `file_path`, `contents` | `file_path` | `write.file_path` |
| `EDIT_FILE` | `file_path` | `file_path` | `edit.file_path` |
| `SEARCH_DIR` | `directory_path`, `query`, `num_results`, `output` | `directory_path` | `search.directory_path` |
| `LIST_DIR` | `directory_path` | `directory_path` | `list.directory_path` |
| `GENERATE_IMAGE` | `prompt`, `image_paths`, `image_name` | `None` | `None` |

Edge equivalence: a `None` proto field serialises to JSON `null`, which
`canonical_path_from_args` skips → `None`, same as today's `field.clone()`. An
empty-string field previously produced `Some("")`, which `src/policy.rs:176-179`
treats as "no path" (allow); now it produces `None`, which is the same allow.
**No verdict changes.**

**Application site 2 — workspaces into the harness config.**
`src/local.rs:589` and `src/wasm.rs:225`:
`directory: Some(crate::wire_path::normalize_wire_path(w))`. This is exactly
0.1.1 `local_connection.py:1418`. Do it inside `connect()`, **not** in
`LocalConnectionStrategy::new` — that fn is `pub const fn`
(`src/local.rs:351`) and a `Vec`-allocating normalization cannot be `const`;
dropping `const` would be a (theoretical) semver break.

**Application site 3 — the policy workspace list.** With WI-1 landed, the single
insertion point is `crate::workspace::resolve`:

```rust
pub fn resolve(configured: Option<&Vec<String>>) -> Vec<String> {
    configured
        .map_or_else(default_workspaces, Clone::clone)
        .iter()
        .map(|w| crate::wire_path::normalize_wire_path(w))
        .collect()
}
```

This mirrors 0.1.9 `local_connection_config.py:124-127` and, because WI-1 made
`resolve` the sole producer, it normalizes the harness list and the policy list
from one call.

**Call sites.** `src/wire_path.rs` (new) · `src/lib.rs:73-78` (`pub mod
wire_path;` — un-gated, must not sit inside the `:68-71` gates) ·
`src/local.rs:1293-1411` (remove the six `canonical_path:` assignments at
`:1320,1347,1358,1368,1385,1395` and the three `canonical_path: None` at
`:1308,1335,1407`) · `src/wasm.rs:1164-1276` (identical fork:
`:1189,1212,1223,1233,1250,1260` and `:1177,1200,1272`) ·
`src/local.rs:588-595` · `src/wasm.rs:224-231` · `src/workspace.rs::resolve` ·
`src/policy.rs:552-568` (test helper `make_tool_call` — its hardcoded key order
already matches `WIRE_PATH_ARGUMENT_KEYS`; replace the hand-rolled loop with
`crate::wire_path::canonical_path_from_args(&args)` so tests and production
cannot drift) · `src/wasm.rs:1329-1531` (existing extractor unit tests — verify
unchanged: every fixture uses plain strings such as `"dir_path"`/`"view_path"`,
which `normalize_wire_path` returns verbatim, so `:1344`, `:1413`, `:1436`,
`:1458`, `:1498` keep passing) · `src/local.rs:1006`, `src/wasm.rs:645`
(tool-confirmation path) and `src/local.rs:447`, `src/wasm.rs:447` (step path)
— **no change needed**, listed so the reviewer can confirm both paths inherit
the fix.

**Tests.**
`src/wire_path.rs::tests::normalize_wire_path_file_uri` — port of 0.1.9
`event_processor_test.py:35-40`.
`::normalize_wire_path_cns_uri` — port of `:42-47`.
`::normalize_wire_path_plain_path` — port of `:49-53`.
`::normalize_wire_path_percent_decodes` — NEW (upstream gets it free from
`url2pathname`): the three percent cases above, plus `%zz` passing through.
`::normalize_wire_path_is_idempotent` — NEW:
`normalize_wire_path(normalize_wire_path(x)) == normalize_wire_path(x)` for all
fixtures. This is what licenses calling it again inside `secure_normalize_path`
(WI-3).
`::canonical_path_from_args_prefers_declared_order` and
`::skips_empty_and_non_string` — NEW; pins the deterministic-order divergence
from upstream's frozenset.
`src/local.rs::tests::extract_builtin_tool_call_normalizes_file_uri` and the
mirror `src/wasm.rs::tests::extract_builtin_tool_call_normalizes_file_uri` —
port of 0.1.9 `hook_router_test.py:900-951`
(`test_handle_pre_tool_normalizes_wire_paths`), adapted to the 0.1.1 extractor:
a `view_file` `StepUpdate` with `file_path = "file:///home/user/foo.py"` must
yield `args["file_path"] == "/home/user/foo.py"` **and**
`canonical_path == Some("/home/user/foo.py")`. Upstream's second half uses
`TargetFile`; add the equivalent via `create_file`.
`tests/integration_tests.rs::workspace_policy_allows_file_uri_inside_workspace`
— NEW end-to-end guard: a `file://` URI under the configured workspace is
ALLOWED (today it is denied). No upstream equivalent; this is the headline
symptom.
`src/local.rs::tests::harness_config_workspaces_are_normalized` — NEW: a
`file:///tmp/ws` workspace reaches `HarnessConfig.workspaces` as `/tmp/ws`.
Pins 0.1.1 `local_connection.py:1418`.

**Behaviour change.** `file://` and `cns://` paths in tool arguments and in the
configured workspace list now reach hooks, policies and the harness as clean
absolute filesystem paths. Against a harness that emits URIs this flips every
file tool from DENIED to correctly evaluated. Against a harness that emits plain
paths — which is what the CI mock does — the transformation is the identity, so
nothing observable changes. `ToolCall.args` values are rewritten in place, so a
user hook reading `args["file_path"]` sees the clean path instead of the URI.

**Risk.**
1. Two extractors must change identically (X1). A fix landing only in
   `src/local.rs` leaves the WASM path broken and the wasm unit tests will not
   catch it, because they assert on plain strings. **Land both in one commit.**
2. The refactor changes control flow from early `return`s to an if/else chain;
   keep the `#[allow(clippy::too_many_lines)]` at `src/local.rs:1293` /
   `src/wasm.rs:1164`, and type-annotate `(&str, serde_json::Value)` explicitly
   or the arms will not unify.
3. `split_uri` must not misfire on Windows drive letters. Verified:
   `"C:/win/path"` parses as scheme `"c"`, which is neither `file` nor `cns`, so
   it passes through untouched.
4. Dropping the URI netloc for `file://host/share` mirrors an upstream defect (a
   UNC share becomes a local path). Kept for parity; if you would rather not,
   return the input unchanged when `netloc` is non-empty and not `localhost`,
   and note the divergence.
5. Ordering with WI-3: `normalize_wire_path` must run **before**
   `secure_normalize_path`, or a URI gets joined onto cwd. The belt-and-braces
   call at the top of `secure_normalize_path` guarantees it even if an extractor
   arm is missed.

---

### WI-3 (S1 + S16) — `secure_normalize_path` + component-wise containment, fail closed

**Defect.** `src/policy.rs:175-191` decides workspace containment on the raw
wire string. `:180` builds `Path::new(path_str)`; `:186` calls
`target_path.starts_with(ws_path)`. `Path::starts_with` is purely lexical — it
never resolves `..`, `.` or symlinks. Compiled and run (rustc 1.8x):

```
Path::new("/allowed/workspace/../../etc/passwd").starts_with("/allowed/workspace") => true  (reported INSIDE)
Path::new("/allowed/workspace/./../secret").starts_with("/allowed/workspace")     => true  (reported INSIDE)
Path::new("/tmp/workspace-evil/x").starts_with("/tmp/workspace")                  => false (correct; keep)
```

A symlink `<ws>/link -> /etc` is likewise never resolved, so `<ws>/link/passwd`
is INSIDE. Additionally `:184-188` silently skips any workspace whose *string*
is not absolute (`ws_path.is_absolute()`), so a relative workspace entry
disables that root with no diagnostic. And `:181-183`
(`if !target_path.is_absolute() { return true; }`) denies every relative
`canonical_path` outright — stricter than upstream, whose
`_secure_normalize_path` is `pathlib.Path(path).resolve()` with no absoluteness
precondition (**S16**).

**Upstream.** 0.1.9 `hooks/policy.py:437-449` (`_secure_normalize_path`, with
the explicit comment that `strict=True` is not used because new files do not
exist yet), `:452-479` (`_is_case_insensitive`), `:483-506`
(`_is_path_in_workspace`: fail closed on OSError at `:490-492`, case fold at
`:494-499`, length guard `:501-502`, component prefix compare `:506`). Tests:
`policy_test.py:840-855`, `:857-864`, `:866-886`, `:888-898`, and the
`workspace_only` suite at `:728-829`. 0.1.1 has no `_secure_normalize_path`, so
this half is drift, not a port regression.

**New file `src/path_safety.rs`**, declared unconditionally in `src/lib.rs`
next to `pub mod policy;` as `pub mod path_safety;`. It must **not** live in
`src/local.rs` or `src/wasm.rs`; `src/policy.rs` would also work, but
`src/agent.rs` needs `default_app_data_dir`/`expand_home` from the same module
(WI-4), so a dedicated module is cleaner. No new runtime dependencies.

**Public API (all additive):**

```rust
pub fn secure_normalize_path(path: &str) -> std::io::Result<std::path::PathBuf>
pub fn is_path_in_workspace(target: &str, workspace: &str) -> bool
pub fn is_case_insensitive(path: &std::path::Path) -> bool
```

Use `&str`, not `&Path`/`impl AsRef<Path>`: the caller has `Option<String>`
(`ToolCall::canonical_path`, `src/types.rs:357`), the function must run
`normalize_wire_path` first (a string op), and a non-generic signature keeps the
closure trivially `Send + Sync`.

**Algorithm.** Rust has no `Path::resolve(strict=False)`.
`std::fs::canonicalize` requires the whole path to exist, which is unusable
because `create_file` targets a path that does not exist yet. Mirror
`pathlib.Path(p).resolve()` by walking components and resolving symlinks
incrementally; `..` is applied to the already-resolved prefix, which is what
makes `<ws>/link/../x` correct.

```rust
use std::collections::VecDeque;
use std::ffi::OsString;
use std::io;
use std::path::{Component, Path, PathBuf};

const MAX_SYMLINK_HOPS: u32 = 40;

enum Seg { Cur, Parent, Normal(OsString) }

/// Splits a path into its root (prefix + root dir, possibly empty) and owned
/// segments. Owned segments are required: `Component<'a>` borrows the path, and
/// we need to push symlink-target segments back onto the queue.
fn split_root(p: &Path) -> (PathBuf, VecDeque<Seg>) {
    let mut root = PathBuf::new();
    let mut segs = VecDeque::new();
    for c in p.components() {
        match c {
            // On Windows `push("C:")` then `push("\\")` yields `C:\` —
            // `PathBuf::push` keeps the prefix when the pushed path has a root
            // but no prefix.
            Component::Prefix(_) | Component::RootDir => root.push(c.as_os_str()),
            Component::CurDir => segs.push_back(Seg::Cur),
            Component::ParentDir => segs.push_back(Seg::Parent),
            Component::Normal(n) => segs.push_back(Seg::Normal(n.to_os_string())),
        }
    }
    (root, segs)
}

enum Probe { Missing, Regular, Symlink, Denied(io::Error) }

fn probe(p: &Path) -> Probe {
    match std::fs::symlink_metadata(p) {
        Ok(md) if md.file_type().is_symlink() => Probe::Symlink,
        Ok(_) => Probe::Regular,
        Err(e) if e.kind() == io::ErrorKind::NotFound => Probe::Missing,
        // wasm32-unknown-unknown has no filesystem: every fs call is
        // Unsupported. Failing closed here would deny every file tool on that
        // target, so degrade to lexical-only normalization instead.
        Err(e) if e.kind() == io::ErrorKind::Unsupported => Probe::Missing,
        // EACCES, ELOOP, ENOTDIR, ENAMETOOLONG -> fail closed
        Err(e) => Probe::Denied(e),
    }
}

pub fn secure_normalize_path(path: &str) -> io::Result<PathBuf> {
    // Defence in depth: idempotent for every non-file/non-cns input, and
    // protects the wasm path if an extractor arm is ever missed. Upstream
    // normalizes only at the extractor; doing it here too cannot change the
    // result for a clean path (pinned by wire_path's idempotence test).
    let path = crate::wire_path::normalize_wire_path(path);
    let p = Path::new(&path);
    let abs: PathBuf = if p.is_absolute() {
        p.to_path_buf()
    } else {
        // S16: upstream resolves relative paths against cwd.
        std::env::current_dir()?.join(p)
    };
    let (root, mut pending) = split_root(&abs);
    let mut out = root;
    let mut hops: u32 = 0;
    while let Some(seg) = pending.pop_front() {
        match seg {
            Seg::Cur => {}
            // pop() at the root returns false and leaves `out` at the root,
            // matching Python: Path("/..").resolve() == PosixPath("/").
            Seg::Parent => { out.pop(); }
            Seg::Normal(n) => {
                out.push(&n);
                match probe(&out) {
                    Probe::Denied(e) => return Err(e),
                    Probe::Missing | Probe::Regular => {}
                    Probe::Symlink => {
                        hops += 1;
                        if hops > MAX_SYMLINK_HOPS {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                "too many levels of symbolic links",
                            ));
                        }
                        let target = std::fs::read_link(&out)?;
                        out.pop();
                        let (troot, tsegs) = split_root(&target);
                        if target.is_absolute() { out = troot; }
                        for s in tsegs.into_iter().rev() { pending.push_front(s); }
                    }
                }
            }
        }
    }
    Ok(out)
}
```

**Verified behaviour** (compiled scratch binary; `tmp = /tmp/pt_demo`,
`ws = tmp/ws`, `tmp/ws/link -> tmp/outside`):

| Input | Normalized | Inside? | Why it matters |
|---|---|---|---|
| `ws/sub/file.txt` | `/tmp/pt_demo/ws/sub/file.txt` | true | existing leaf |
| `ws/newdir/newfile.txt` | `/tmp/pt_demo/ws/newdir/newfile.txt` | true | **non-existent leaf AND intermediate dir — `create_file` still works** |
| `ws/../../etc/passwd` | `/tmp/etc/passwd` | false | `..` escape |
| `ws/./../secret` | `/tmp/pt_demo/secret` | false | `./..` escape |
| `ws/link/evil.txt` | `/tmp/pt_demo/outside/evil.txt` | false | symlinked ancestor |
| `ws` | `/tmp/pt_demo/ws` | true | path == workspace |
| `ws-evil/file.txt` | `/tmp/pt_demo/ws-evil/file.txt` | false | prefix attack |
| `/..` | `/` | — | root parent clamps |
| `""` | cwd | — | matches `Path("").resolve()` |

**`is_case_insensitive`** (mirrors 0.1.9 `policy.py:452-479`):

```rust
pub fn is_case_insensitive(path: &Path) -> bool {
    const PLATFORM_DEFAULT: bool = cfg!(any(windows, target_os = "macos"));
    #[cfg(not(unix))] { let _ = path; PLATFORM_DEFAULT }
    #[cfg(unix)] {
        // Walk up to the nearest existing ancestor with at least one cased char
        // and probe it, exactly like upstream's recursive parent probe.
        let mut cur = path;
        for _ in 0..64 {
            let Some(name) = cur.file_name().and_then(|n| n.to_str())
                else { return PLATFORM_DEFAULT };
            let swapped: String = name.chars().flat_map(swap_case).collect();
            if swapped == name {
                match cur.parent() {
                    Some(p) if p != cur => { cur = p; continue }
                    _ => return PLATFORM_DEFAULT,
                }
            }
            let Some(parent) = cur.parent() else { return PLATFORM_DEFAULT };
            if !cur.exists() { return PLATFORM_DEFAULT }   // upstream policy.py:458-459
            return same_file(cur, &parent.join(swapped));
        }
        PLATFORM_DEFAULT
    }
}

#[cfg(unix)]
fn same_file(a: &Path, b: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (std::fs::metadata(a), std::fs::metadata(b)) {
        (Ok(x), Ok(y)) => x.dev() == y.dev() && x.ino() == y.ino(),
        _ => false, // upstream: except OSError -> False
    }
}

fn swap_case(c: char) -> impl Iterator<Item = char> {
    // char::to_lowercase/to_uppercase return iterators (1:N mappings exist).
    if c.is_uppercase() { Box::new(c.to_lowercase()) as Box<dyn Iterator<Item = char>> }
    else { Box::new(c.to_uppercase()) }
}
```

Upstream caches this with `lru_cache(maxsize=256)`. Optional here: a
`static CI_CACHE: OnceLock<Mutex<HashMap<PathBuf, bool>>>` cleared wholesale
past 256 entries, accessed with `lock().ok()` — never `unwrap`.
**Acceptable simplification for v1:** drop the dynamic probe entirely and return
`PLATFORM_DEFAULT`. That is exactly what upstream falls back to for
non-existent paths and what `policy_test.py:857-864` asserts; the only loss is
case-sensitive APFS volumes on macOS (would fold when it should not — an
under-deny) and case-insensitive mounts on Linux (would not fold — an
over-deny, safe).

**`is_path_in_workspace`** (mirrors `policy.py:483-506`):

```rust
pub fn is_path_in_workspace(target: &str, workspace: &str) -> bool {
    let (Ok(t), Ok(w)) = (secure_normalize_path(target), secure_normalize_path(workspace))
        else { return false };            // policy.py:490-492: fail closed on OSError
    let fold = is_case_insensitive(&w);   // policy.py:494 probes the WORKSPACE
    let key = |c: std::path::Component<'_>| {
        let s = c.as_os_str().to_string_lossy().into_owned();
        if fold { s.to_lowercase() } else { s }
    };
    let t_parts: Vec<String> = t.components().map(key).collect();
    let w_parts: Vec<String> = w.components().map(key).collect();
    if t_parts.len() < w_parts.len() { return false; }   // policy.py:501-502
    // No slicing: keeps clippy quiet and cannot panic.
    t_parts.iter().zip(w_parts.iter()).all(|(a, b)| a == b)
}
```

An empty workspace **list** is handled by the caller, not here:
`workspace_only(vec![])` leaves the `any()` over an empty list false, so
`_outside_workspace` is true and every scoped tool with a path is DENIED. That
is upstream's semantics (`policy.py:530`) and is already this crate's behaviour
— preserve it, and pin it with a test. Distinct from the config-level case at
`local_connection_config.py:128-130`, which skips prepending `workspace_only`
entirely when `workspaces` is empty — that is `src/agent.rs:270`'s
`if !workspaces.is_empty()`.

**Rewrite of `src/policy.rs:175-191`:**

```rust
    let is_outside_workspace = move |tc: &ToolCall| -> bool {
        let path_str = tc.canonical_path.as_deref().unwrap_or("");
        if path_str.is_empty() {
            return false; // policy.py:526-528: omit-path edge cases stay allowed
        }
        !workspaces
            .iter()
            .any(|ws| crate::path_safety::is_path_in_workspace(path_str, ws))
    };
```

Delete `use std::path::Path;` at `src/policy.rs:9` if nothing else in the file
uses it. The closure still captures only `workspaces: Vec<String>`, so
`Arc<dyn Fn(&ToolCall) -> bool + Send + Sync>` at `src/policy.rs:34` is
unaffected and `workspace_only`'s signature is unchanged — `docs/hooks.md:320`,
`skills/.../safety_policies.md:42` and `src/agent.rs:279` keep compiling
verbatim.

Normalization is done **per call**, as upstream does, rather than precomputed at
`workspace_only()` construction: `workspace_only` returns `Vec<Policy>` not
`Result`, so a failing workspace has nowhere to surface at construction, and
per-call keeps picking up a workspace directory created after `Agent::start`.

**S16 sub-decision — shipping without the relative-path loosening.** S16 is the
one part of this item that *loosens* behaviour, so it is reviewable and
revertable on its own. To ship WI-3 without it, replace the `else` branch of the
`abs` binding with:

```rust
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "relative path"));
```

which keeps today's deny while still fixing `..` and symlinks.

**Safety argument for taking S16:** (a) the default workspace *is* cwd
(`src/agent.rs:264-267`), and for the native strategy the harness is a child
process that inherits our cwd, so the two agree; (b) a relative path that
escapes — `../../etc/passwd` — still normalizes above cwd and is still denied,
because the `..` collapsing happens after the join; (c) it is exactly what
upstream does, so a policy ported from Python behaves the same.

**Caveat to document:** for `WasmConnectionStrategy` the harness is a *remote*
process (`src/wasm.rs:108-121` dials `ANTIGRAVITY_HARNESS_HOST`), so its cwd is
unrelated to ours and a relative path resolved locally is meaningless. On
`wasm32-unknown-unknown` `current_dir()` returns `Unsupported`, so
`secure_normalize_path` errors and the call fails closed — which is the correct
outcome there. **Do not "fix" that by defaulting to `/`.**

**Call sites.** `src/path_safety.rs` (new) · `src/lib.rs:73-78` (`pub mod
path_safety;`) · `src/policy.rs:9` (drop `use std::path::Path` if now unused) ·
`src/policy.rs:175-191` (closure body replaced) · `src/policy.rs:181-183`
(delete the `!target_path.is_absolute()` early return) · `src/policy.rs:184-189`
(drop `ws_path.is_absolute() &&`) · `src/agent.rs:279` (call site unchanged —
verify it still compiles; signature is stable) · `Cargo.toml` (add
`[dev-dependencies] tempfile = "3"` for the new tests, or hand-roll temp dirs
under `std::env::temp_dir()`; there are no dev-dependencies today) ·
`docs/agent.md:117` (state that relative workspaces resolve against cwd).

**Tests.**

Ports:
- `src/path_safety.rs::tests::secure_normalize_path_resolves_existing_symlinks`
  — port of `policy_test.py:840-855`. Create `real_dir`, symlink
  `symlink_dir -> real_dir`, assert
  `secure_normalize_path(symlink_dir/"file.txt") == real_dir/"file.txt"`.
  `#[cfg(unix)]` + `std::os::unix::fs::symlink`; skip (return early) if symlink
  creation errors, as upstream does at `policy_test.py:848-849`.
- `::is_path_in_workspace_structural_containment` — port of `:866-886`.
- `::is_case_insensitive_prober` — port of `:857-864`:
  `assert_eq!(is_case_insensitive(&tmp), cfg!(any(windows, target_os = "macos")))`.
- `::is_path_in_workspace_case_folding` — port of `:888-898`; branch on
  `is_case_insensitive(ws)`; on a case-sensitive fs assert the uppercased target
  is NOT in the workspace.
- `src/policy.rs::tests::test_workspace_only_rejects_prefix_attack` — port of
  `policy_test.py:808-819`.
- `::test_workspace_only_allows_exact_workspace_path` — port of `:821-829`.
- `::test_workspace_only_allows_when_no_path_arg` — port of `:776-783`: a
  `VIEW_FILE` call with no path argument is ALLOWED.

New (no upstream equivalent; upstream gets these free from
`resolve(strict=False)`):
- `::normalizes_nonexistent_leaf_and_intermediate` — `<ws>/newdir/newfile.txt`
  normalizes unchanged and is inside. **This is the `create_file` regression
  guard.**
- `::dotdot_escape_is_outside` — `<ws>/../../etc/passwd` and `<ws>/./../secret`
  are outside. **This is the S1 regression test.**
- `::symlink_escape_is_outside` — `#[cfg(unix)]`; `<ws>/link -> <tmp>/outside`;
  `<ws>/link/evil.txt` is outside.
- `::root_parent_clamps` — `secure_normalize_path("/..") == "/"`.
- `::relative_target_resolves_against_cwd` (S16) — set cwd to a temp workspace,
  assert `is_path_in_workspace("sub/file.txt", ws)` is true and
  `is_path_in_workspace("../escape.txt", ws)` is false.
- `::relative_workspace_entry_resolves` (S16) —
  `is_path_in_workspace(<abs under cwd>, ".")` is true.
- `src/policy.rs::tests::test_workspace_only_relative_escape_still_denied` —
  the security half of S16.
- `::test_workspace_only_empty_list_denies_all_scoped_tools` — pins
  `workspace_only(vec![])` => deny (derived from `policy.py:530`).

Must keep passing unmodified: `src/policy.rs::tests::test_workspace_only`
(existing, `:860-875`). Its paths `/allowed/workspace/...` and `/forbidden/...`
do not exist, so both sides normalize lexically and the verdicts are unchanged.

**cwd-mutating tests are process-global.** Guard them with a shared
`static CWD_LOCK: Mutex<()>` — `set_current_dir` is process-global and Rust
tests run in parallel threads. Do not scatter `set_current_dir` calls.

**Behaviour change.** Paths containing `..`/`.` or traversing a symlink out of
the workspace are now DENIED where they were previously ALLOWED. A workspace
root that is itself a symlink now resolves, so a target under the real directory
matches — previously a symlinked workspace root matched nothing at all unless
the string prefixes lined up. On macOS and Windows comparison becomes
case-insensitive (previously always case-sensitive), which ALLOWS `/WS/x`
against workspace `/ws` where it used to be denied. **(S16)** A relative
`canonical_path` is now resolved against the SDK process's cwd and allowed when
it lands inside a workspace; relative paths that escape remain denied. A
relative entry in the workspace list is now honoured instead of silently
ignored.

**Risk.**
1. **Blocking filesystem I/O inside a sync predicate** that runs on a tokio
   worker via `PolicyEnforcer::pre_tool_call` (`src/policy.rs:388-393`). It is a
   handful of `symlink_metadata` calls, sub-millisecond on local disks; upstream
   is likewise synchronous. **Do NOT reach for `spawn_blocking`/`block_on`** —
   the predicate type is `Fn(&ToolCall) -> bool` and `block_on` inside a runtime
   panics. Async predicates are S10, out of scope.
2. The predicate runs inside `std::panic::catch_unwind` (`src/policy.rs:391`); a
   panic converts to DENY, so it is fail-closed either way, but the new code
   must contain no `unwrap`/`expect`/`panic`/slicing.
3. **Windows:** `read_link` on a junction can return a verbatim `\\?\C:\...`
   path while the workspace side does not, making the component vectors differ
   and over-denying. Fail-closed, so acceptable; optionally strip
   `Component::Prefix(VerbatimDisk)` down to `Disk` in `split_root`. There is no
   Windows CI job today (X3), so this is untested-by-construction.
4. `to_lowercase()` vs Python's `casefold()` differ for ß/ſ/İ. Benign
   (over-strict) and only on case-insensitive volumes.
5. **TOCTOU:** a symlink swapped between the policy check and the harness's open
   is not defended against — same as upstream, which cannot be fixed at this
   layer.
6. `canonical_path` is the only thing scoped. `GENERATE_IMAGE.image_paths` is a
   repeated field and carries no `canonical_path` (`src/local.rs:1398-1408`), so
   it stays unscoped, exactly as upstream.
7. **S16 is a loosening.** If any deployment relies on "relative means denied"
   as a crude filter, it regresses. Mitigation: the escape case is still denied,
   and the migration note is explicit.
8. `current_dir()` is a syscall on every relative-path check. Only relative
   paths pay for it, and built-in tools essentially always emit absolute paths.

---

### WI-4 (C20) — `app_data_dir`: expand `~`, require absolute, drop the `/tmp` fallback

**Defect.** `src/agent.rs:271-278`:

```rust
let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
let app_data_dir = self.config.app_data_dir.clone()
    .unwrap_or_else(|| format!("{home}/.gemini/antigravity"));
let mut allowed_paths = workspaces;
allowed_paths.push(app_data_dir);
```

Three defects. **(1)** With `HOME` unset the agent grants itself a workspace
root under world-writable `/tmp`; upstream never produces a `/tmp` path —
`DEFAULT_APP_DATA_DIR` is `Path("~")/".gemini"/"antigravity"` expanded and
resolved at import (`local_connection_config.py:35-37`), and Python's
`expanduser` falls back to the passwd database, never to `/tmp`. After WI-3 this
is exploitable in the obvious way: an attacker pre-creates
`/tmp/.gemini/antigravity` as a symlink to a directory of their choosing,
`secure_normalize_path` faithfully resolves the **workspace** side, and the
agent gets write access to the link target. **(2)** A user-supplied
`app_data_dir` is neither `~`-expanded nor validated absolute; upstream rejects
a non-absolute value with a `ValueError` (`:109-113`). **(3)** It is never
normalized, so an entry that is relative or a URI becomes a workspace root that
matches nothing.

**Upstream.** 0.1.9 `local_connection_config.py:35-37`, `:109-113`
(`_validate_app_data_dir`, raising
`f"app_data_dir must be an absolute path, got '{v}'"`), `:119-127`
(`app_data_path = self.app_data_dir or DEFAULT_APP_DATA_DIR`;
`expanduser().resolve()`; appended to `allowed_paths` **after** the normalized
workspaces). No dedicated upstream test; the behaviour is pinned by the pydantic
validator itself.

**Add to `src/path_safety.rs`:**

```rust
/// $HOME on unix; %USERPROFILE% then %HOMEDRIVE%%HOMEPATH% on Windows.
/// Hand-rolled rather than using `std::env::home_dir` (deprecated for years
/// over its wrong Windows semantics) or pulling in the `dirs` crate.
pub fn home_dir() -> Option<std::path::PathBuf> {
    #[cfg(not(windows))]
    { std::env::var_os("HOME").filter(|v| !v.is_empty()).map(Into::into) }
    #[cfg(windows)]
    {
        if let Some(p) = std::env::var_os("USERPROFILE").filter(|v| !v.is_empty()) {
            return Some(p.into());
        }
        let (d, p) = (std::env::var_os("HOMEDRIVE")?, std::env::var_os("HOMEPATH")?);
        let mut s = d; s.push(&p); Some(s.into())
    }
}

/// Expands a leading `~` or `~/`. `~user/...` is NOT supported (it needs the
/// passwd database) and is reported as an error rather than silently treated
/// as a literal directory named "~user".
pub fn expand_home(path: &str) -> Result<String, anyhow::Error> {
    if path == "~" || path.starts_with("~/")
        || (cfg!(windows) && path.starts_with("~\\"))
    {
        let home = home_dir().ok_or_else(|| anyhow::anyhow!(
            "cannot expand '~' in '{path}': HOME (or USERPROFILE on Windows) is \
             not set. Pass an absolute app_data_dir instead."))?;
        let rest = path.get(1..).unwrap_or("").trim_start_matches(['/', '\\']);
        return Ok(home.join(rest).to_string_lossy().into_owned());
    }
    if path.starts_with('~') {
        return Err(anyhow::anyhow!("'~user' expansion is not supported: '{path}'"));
    }
    Ok(path.to_string())
}

/// `~/.gemini/antigravity`, resolved. Mirrors DEFAULT_APP_DATA_DIR
/// (local_connection_config.py:35-37). Errors when the home directory is
/// unknown — there is deliberately no /tmp fallback.
pub fn default_app_data_dir() -> Result<std::path::PathBuf, anyhow::Error> {
    let home = home_dir().ok_or_else(|| anyhow::anyhow!(
        "cannot determine the home directory (HOME/USERPROFILE unset); \
         set app_data_dir explicitly to an absolute path"))?;
    let p = home.join(".gemini").join("antigravity");
    Ok(secure_normalize_path(&p.to_string_lossy()).unwrap_or(p))
}
```

The `unwrap_or(p)` is intentional: if the directory does not exist yet,
`secure_normalize_path` still succeeds (lexical tail), and the only way it
errors is a genuinely inaccessible ancestor, in which case the unresolved path
is a strictly better allow-list entry than aborting startup.

**Rewrite `src/agent.rs:270-282`** (using the `workspaces` binding from WI-1):

```rust
            if !workspaces.is_empty() {
                let app_data_dir = match self.config.app_data_dir.clone() {
                    Some(raw) => {
                        let expanded = crate::path_safety::expand_home(&raw)?;
                        if !std::path::Path::new(&expanded).is_absolute() {
                            return Err(anyhow!(
                                "app_data_dir must be an absolute path, got '{raw}'"));
                        }
                        crate::path_safety::secure_normalize_path(&expanded)
                            .map_or(expanded, |p| p.to_string_lossy().into_owned())
                    }
                    None => crate::path_safety::default_app_data_dir()?
                        .to_string_lossy().into_owned(),
                };
                let mut allowed_paths = workspaces.clone();
                allowed_paths.push(app_data_dir);
                let mut ws_policies = policy::workspace_only(allowed_paths);
                ws_policies.append(&mut final_policies);
                final_policies = ws_policies;
            }
```

The error message is copied verbatim from upstream (`:112`). The enclosing block
already returns `Result<_, anyhow::Error>` — `src/agent.rs:200` and `:287` both
`return Err(anyhow!(...))` — so `?` and `return Err` compile in place with no
signature change.

**Timing divergence to document:** upstream validates at pydantic-config time,
we validate at `Agent::start`; the crate has no config-validation hook, and
moving it into `AgentBuilder::app_data_dir` would make that setter fallible,
which is a bigger API change than this cluster warrants. Say so in
`docs/agent.md`.

**Strict-parity alternative for the `~` question:** upstream's validator runs
*before* any expansion, so `os.path.isabs("~/x")` is False and upstream actually
REJECTS `~/x`. Expanding first is a deliberate ergonomic divergence that cannot
widen the sandbox (it can only resolve to the real home directory). If you would
rather be bit-exact, delete the `expand_home` call and let `~/x` hit the
absolute-path error.

**Scope fence.** This item only changes the policy allow-list.
`HarnessConfig.app_data_dir` is currently populated from `save_dir`
(`src/local.rs:454,674`, `src/wasm.rs:310`) — that conflation is W18/WP-4. Do
not touch it here; note in the W18 ticket that `default_app_data_dir()` is the
function it should call.

**Call sites.** `src/path_safety.rs` (add the three fns) ·
`src/agent.rs:270-282` (the `HOME`-or-`/tmp` block at `:271-278` replaced) ·
`docs/agent.md` (the `app_data_dir` row; document the absolute-path requirement
and the new startup error) ·
`skills/google-antigravity-sdk-rust/references/safety_policies.md:63` (add that
`app_data_dir` joins the allow-list).

**Tests.**
- `src/path_safety.rs::tests::expand_home_expands_tilde` / `::rejects_tilde_user`
  / `::passes_through_absolute` — NEW. No upstream test exists (`expanduser` is
  stdlib); the contract is derived from `local_connection_config.py:123`.
- `::default_app_data_dir_errors_without_home` — NEW; **the security regression
  test.** Assert `Err`, and explicitly assert the message does **not** contain
  `/tmp`.
- `src/agent.rs::tests::rejects_relative_app_data_dir` — NEW; port of the
  assertion behind `_validate_app_data_dir` (`:109-113`): building an agent with
  `app_data_dir = "relative/dir"` fails with "must be an absolute path".
- `src/agent.rs::tests::app_data_dir_is_in_the_workspace_allowlist` — NEW; a
  path under the configured `app_data_dir` is allowed by the generated
  `workspace_only` policies. Pins `:122-127`.

**Testing env vars: `std::env::set_var` is `unsafe` in edition 2024 and the
crate sets `unsafe_code = "forbid"` (`Cargo.toml:42`), so you CANNOT call it.**
Test `home_dir`/`default_app_data_dir` by refactoring them onto a private
`fn app_data_dir_from(home: Option<PathBuf>)` that the tests drive directly, and
keep the env read in a one-line wrapper.

**Behaviour change.** With `HOME`/`USERPROFILE` unset, `Agent::start` now fails
with an actionable error instead of silently adding `/tmp/.gemini/antigravity`
to the workspace allow-list. A relative `app_data_dir` is now rejected at
startup instead of becoming a workspace root that matches nothing. A
`~`-prefixed `app_data_dir` is now expanded instead of being treated as a
literal directory named `~`.

**Risk.**
1. An agent that previously started with `HOME` unset now fails at
   `Agent::start` unless `app_data_dir` is set. That is intended (it is the
   vulnerable configuration), but it will surface in minimal containers and CI
   images that clear the environment. The error message names the fix.
2. `to_string_lossy()` mangles non-UTF-8 paths. `ToolCall::canonical_path` is a
   `String` (`src/types.rs:357`), so the whole policy layer is already
   UTF-8-only; a lossy home directory would fail the comparison and over-deny.
   Acceptable, worth a comment.
3. Interacts with WI-9 (the `has_allow_all` gate): the body is self-contained,
   so either ordering merges cleanly — just **do not re-introduce the
   `HOME`-or-`/tmp` line while resolving**.

---

### WI-5 (S12 + N6) — the `BuiltinTools` helper set and `CapabilitiesConfig` validation

**Defect (S12).** `src/types.rs:214-221` returns
`[FindFile, ListDir, ViewFile, SearchDir]`. Upstream 0.1.1 `types.py:249-262`
returns `[LIST_DIR, SEARCH_DIR, FIND_FILE, VIEW_FILE, FINISH]` — `FINISH` has
been a member since 0.1.1. Consequences: (a) `src/agent.rs:592-604`
`AgentBuilder::read_only()` emits `deny_all()` plus `allow()` per read-only
tool, so `FINISH` is denied — a `read_only()` agent cannot invoke the finish
tool, which is how structured output is produced (`src/agent.rs:557-560`);
(b) `src/agent.rs:239-240` computes
`has_write_tools = active_tools.iter().any(|t| !read_only.contains(t))`, so a
user restricting `enabled_tools` to read-only tools plus `FINISH` is still told
write tools are enabled.

**Defect (N6).** **Audit correction:** "Rust silently lets `enabled_tools` win"
is false for the `Agent` path — `src/agent.rs:199-203` already returns
`Err("enabled_tools and disabled_tools are mutually exclusive")` and
`tests/integration_tests.rs:86-104` pins it. The real gap is the
**direct-strategy path**, which `docs/connections.md:204-219` teaches by
publishing an exhaustive `WasmConnectionStrategy` struct literal.
`src/local.rs:610-623` computes `active_tools` as
`self.capabilities_config.enabled_tools.as_ref().map_or_else(|| {...disabled...},
|enabled| enabled.iter().copied().collect())` — when both are `Some`,
`enabled_tools` wins with **no diagnostic**. `src/wasm.rs:245-260` is identical.
Upstream validates at model construction (0.1.1 `types.py:366-372`,
`CapabilitiesConfig._check_mutually_exclusive`), so no code path can observe the
conflicting state. Secondary defect: the ten-element builtin list is hardcoded
**four** times (`src/agent.rs:208-219`, `src/agent.rs:222-233`,
`src/local.rs:597-608`, `src/wasm.rs:233-244`) — which is how `FINISH` went
missing from `read_only()` in the first place.

**`src/types.rs`, in `impl BuiltinTools` (after `read_only()` at `:213-221`):**

```rust
/// Returns a list of all safe, read-only tools.
///
/// Mirrors upstream `BuiltinTools.read_only()` (0.1.1 types.py:249-262).
/// Order matches upstream so `safe_defaults()` and `AgentBuilder::read_only()`
/// emit policies in upstream order, making future diffs against policy_test.py
/// trivial.
pub fn read_only() -> Vec<Self> {
    vec![Self::ListDir, Self::SearchDir, Self::FindFile, Self::ViewFile, Self::Finish]
}

/// Returns every builtin tool, in wire-config order.
pub fn all_tools() -> Vec<Self> {
    vec![Self::CreateFile, Self::EditFile, Self::FindFile, Self::ListDir,
         Self::RunCommand, Self::SearchDir, Self::ViewFile, Self::StartSubagent,
         Self::GenerateImage, Self::Finish]
}

/// Tools that perform file read/write/create operations.
/// Exact mirror of upstream `BuiltinTools.file_tools()`
/// (0.1.1 types.py:294-307, 0.1.9 types.py:274-287) — provided for parity when
/// porting a Python policy. NOTE: `workspace_only()` deliberately scopes a
/// superset; see `path_scoped_tools()`.
pub fn file_tools() -> Vec<Self> {
    vec![Self::ViewFile, Self::CreateFile, Self::EditFile]
}

/// Every built-in tool whose arguments carry a filesystem path, and which
/// therefore gets a `canonical_path` the workspace sandbox can enforce.
/// Superset of `file_tools()`: upstream leaves directory-scoped reads
/// unscoped, which lets `search_directory` return file *contents* from
/// anywhere on disk. This crate does not.
pub fn path_scoped_tools() -> Vec<Self> {
    vec![Self::ViewFile, Self::CreateFile, Self::EditFile,
         Self::FindFile, Self::ListDir, Self::SearchDir]
}
```

**Do NOT add `READ_URL_CONTENT`** to `read_only()`. It does not exist as a
`BuiltinTools` variant in 0.1.1 (`types.py:237-247` lists eleven members, no
`SEARCH_WEB`/`READ_URL_CONTENT`); it is introduced at 0.1.9 `types.py:222-223`
and added to `read_only()` at `:238`. That belongs to A13/W13 in the migration,
together with the `HarnessSideTools.read_url_content = 14` field. Adding the
variant now would make `HarnessSideTools` construction at `src/local.rs:626-659`
inconsistent and would emit a tool name the pinned harness does not know.

**`src/types.rs`, new `impl CapabilitiesConfig` after the struct (`:243`):**

```rust
impl CapabilitiesConfig {
    /// Mirrors upstream `CapabilitiesConfig._check_mutually_exclusive`
    /// (0.1.1 types.py:366-372).
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.enabled_tools.is_some() && self.disabled_tools.is_some() {
            return Err(anyhow::anyhow!(
                "enabled_tools and disabled_tools are mutually exclusive"
            ));
        }
        Ok(())
    }

    /// The set of builtin tools this configuration exposes to the model.
    pub fn active_tools(&self) -> Result<Vec<BuiltinTools>, anyhow::Error> {
        self.validate()?;
        Ok(match (&self.enabled_tools, &self.disabled_tools) {
            (Some(enabled), _) => enabled.clone(),
            (None, Some(disabled)) => BuiltinTools::all_tools()
                .into_iter().filter(|t| !disabled.contains(t)).collect(),
            (None, None) => BuiltinTools::all_tools(),
        })
    }
}
```

Keep the error substring `"mutually exclusive"` verbatim —
`tests/integration_tests.rs:104` asserts on it. `anyhow` is already a dependency
(`Cargo.toml:19`) but `src/types.rs` may not import it; use the fully-qualified
`anyhow::anyhow!`/`anyhow::Error` as written to avoid touching the import block.

**Collapse the four hardcoded lists.**

- `src/agent.rs:196-237` becomes:
  ```rust
  self.config.capabilities.validate()?;
  let active_tools = self.config.capabilities.active_tools()?;
  ```
  deleting `:208-219` and `:222-233`. `active_tools` is used at `:240` via
  `.iter().any(..)`, which works unchanged on `Vec<BuiltinTools>`.
- `src/local.rs:597-623` becomes:
  ```rust
  let active_tools: HashSet<BuiltinTools> =
      self.capabilities_config.active_tools()?.into_iter().collect();
  ```
  `LocalConnectionStrategy::connect` already returns
  `Result<_, anyhow::Error>`, so `?` type-checks; `HashSet` is already imported
  (`src/local.rs:610`). Delete the `all_tools` array at `:597-608`.
- `src/wasm.rs:233-260` — identical collapse. `WasmConnectionStrategy::connect`
  also returns `Result<WasmConnection, anyhow::Error>` (`src/wasm.rs:88`).

**Call sites.** `src/types.rs:213-221` (`read_only` body) · `src/types.rs:222`
(add `all_tools`, `file_tools`, `path_scoped_tools`) · `src/types.rs:243` (add
`impl CapabilitiesConfig`) · `src/agent.rs:196-237` (replace with two lines) ·
`src/agent.rs:239-240` (no code change; `has_write_tools` behaviour shifts —
verify) · `src/agent.rs:592-604` (`AgentBuilder::read_only()` — no code change;
it now also emits `allow("FINISH")`) · `src/local.rs:597-623` ·
`src/wasm.rs:233-260` · `docs/agent.md:170` (the `.read_only()` row lists four
tools — add FINISH) · `docs/agent.md:578` (comment
`// Only FindFile, ListDir, ViewFile, SearchDir`) · `docs/connections.md:204-219`
(note that `connect()` now validates) ·
`skills/google-antigravity-sdk-rust/references/built_in_tools.md` (read-only
table).

**Tests.**
- `src/types.rs::tests::test_read_only_matches_upstream` — asserts the exact
  five-element list. Ports 0.1.1 `types_test.py` (search `read_only`).
- `::file_tools_matches_upstream` — asserts exactly
  `[ViewFile, CreateFile, EditFile]`, pinning parity with `types.py:283-287`.
  **This is the regression guard against someone "fixing" it toward the audit's
  wrong claim.**
- `::test_capabilities_validate_rejects_both` — `Err` containing
  `"mutually exclusive"`. Direct port of `types.py:366-372`.
- `::test_active_tools_default_is_all` / `::test_active_tools_honours_disabled`
  / `::test_active_tools_honours_enabled` — pins the three branches and the
  ordering of `all_tools()`.
- `src/agent.rs::tests::test_read_only_builder_allows_finish` —
  `AgentBuilder::read_only()` produces a policy list in which
  `enforce(..)?.pre_tool_call(&make_tool_call("FINISH", json!({})))` returns
  `allow == true`. **This is the regression that matters:** before the fix,
  `deny_all()` swallows FINISH.
- `::test_has_write_tools_false_for_read_only_plus_finish` —
  `capabilities.enabled_tools = Some(BuiltinTools::read_only())` with empty
  policies and empty workspaces must start successfully.
- `tests/integration_tests.rs::test_agent_start_mutually_exclusive_capabilities`
  (existing, `:86-104`) — keep unchanged; it must still pass through the new
  code path.
- `tests/integration_tests.rs::test_strategy_connect_rejects_both_tool_lists` —
  NEW; construct a `LocalConnectionStrategy` directly with both fields set and
  assert `connect()` returns `Err`. **This is the case the Agent-level test
  never covered.**
- `src/local.rs` / `src/wasm.rs`: assert that `HarnessSideTools` flags derived
  from `active_tools()` match the pre-refactor values for the default config
  (guards the four-copy dedup).

**Behaviour change.** `.read_only()` agents can now call the finish tool, so
structured output works under a read-only policy where it previously could not.
`has_write_tools` becomes false for more capability configurations, meaning
`Agent::start()` succeeds for some configs that previously errored.
`LocalConnectionStrategy::connect()` and `WasmConnectionStrategy::connect()` now
return `Err` when both tool lists are set, instead of silently letting
`enabled_tools` win. Nothing that was allowed becomes denied.

**Risk.** S12 is purely loosening. The refactor risk is larger than the fix
risk: collapsing four hardcoded lists into `all_tools()` **must preserve the
exact ordering**, because `src/local.rs:626-659` derives ten `HarnessSideTools`
booleans from set membership (order-independent, safe) while `src/agent.rs:240`
computes `has_write_tools` from the same vector (also order-independent) — but
`Tool` list ordering elsewhere (T8) is order-sensitive, so keep the order
identical to today's. Confirm `?` is legal at the two strategy sites: both
`connect()` signatures already return `Result<_, anyhow::Error>`.
`AgentBuilder::read_only()` now emits five `allow()` policies rather than four;
no snapshot test asserts the count today.

---

### WI-6 (S7) — `workspace_only` scope: route through `path_scoped_tools()`, add `workspace_only_for`

**Defect.** `src/policy.rs:167-173` hardcodes a five-entry list
`["CREATE_FILE", "EDIT_FILE", "VIEW_FILE", "LIST_DIR", "SEARCH_DIR"]` inline,
with no counterpart on `BuiltinTools`. Two real problems: (a) `FIND_FILE` is
omitted although `src/local.rs:1312-1322` and `src/wasm.rs:1181-1191` set
`canonical_path` from its `directory_path`, so `find_file` can enumerate any
directory on the machine while `list_directory` cannot — an inconsistency with
no rationale; (b) there is no `BuiltinTools::file_tools()` for anyone porting a
Python policy or wanting upstream's exact scope. (b) is delivered by WI-5.

> **Contradiction between two reviewed specs — resolved here.**
> `path-safety/S7` says: keep `LIST_DIR`/`SEARCH_DIR`, add `FIND_FILE` (six
> tools). `policy-wiring/new-workspace-tool-scope` says the opposite: drop
> `LIST_DIR`/`SEARCH_DIR` and do **not** add `FIND_FILE` (three tools, upstream
> exact). Both agree on the underlying fact: `BuiltinTools.file_tools()` is
> `[VIEW_FILE, CREATE_FILE, EDIT_FILE]` identically at 0.1.1 `types.py:294-307`
> and 0.1.9 `types.py:274-287`. The disagreement is a policy judgement, not a
> factual one.
>
> **Recommendation: take S7's six-tool superset**, because S7's own design makes
> the other position a one-call opt-in — `workspace_only_for(&BuiltinTools::file_tools(), ws)`
> yields exactly the three-tool upstream scope. Reasons: (1) shipping a
> *loosening* (`list_directory`/`search_directory` outside the workspace become
> allowed) in the same release as the S1/S4 hardening makes the diff impossible
> to audit; (2) `search_directory` returns matching file *content* lines and
> `find_file`/`list_directory` enumerate the filesystem, so scoping them is
> defence in depth that costs nothing; (3) diverging *stricter* than upstream is
> defensible, diverging *looser* silently inside a security release is not.
>
> Counter-argument to record: WI-1 fixes the "we send `[]` to the harness"
> problem, so after WI-1 the harness *does* receive workspaces and may apply its
> own scoping, weakening argument (2). Revisit after WI-1 has been exercised
> against the real 0.1.1 binary; if you then decide to match upstream exactly,
> the change is one identifier (`path_scoped_tools` → `file_tools`) plus a
> CHANGELOG entry.

**`src/policy.rs:166-207`:**

```rust
pub fn workspace_only(workspaces: Vec<String>) -> Vec<Policy> {
    workspace_only_for(&crate::types::BuiltinTools::path_scoped_tools(), workspaces)
}

/// Same, over an explicit tool list. Use
/// `workspace_only_for(&BuiltinTools::file_tools(), ws)` for upstream-exact
/// scoping (0.1.1 types.py:294-307).
pub fn workspace_only_for(
    tools: &[crate::types::BuiltinTools],
    workspaces: Vec<String>,
) -> Vec<Policy> {
    let is_outside_workspace = move |tc: &ToolCall| -> bool { /* WI-3 body */ };
    let when_fn = Arc::new(is_outside_workspace);
    tools.iter()
        .map(|t| Policy::new(t.as_str().to_string(), Decision::Deny,
                             Some(when_fn.clone()), None,
                             "workspace_only".to_string()))
        .collect()
}
```

`Policy::new` takes `String` for `tool` (`src/policy.rs:55-69`) and
`BuiltinTools::as_str` is `const fn -> &'static str` (`src/types.rs:198`), so
`.as_str().to_string()` is the whole conversion. `workspace_only`'s public
signature is unchanged, so `src/agent.rs:279`, `docs/hooks.md:320` and
`skills/.../safety_policies.md:42` compile untouched. `workspace_only_for` is
additive. `Arc<dyn Fn..>` is cloned per tool exactly as today, so one predicate
instance is shared.

**Exact behaviour delta**, given a configured workspace and a path outside it:

| Tool | Before | After |
|---|---|---|
| `VIEW_FILE` | denied | denied |
| `CREATE_FILE` | denied | denied |
| `EDIT_FILE` | denied | denied |
| `LIST_DIR` | denied | denied *(diverges from upstream, which allows)* |
| `SEARCH_DIR` | denied | denied *(diverges from upstream, which allows)* |
| **`FIND_FILE`** | **ALLOWED** | **DENIED** |
| `RUN_COMMAND` / `START_SUBAGENT` / `GENERATE_IMAGE` / `FINISH` | unaffected | unaffected (no `canonical_path`) |

**Call sites.** `src/policy.rs:166-207` (split into `workspace_only` +
`workspace_only_for`; the five-element literal at `:167-173` deleted) ·
`src/agent.rs:279` (unchanged call — verify) · `docs/agent.md:117` and `:280` ·
`skills/google-antigravity-sdk-rust/references/safety_policies.md:42,63` ·
`skills/google-antigravity-sdk-rust/references/built_in_tools.md:9-18` (the
table has a "path?" column; make it agree with `path_scoped_tools()`).

**Tests.**
- `src/policy.rs::tests::test_workspace_only_scopes_path_carrying_tools` — NEW;
  asserts `workspace_only(vec![ws]).len() == 6` and that the tool names are
  exactly `path_scoped_tools()`.
- `::test_workspace_only_denies_find_file_outside` — NEW; **the one behaviour
  change.**
- `::test_workspace_only_allows_non_file_tools` — port of 0.1.9
  `policy_test.py:776-782` (`test_allows_non_file_tools`): `RUN_COMMAND` is
  unaffected.
- `::test_workspace_only_for_upstream_scope` — NEW;
  `workspace_only_for(&BuiltinTools::file_tools(), ws)` yields 3 policies and
  leaves `LIST_DIR` allowed, documenting the escape hatch.

**Behaviour change.** `FIND_FILE` targeting a directory outside every configured
workspace is now denied; previously allowed. Nothing else changes. New public
`policy::workspace_only_for()`.

**Risk.** The tool identifiers are this crate's invented UPPERCASE names
(`src/types.rs:200-210`); they match what `extract_builtin_tool_call` produces
(`src/local.rs:1304-1401`), so the policies still match end to end. **Do not
lowercase them here** — that is S8 and must land as one atomic change across
`policy.rs`, both extractors, every doc and `examples/policies.rs`. Denying
`FIND_FILE` is a real, if small, functional restriction: an agent that used
`find_file` to locate a file outside the workspace before opening it now fails
at the find step rather than the open step. Call it out in the CHANGELOG.

---

### WI-7 (S4) — Apply `workspace_only` unconditionally; `workspaces(vec![])` is the sole opt-out

**Defect.** `src/agent.rs:256-260` scans the policy list for a wildcard APPROVE
named `"allow_all"` — exactly what `policy::allow_all()` produces
(`src/policy.rs:112-120`) — and `:262` gates the entire workspace-policy block
on `!has_allow_all`. Upstream's `_apply_workspace_policies` is an unconditional
pydantic model-validator with no such exemption, and its docstring states the
opposite intent verbatim: *"Always prepends — even when the user sets explicit
policies — so that file operations are always restricted to the configured
workspaces. Users who want truly unrestricted access should set
`workspaces=[]`"* (0.1.1 `local_connection_config.py:133-138`). The class
docstring at `:38-46` explicitly presents `policies=[policy.allow_all()]` as the
sanctioned way to get autonomous shell access *while* file tools stay
workspace-restricted. Because `allow_all()` is what `src/lib.rs:29-31`,
`README.md:76,334,359,388` and eight examples recommend, the default Rust
posture is strictly less sandboxed than upstream.
`skills/.../references/safety_policies.md:66` documents the exemption as a
feature.

**Upstream.** 0.1.1 `local_connection_config.py:130-141` (unconditional, with
the `workspaces=[]` docstring) and `:38-46`; 0.1.9 `:114-133` (adds the
`name != "workspace_only"` idempotence filter and `normalize_wire_path` over the
workspaces).

**Rewrite `src/agent.rs:248-283`.** With WI-1, WI-2, WI-3 and WI-4 landed, the
block reduces to:

```rust
// Upstream local_connection_config.py:130-141 (0.1.1) / :112-133 (0.1.9):
// workspace scoping is applied unconditionally. The documented opt-out is
// `workspaces(vec![])`, not a policy name.
//
// Idempotence (0.1.9 :114-118): drop any workspace_only policies already in
// the list before prepending fresh ones, so re-application cannot stack
// duplicate DENY rules.
final_policies.retain(|p| p.name != "workspace_only");

if !workspaces.is_empty() {
    // ... WI-4's app_data_dir block, unchanged ...
}
```

Delete the `has_allow_all` scan at `:256-260` and the `if !has_allow_all {`
wrapper at `:262`. `final_policies` is already `let mut` (`:243`). The `retain`
runs **unconditionally**, matching 0.1.9's `else: self.__dict__["policies"] =
other_policies` branch, so `workspaces(vec![])` also strips a hand-passed
`workspace_only` group. That is upstream's semantics; call it out in the doc
comment.

**The two "MANDATORY CO-CHANGE" items in the original S4 spec are dead by
sequencing.** S4 as written required an "S16-lite" (resolve relative paths
against cwd) and an "S5-lite" (minimal `file://`/`cns://` handling) because
without them S4 denies far more than upstream. WI-3 delivers the full S16 and
WI-2 delivers the full S5, both strictly before this item. **Do not implement
either lite version** — they would be immediately superseded, and the lite
`normalize_wire_path` is wrong for percent-encoded and `cns://` paths.

**Opt-out.** No new API is required: `.workspaces(vec![])` already works,
because `crate::workspace::resolve` (WI-1) only falls back to cwd for `None`,
and the block is gated on `!workspaces.is_empty()`. **Do not invent a
`.no_workspace_sandbox()` flag** — upstream has exactly one opt-out and a second
one would drift.

**Call sites.** `src/agent.rs:248-283` (delete the `has_allow_all` scan;
unconditional prepend + `retain`) ·
`skills/google-antigravity-sdk-rust/references/safety_policies.md:63-66` (delete
the "Bypassing the Workspace Gate" paragraph) · `docs/agent.md:280` (step 4 of
the startup sequence: remove "unless `allow_all()` was used") ·
`docs/agent.md:117` (document `vec![]` as the opt-out) ·
`examples/agent_server/src/main.rs:323-331` (the comment explaining why
`.workspaces()` is skipped is now the wrong advice — switch to
`.workspaces(vec![])` with a note, or set a real workspace) ·
`src/policy.rs:111-120` (`allow_all` rustdoc: state that file tools stay
workspace-restricted, mirroring upstream's class docstring).

**Tests.**
- `tests/integration_tests.rs::test_allow_all_still_gets_workspace_policies` —
  `Agent::builder().allow_all()` with `workspaces(vec!["/allowed"])`; assert a
  `VIEW_FILE` on `/etc/passwd` is denied and one on `/allowed/x` is allowed.
  **Direct port of the upstream docstring contract at 0.1.1
  `local_connection_config.py:133-138`.**
- `::test_workspaces_empty_opts_out` — `.workspaces(vec![]).allow_all()` allows
  `/etc/passwd`.
- `::test_workspace_policies_are_idempotent` — passing
  `policy::workspace_only(vec!["/a"])` explicitly alongside
  `workspaces(vec!["/b"])` yields exactly one `workspace_only` group, scoped to
  `/b` + `app_data_dir`. Ports 0.1.9 `local_connection_config.py:114-118`.
- `src/policy.rs::tests::test_allows_relative_path_inside_cwd` — with
  `workspace_only(vec![cwd])`, a `VIEW_FILE` whose `canonical_path` is
  `"src/main.rs"` must be ALLOWED. Ports the resolve-based semantics of 0.1.1
  `policy_test.py:728-748`.
- `::test_normalizes_file_uri_before_workspace_check` —
  `canonical_path = "file:///allowed/workspace/x.rs"` with workspace
  `/allowed/workspace` must be ALLOWED. Ports 0.1.9
  `event_processor_test.py:33-52` and `hook_router_test.py:900-951`.
- Preserve `src/policy.rs:860-875` `test_workspace_only` and the
  `workspace-evil` prefix-attack case unchanged.

**Behaviour change — the largest DENY change in this plan.** Agents built with
`allow_all()` (or any policy set) now have file tools restricted to the
configured workspaces, defaulting to cwd plus the app-data dir. Previously
`allow_all()` disabled that restriction entirely. Migration: pass
`.workspaces(vec!["/path/one".into(), "/path/two".into()])` to widen the
sandbox, or `.workspaces(vec![])` to opt out completely (the upstream-sanctioned
escape). Passing your own `policy::workspace_only(..)` group no longer survives
— the SDK replaces it with one derived from `workspaces`.

**Risk.** Every `allow_all()` agent — the shape the README and every example
teach — gains six file-tool DENY rules scoped to cwd + `~/.gemini/antigravity`.
**Landing this before WI-2 and WI-3 turns a security fix into a bug report**
(every relative path and every `file://` path would be denied). Second hazard:
the unconditional `retain` silently discards a user's hand-built
`workspace_only(vec![custom])` group — upstream does exactly this (0.1.9), but
it is surprising, so it needs the doc note. Ship with a CHANGELOG entry headed
**"`allow_all()` no longer disables the workspace sandbox"**.

---

### WI-8 (S3 + S14) — Route through `policy::enforce()`; add the `has_mcp_servers` guard term

These two edit adjacent lines of `Agent::start` (`:285-295`). **Land them as
one commit.**

**Defect (S3).** `src/agent.rs:293` constructs the enforcer directly:
`let enforcer = Arc::new(PolicyEnforcer::new(final_policies, Vec::new()));`.
`policy::enforce()` (`src/policy.rs:222-254`) — which implements both upstream
fail-closed startup guards and derives `server_names` — is called only from
`src/policy.rs`'s own `#[cfg(test)]` block (`:632, :641, :651, …`).
Consequences: (a) `server_names` is permanently empty, so
`PolicyEnforcer::parse_mcp_tool` (`src/policy.rs:329-338`) always returns
`None`, `is_mcp` is always false, and every `server/tool` and `server/*` policy
is inert — a silently wrong security posture; (b) an ASK_USER policy with no
handler degrades from a startup `ValueError` to a runtime `Err` at
`src/policy.rs:457-460`, which then failed open via S2; and MCP policies
declared without `mcp_servers` no longer raise.

**Defect (S14).** `src/agent.rs:286-290` guards only on
`has_write_tools && final_policies.is_empty()`. Upstream's guard has three
terms: `(has_write_tools or has_mcp_servers) and not active_policies and not
has_tool_decide_hook`.

**Upstream.** 0.1.1 `agent.py:126-131`
(`self._hook_runner.register_hook(policy.enforce(active_policies, mcp_servers=self._config.mcp_servers))`);
identical at 0.1.9 `agent.py:102-107`. `enforce` itself: 0.1.1
`hooks/policy.py:853-904`. Guard: 0.1.1 `agent.py:112-124`, identical at 0.1.9
`agent.py:89-100`. Tests: 0.1.1 `policy_test.py:113-134`, `:963-971`,
`:981-999`.

**`src/agent.rs:285-290` (S14):**

```rust
// Upstream agent.py:112-124. NOTE: upstream's third term,
// `not has_tool_decide_hook`, is intentionally NOT ported — see below.
let has_mcp_servers = !self.config.mcp_servers.is_empty();
if (has_write_tools || has_mcp_servers) && final_policies.is_empty() {
    return Err(anyhow!(
        "Write tools or MCP servers are enabled without a safety policy. \
         Add policies=[policy.allow_all()] to approve all tool calls, \
         or policies=[policy.deny_all(), policy.allow(\"tool_name\")] \
         to selectively allow specific tools."
    ));
}
```

**Why the escape hatch is omitted — the load-bearing decision; record it in the
code comment.** Upstream's
`has_tool_decide_hook = bool(self._hook_runner.pre_tool_call_decide_hooks)` is
meaningful because its `HookRunner` keeps nine typed lists (0.1.1
`hook_runner.py:57-65`) populated by `isinstance` dispatch (`:122-135`), so
`pre_tool_call_decide_hooks` genuinely means "someone registered a gating hook".
Rust's `HookRunner` stores one untyped `Vec<Arc<dyn DynHook>>`
(`src/hooks.rs:206`) and `Hook::pre_tool_call` has a default no-op returning
`allow: true` (`src/hooks.rs:37-47`), so *every* registered hook is
indistinguishable from a decide hook. Approximating the hatch as
`!self.config.hooks.is_empty()` would let a purely observational logging hook
disable the safety guard — reintroducing precisely the fail-open this plan
exists to close. Defer to H3 (`fn declares(&self) -> HookKinds` with an empty
default); when H3 lands, the third term becomes
`&& !self.hook_runner.declares_any(HookKinds::PRE_TOOL_CALL).await`.

**Guard placement.** Leave the guard where it is (after the workspace prepend at
`:262-283`). That matches upstream, where `_apply_workspace_policies` is a
pydantic `model_validator(mode="after")` running at config construction, so
`Agent.__aenter__` already sees the prepended list (0.1.1
`local_connection_config.py:128-141`). It also means the guard is only reachable
when `workspaces` is empty — true on both sides, unchanged from today.

**`src/agent.rs:292-295` (S3):**

```rust
if !final_policies.is_empty() {
    // Upstream agent.py:105-110 — enforce() performs the fail-closed startup
    // validations and derives the MCP server names used for tool-name parsing.
    let enforcer = Arc::new(policy::enforce(
        final_policies,
        Some(self.config.mcp_servers.as_slice()),
    )?);
    self.hook_runner.register(enforcer).await;
}
```

Then delete the now-unused `PolicyEnforcer` import from `src/agent.rs:5`
(`use crate::policy::{self, Policy, PolicyEnforcer};` →
`use crate::policy::{self, Policy};`) — unused-import is a warning, but
`clippy::all` is `deny` at `Cargo.toml:48`, so leaving it fails CI.

**Compile notes.** `policy::enforce` returns
`Result<PolicyEnforcer, anyhow::Error>` and `Agent::start` returns
`BoxFuture<'static, Result<Agent<Started>, anyhow::Error>>`, so `?` type-checks.
Write `Some(self.config.mcp_servers.as_slice())` rather than
`Some(&self.config.mcp_servers)`; both compile, but the explicit slice is
unambiguous. Borrowck is clean: `final_policies` was cloned at `:243`, the
`&self.config.mcp_servers` borrow ends at the statement, and `self.config` is
not moved until `:337`/`:384`. `Arc<PolicyEnforcer>` still unsizes to
`Arc<dyn DynHook>` because `PolicyEnforcer` implements `Hook`
(`src/policy.rs:374`) and the blanket `impl<T: Hook + ?Sized> DynHook for T` at
`src/hooks.rs:150` applies. `enforce` treats `Some(&[])` and `None` identically
(`src/policy.rs:230` uses `is_none_or(<[_]>::is_empty)`), so the MCP
fail-closed guard still fires for a config with MCP policies and no servers.

**Sequencing note.** Land WI-5 (S12) **before** this item: after `read_only()`
gains `FINISH`, `has_write_tools` becomes false for more configurations, which
*loosens* the first guard term at the same time S14 tightens the second. Landing
S12 first makes both deltas visible in one CI run.

**Call sites.** `src/agent.rs:285-290` · `src/agent.rs:292-295` ·
`src/agent.rs:5` (drop the `PolicyEnforcer` import).

**Tests.**
- `tests/integration_tests.rs::test_start_rejects_ask_user_policy_without_handler`
  — `.policies(vec![Policy::new("RUN_COMMAND".into(), Decision::AskUser, None,
  None, "oops".into())])`; assert `start()` returns `Err` containing
  `"missing an ask_user handler"`. Ports 0.1.1 `policy_test.py:113-134`.
- `::test_start_rejects_mcp_policy_without_servers` —
  `.policies(deny_mcp(&server, None))` with `mcp_servers` empty must fail with
  `"'mcp_servers' was not"`. Ports 0.1.1 `policy_test.py:963-971`.
- `::test_start_populates_mcp_server_names` — register
  `McpServerConfig::Stdio{name:"math",..}` plus `deny_mcp(&s, None)` and
  `allow_all()`; drive a `mcp_math_add` `ToolCall` and assert it is denied. Ports
  0.1.1 `policy_test.py:981-999`. **Blocked on WP-2 for the mock's `toolCall`
  frame**; until then assert at the `policy::enforce` level, which
  `src/policy.rs:950-983` already covers.
- `::test_start_rejects_mcp_servers_without_policies` (S14) —
  `AgentConfig{ mcp_servers: vec![McpServerConfig::Stdio{..}], policies:
  Some(vec![]), workspaces: Some(vec![]), capabilities: CapabilitiesConfig{
  enabled_tools: Some(BuiltinTools::read_only()), .. }, .. }` must fail with
  `"Write tools or MCP servers"`. `workspaces: Some(vec![])` is required to keep
  the workspace prepend from making `final_policies` non-empty.
- `::test_start_allows_read_only_tools_without_policies` — same config minus
  `mcp_servers` must still start (regression guard on the `has_write_tools` term
  after WI-5).
- Keep `src/policy.rs::tests::test_enforce_fails_closed_on_missing_servers`
  (`:933-947`) and `::test_longest_match_mcp_parsing` (`:949-983`) — they now
  describe a live code path rather than a dead one.
- Update any existing assertion on the old message text:
  `"Write tools are enabled without a safety policy"` becomes
  `"Write tools or MCP servers are enabled without a safety policy"`.

**Behaviour change.** `Agent::start()` now returns `Err` for three
configurations that previously started: an ASK_USER policy with no handler; MCP
policies with no registered servers; MCP servers configured with an explicitly
empty policy list. MCP-targeted policies (`server/tool`, `server/*`) stop being
inert. Rust remains stricter than upstream in one respect: registering a gating
hook does not substitute for policies (documented, deferred to H3).

**Risk.** Configurations that previously started and then failed open now refuse
to start. Grep before landing: no in-tree example or test constructs either
shape (`examples/policies.rs:78-93` always pairs `Decision::AskUser` with a
handler), so the in-repo blast radius is zero. Second-order: `server_names`
becoming non-empty makes `parse_mcp_tool` succeed, which changes
`matches_target`'s `is_mcp` branch (`src/policy.rs:356-372`) for any tool
literally named `mcp_<registered-server>_<something>`. The S14 tightening is
narrow — it requires the user to have passed `policies(vec![])` explicitly, since
`src/agent.rs:243-246` defaults to `confirm_run_command(None)` when `policies` is
`None`. **Resist the temptation to add an approximate escape hatch.**

---

### WI-9 (S13) — `policy::safe_defaults(handler)`

**Defect.** `src/policy.rs` has `allow`, `deny`, `ask_user`, `allow_all`,
`deny_all`, `confirm_run_command` and `workspace_only`, but no `safe_defaults`.
Upstream has had it since 0.1.1 (`hooks/policy.py:371-384`) and it is the only
preset that gives a genuinely interactive, deny-by-omission posture; without it
the closest Rust equivalent is `confirm_run_command(Some(h))`, which asks only
about `RUN_COMMAND` and allows every file write silently.

**Add to `src/policy.rs` immediately after `allow_all()` (after `:120`),
matching upstream's file order:**

```rust
/// Creates a set of safe default policies.
///
/// Allows all read-only tools and asks the user for any other tool call.
/// Mirrors upstream `policy.safe_defaults` (0.1.1 policy.py:371-384).
pub fn safe_defaults(
    handler: impl Fn(&ToolCall) -> bool + Send + Sync + 'static,
) -> Vec<Policy> {
    let mut policies: Vec<Policy> = crate::types::BuiltinTools::read_only()
        .into_iter()
        .map(|t| allow(t.as_str()))
        .collect();
    policies.push(ask_user("*", handler));
    policies
}
```

**Compile notes.** `ask_user` (`src/policy.rs:101-109`) already takes
`impl Fn(&ToolCall) -> bool + Send + Sync + 'static` and boxes it into an `Arc`,
so a single by-value `handler` moved into the one `ask_user` call needs **no
`Clone` bound** — unlike `ask_user_mcp` (`:487-498`), which requires `+ Clone`
because it fans the handler out across N policies. Names are left empty by
`allow()`/`ask_user()`, matching upstream, so deny messages fall back to the tool
selector via `src/policy.rs:409`. Add `crate::types::BuiltinTools` to the
`use crate::types::{...}` line at `src/policy.rs:8`.

**Bucketing check:** the read-only `allow`s land in `LEVEL_SPECIFIC_ALLOW`
(bucket 2) and the wildcard `ask_user` in `LEVEL_GLOBAL_ASK` (bucket 7) via
`bucket_index` (`src/policy.rs:289-309`), so specific-allow correctly beats
global-ask — exactly the precedence 0.1.1 `policy_test.py:611-645` asserts.

**Call sites.** `src/policy.rs:120` (insert after `allow_all`) ·
`src/policy.rs:8` (add `BuiltinTools` to the types import) ·
`docs/hooks.md:305-325` (policy builder catalogue) ·
`skills/google-antigravity-sdk-rust/references/safety_policies.md` (builder
list) · `README.md` policy section.

**Tests.**
- `::test_safe_defaults_allows_read_only_tools` — direct port of 0.1.1
  `policy_test.py:611-629`: build `enforce(safe_defaults(|_| false), None)` and
  assert `allow == true` for each of `LIST_DIR`, `SEARCH_DIR`, `FIND_FILE`,
  `VIEW_FILE`, `FINISH`. (Upstream iterates the lowercase names; use the crate's
  current uppercase identifiers until S8 lands.)
- `::test_safe_defaults_asks_for_other_tools` — direct port of `:631-645`: a
  handler returning `true` is invoked exactly once for `RUN_COMMAND` and the call
  is allowed.
- `::test_safe_defaults_denies_when_handler_refuses` — handler returns `false` →
  `allow == false` and `message.contains("User denied")`, matching
  `src/policy.rs:438-446`.

**Behaviour change.** None for existing users — a new opt-in builder.

**Risk.** Additive. The one real coupling is WI-5: **without `FINISH` in
`read_only()`, `safe_defaults()` routes the finish tool through the ASK_USER
handler on every single turn** — highly visible and clearly wrong. Do not land
this before WI-5. Because the wildcard is ASK_USER rather than DENY, a
`safe_defaults` agent still satisfies the `!final_policies.is_empty()` startup
guard, which is correct.

---

### WI-10 (S15 + N8) — MCP builder options, `approve_` auto-name, and `IntoPolicies`

Two independent additive `src/policy.rs` changes; grouped because they touch
adjacent regions of the same file and neither is worth its own release note.

#### S15 — `when`/`name` options and the auto-name prefix

**Defect.** `src/policy.rs:477-498` exposes `allow_mcp(server, tools)`,
`deny_mcp(server, tools)` and `ask_user_mcp(server, tools, handler)` with no way
to attach a predicate or a custom name, while upstream's `_mcp_policies` takes
both (0.1.1 `policy.py:135-201`, keyword-only `when` and `name`). Auto-generated
names also diverge: `decision_label` at `src/policy.rs:533-539` maps
`Decision::Approve` to `"allow"`, producing `allow_math_calc`, where upstream
uses `decision.value.lower()` = `"approve"`, producing `approve_math_calc`
(pinned by 0.1.1 `policy_test.py:923`). Upstream's custom-name rule also differs:
with `name="custom"` it produces `custom_calc` per tool (`policy.py:189-191`,
pinned at `policy_test.py:945-949`) and `custom` alone for the server-wide
wildcard (`:161`).

> **Audit correction.** The audit calls the naming difference "user-visible in
> deny messages". It is not. Approve policies return
> `HookResult{allow: true, message: String::new()}` (`src/policy.rs:417-422`), so
> an APPROVE policy's name never reaches any user-facing string; DENY (`"deny"`)
> and ASK_USER (`"ask_user"`) labels already match upstream exactly. The rename
> affects `tracing` output, the `Debug` impl at `src/policy.rs:41-51`, and the
> upstream test assertion — nothing more. **Fix it for parity, not severity.**

**(1) `src/policy.rs:533-539`:**

```rust
const fn decision_label(d: Decision) -> &'static str {
    match d {
        Decision::Approve => "approve",   // was "allow" — Decision.APPROVE.value.lower()
        Decision::Deny => "deny",
        Decision::AskUser => "ask_user",
    }
}
```

**(2) Options struct plus three additive `_with` constructors** (Rust has no
keyword arguments, so widening the existing three signatures would break every
caller). Insert before `allow_mcp` (`:477`):

```rust
/// Optional refinements for the MCP policy builders, mirroring upstream's
/// keyword-only `when` / `name` parameters (0.1.1 policy.py:135-146).
#[derive(Clone, Default)]
pub struct McpPolicyOptions {
    /// Predicate gating whether the policy applies to a given call.
    pub when: Option<Arc<dyn Fn(&ToolCall) -> bool + Send + Sync>>,
    /// Base name; per-tool policies get `"{name}_{tool}"`, the server-wide
    /// wildcard gets `"{name}"` verbatim (upstream policy.py:161, :189-191).
    pub name: Option<String>,
}

impl std::fmt::Debug for McpPolicyOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpPolicyOptions")
            .field("when_is_some", &self.when.is_some())
            .field("name", &self.name)
            .finish()
    }
}

pub fn allow_mcp_with(server: &McpServerConfig, tools: Option<&[&str]>,
                      opts: McpPolicyOptions) -> Vec<Policy>;
pub fn deny_mcp_with(server: &McpServerConfig, tools: Option<&[&str]>,
                     opts: McpPolicyOptions) -> Vec<Policy>;
pub fn ask_user_mcp_with(server: &McpServerConfig, tools: Option<&[&str]>,
                         handler: impl Fn(&ToolCall) -> bool + Send + Sync + 'static + Clone,
                         opts: McpPolicyOptions) -> Vec<Policy>;
```

A manual `Debug` impl is **required**: `Cargo.toml:43` sets
`missing_debug_implementations = "warn"` and `Arc<dyn Fn..>` has no derive — copy
the shape already used for `Policy` at `src/policy.rs:41-51`.

**(3) Widen the private helper (`:501-531`)** and route the three existing public
functions through it with `McpPolicyOptions::default()`:

```rust
fn mcp_policies(
    server_name: &str,
    decision: Decision,
    tools: Option<&[&str]>,
    handler: Option<Arc<dyn Fn(&ToolCall) -> bool + Send + Sync>>,
    opts: &McpPolicyOptions,
) -> Vec<Policy> {
    match tools {
        None => vec![Policy::new(
            format!("{server_name}/*"),
            decision,
            opts.when.clone(),
            handler,
            opts.name.clone().unwrap_or_else(
                || format!("{}_{server_name}_all", decision_label(decision))),
        )],
        Some(tools) => tools.iter().map(|t| Policy::new(
            format!("{server_name}/{t}"),
            decision,
            opts.when.clone(),
            handler.clone(),
            opts.name.as_ref().map_or_else(
                || format!("{}_{server_name}_{t}", decision_label(decision)),
                |n| format!("{n}_{t}"),
            ),
        )).collect(),
    }
}
```

The existing three public functions keep their signatures and pass
`&McpPolicyOptions::default()` — fully source-compatible.

Upstream's two `ValueError` guards (`mcp_tools` given for a plain string tool
name; `mcp_tools` passed as a bare string — `policy.py:161-166`,
`policy_test.py:951-962`) are already unrepresentable in Rust: `allow(&str)` and
`allow_mcp(&McpServerConfig, Option<&[&str]>)` are distinct functions with
distinct types. **Nothing to port.**

#### N8 — `IntoPolicies`

**Defect.** Every group builder returns `Vec<Policy>` (`confirm_run_command` at
`:134-163`, `workspace_only` at `:166-207`, the three MCP builders at
`:477-498`, and `safe_defaults` from WI-9), while `AgentBuilder::policies` takes
a flat `Vec<Policy>` (`src/agent.rs:574`) and `AgentBuilder::policy` takes a
single `Policy` (`:537`). There is no way to write the upstream idiom
`policies=[policy.allow("read_file"), policy.allow(mcp_cfg)]`, which mixes
singles and groups — upstream flattens it in `_flatten_policies`
(0.1.1 `policy.py:817-851`), called from `enforce` (`:892`).

A heterogeneous `vec![]` is impossible in Rust, so the faithful port is a
conversion trait plus an accumulating builder method.

**(1) `src/policy.rs` (~`:540`):**

```rust
/// Conversion allowing a single [`Policy`] or a group of them to be passed
/// interchangeably wherever policies are accepted. The Rust analogue of
/// upstream `_flatten_policies` (0.1.1 policy.py:817-851).
pub trait IntoPolicies {
    fn into_policies(self) -> Vec<Policy>;
}
impl IntoPolicies for Policy {
    fn into_policies(self) -> Vec<Policy> { vec![self] }
}
impl IntoPolicies for Vec<Policy> {
    fn into_policies(self) -> Vec<Policy> { self }
}
impl<const N: usize> IntoPolicies for [Policy; N] {
    fn into_policies(self) -> Vec<Policy> { self.into_iter().collect() }
}
```

**(2) `src/policy.rs:222-225` — widen `enforce`:**

- Before: `pub fn enforce(policies: Vec<Policy>, mcp_servers: Option<&[McpServerConfig]>) -> Result<PolicyEnforcer, anyhow::Error>`
- After: `pub fn enforce(policies: impl IntoPolicies, mcp_servers: Option<&[McpServerConfig]>) -> Result<PolicyEnforcer, anyhow::Error>` with
  `let policies = policies.into_policies();` as the first line.

All twenty existing `enforce(vec![...], None)` call sites in `src/policy.rs`'s
test module keep compiling.

**(3) `src/agent.rs:537-545` and `:574-581`:**

```rust
pub fn policy(mut self, policy: impl crate::policy::IntoPolicies)
    -> AgentBuilder<HasPolicies>
{
    let mut policies = self.config.policies.take().unwrap_or_default();
    policies.extend(policy.into_policies());
    self.config.policies = Some(policies);
    AgentBuilder { config: self.config, _policy_marker: std::marker::PhantomData }
}

pub fn policies(self, policies: impl crate::policy::IntoPolicies)
    -> AgentBuilder<HasPolicies>
{
    let mut config = self.config;
    config.policies = Some(policies.into_policies());
    AgentBuilder { config, _policy_marker: std::marker::PhantomData }
}
```

`policy()` becoming accumulating-and-group-accepting is what delivers the
upstream ergonomics:
`.policy(policy::confirm_run_command(None)).policy(policy::allow_mcp(&s, None)).policy(policy::allow_all())`.

**Compile notes** (verified by building these exact impls, rustc 1.94.1, edition
2024). The orphan rule permits `impl IntoPolicies for Vec<Policy>` (local trait)
trivially. `.policies(vec![])` still infers, because `Vec<?T>: IntoPolicies` has
exactly one applicable impl. The `[Policy; N]` impl is what makes
`.policies([deny_all(), allow("VIEW_FILE")])` work without `vec!`. Two hazards:
(a) `.policy(x)` where `x: Vec<Policy>` used to be a type error and now compiles
— document that it *appends the whole group*; (b) taking `AgentBuilder::policies`
as a function pointer no longer works (nothing in-tree does this).

**Call sites.** `src/policy.rs:533-539` (`decision_label`) · `:477-498` (three
public builder bodies) · `:501-531` (helper signature + body) · `:477` (insert
`McpPolicyOptions`) · `:540` (add `IntoPolicies` + three impls) · `:222-225`
(`enforce` signature + first line) · `src/agent.rs:537-545` ·
`src/agent.rs:574-581` · `examples/custom_tools.rs:149-152`,
`examples/structured_output.rs:95-96`, `examples/policies.rs:65-93` (unchanged —
verify they still compile, which is the point) · `docs/mcp.md` (add the `_with`
variants and correct the auto-name examples) ·
`skills/google-antigravity-sdk-rust/examples/getting_started/mcp_tools.md` ·
`docs/agent.md:169-170` (builder method table) · `docs/hooks.md:305-380`
(composition examples).

**Tests.**
- `::test_allow_mcp_builder_specific_tools_name` —
  `allow_mcp(&server, Some(&["calc", "multiply"]))` yields tools `math/calc`,
  `math/multiply` and `policies[0].name == "approve_math_calc"`. Direct port of
  0.1.1 `policy_test.py:915-923`.
- `::test_allow_mcp_builder_wildcard_name` — `allow_mcp(&server, None)` yields
  one policy `math/*` named `approve_math_all`. Ports `:907-914` plus the
  `assertIn("math_advanced_all", result.message)` assertion at `:993`.
- `::test_mcp_builder_custom_name_unique` —
  `allow_mcp_with(&server, Some(&["calc"]), McpPolicyOptions{ name:
  Some("custom".into()), ..Default::default() })` yields name `"custom_calc"`.
  Direct port of `:945-949`.
- `::test_mcp_builder_when_predicate_skips` — a
  `deny_mcp_with(.., when: Some(|tc| false))` policy does not match, so a
  trailing `allow_all()` wins.
- `::test_enforce_flattens_nested_policies` — `enforce(confirm_run_command(None),
  None)` and `enforce(vec![allow("read_file")], None)` both build. Direct port of
  0.1.1 `policy_test.py:972-980`.
- `src/agent.rs::tests::test_builder_policy_accepts_group` —
  `.policy(policy::workspace_only(vec!["/a".into()])).policy(policy::allow_all())`
  yields the workspace group then `allow_all`, proving order is preserved across
  group boundaries.
- `::test_builder_policy_accepts_single` — the pre-existing single-`Policy` form
  still compiles and appends one entry (regression guard).
- `::test_into_policies_array_form` — `[deny_all(), allow_all()].into_policies().len() == 2`.
- Existing `::test_prefix_wildcard_matches_mcp_tool` (`:877-903`) and
  `::test_specific_allow_beats_prefix_deny` (`:905-931`) must keep passing
  unchanged.
- Upstream's two negative `_flatten_policies` cases (`policy_test.py:1017-1039`,
  non-Policy elements in a nested sequence) are unrepresentable in Rust's type
  system and need no port — record that in a comment.

**Behaviour change.** No decision changes. APPROVE-policy auto-names change from
`allow_*` to `approve_*` in logs and `Debug` output. `.policy()` gains the
ability to accept a group and now appends rather than pushes a single item — for
existing single-`Policy` callers the observable result is identical. Three new
opt-in `_with` constructors.

**Risk.** `impl Trait` in argument position is technically breaking for exotic
callers (function-pointer coercion, turbofish), so ship in a minor bump.
Inference for `.policies(vec![])` was verified to still work, but adding a fourth
`IntoPolicies` impl later over another generic container could break it — **keep
the impl set closed to `Policy`, `Vec<Policy>`, `[Policy; N]`**. Do **not** add a
blanket `impl<I: IntoIterator<Item = Policy>> IntoPolicies for I`: it would
destroy inference for `vec![]`. `opts.when.clone()` must be an `Arc` clone shared
across the per-tool policies, matching how `handler.clone()` is already used at
`src/policy.rs:525`.

---

### WI-11 (C5) — `StepTracker` must clear `handled_requests` when a step leaves WAITING_FOR_USER

**Defect.** `src/local.rs:1276-1291` — `StepTracker::update_state` is
`pub const fn update_state(&mut self, state: i32) { self.state = state; }`. It
never clears `handled_requests`. `mark_handled` (`:1284-1290`) inserts the
request name and returns `false` forever after. The reader loop calls
`update_state` then `mark_handled("questions_request")` /
`mark_handled("tool_confirmation_request")` at `src/local.rs:768-780` and gates
the response dispatch on the returned bool (`:927` `is_questions_new`, `:999`
`is_tool_conf_new`). So for a step that goes WAITING_FOR_USER → ACTIVE →
WAITING_FOR_USER on the **same** `(trajectory_id, step_index)` — the harness
re-asking a question, or re-requesting confirmation after the model retries — the
second request returns `false`, no `UserQuestionsResponse` / `ToolConfirmation`
is ever sent, and **the harness blocks forever waiting for an answer**.
`src/wasm.rs:43-66` is a byte-identical copy with the same bug.

**Upstream.** 0.1.1 `local_connection.py:54-85` (`_StepTracker`; `update_state`
at `:61-69` does the clear, `mark_handled` at `:71-85`); test at
`local_connection_test.py:743-805`. 0.1.9: `event_processor.py:105-129`.

**In BOTH `src/local.rs:1276-1291` and `src/wasm.rs:50-66`.** Note `const fn`
must go — `HashSet::clear` is not const:

```rust
/// Step state values from `StepUpdate.State` (proto/localharness.proto).
const STATE_WAITING_FOR_USER: i32 = 3;

impl StepTracker {
    /// Updates the tracked step status and clears the handled-request set when
    /// the step leaves WAITING_FOR_USER, so a re-asked question or a re-issued
    /// tool-confirmation request on the same step is answered again.
    /// Mirrors `local_connection.py:61-69` (0.1.1) /
    /// `event_processor.py:116-123` (0.1.9).
    pub fn update_state(&mut self, state: i32) {
        if self.state == STATE_WAITING_FOR_USER && state != STATE_WAITING_FOR_USER {
            self.handled_requests.clear();
        }
        self.state = state;
    }
}
```

Keep `mark_handled`'s extra `self.state == 3` guard (`src/local.rs:1285` /
`src/wasm.rs:60`) as-is. Upstream's `mark_handled` has no state guard
(`local_connection.py:71-85`), but the Rust guard is *stricter*, is already
pinned by the wasm unit test at `src/wasm.rs:1311-1324`, and is compatible with
upstream's own regression test: the reader loop calls `update_state(3)` before
`mark_handled` (`src/local.rs:768-780`), so on re-entry to WAITING the guard is
satisfied. **Add a `// Divergence from upstream: ...` comment rather than
silently changing two behaviours in one patch.**

Also replace the magic `3` at `src/local.rs:1285` / `src/wasm.rs:60` with
`STATE_WAITING_FOR_USER`, and let the `Some(3) => StepStatus::WaitingForUser` arm
at `src/local.rs:822` reference the same const.

**Call sites.** `src/local.rs:1276-1291` · `src/local.rs:1285` ·
`src/wasm.rs:50-66` (note wasm's `StepTracker` also has `new()`/`Default`) ·
`src/wasm.rs:60` · `src/wasm.rs:1311-1324` (existing unit tests — extend, do not
delete).

**Tests.**
- `step_tracker_clears_handled_requests_on_leaving_waiting` — port of 0.1.1
  `local_connection_test.py:743-805`
  (`test_state_transition_clears_handled_requests`, docstring: "Verifies WAITING
  -> ACTIVE -> WAITING transitions re-trigger handlers"; it asserts
  `hook_instance.call_count == 2` and `len(harness.ws.sent_messages) == 2`).
  Rust unit form:
  `update_state(3); assert!(mark_handled("tool_confirmation_request"));
  assert!(!mark_handled("tool_confirmation_request")); update_state(1);
  update_state(3); assert!(mark_handled("tool_confirmation_request"));`
  Add to `src/wasm.rs` next to `:1316-1325` **and** create a
  `#[cfg(test)] mod tests` in `src/local.rs`, which has no `StepTracker` unit
  tests today.
- `step_tracker_does_not_clear_while_still_waiting` — two consecutive
  `update_state(3)` calls must NOT clear (this is the dedup the tracker exists
  for).
- Keep the existing `src/wasm.rs:1310-1325` assertions (`mark_handled` false when
  state != 3) — they pin the intentional divergence.
- Optional end-to-end (S): extend `src/bin/mock_localharness.rs` to emit
  WAITING(questions_request) → ACTIVE → WAITING(questions_request) on step 1 and
  assert two `questionResponse` frames arrive.

**Behaviour change.** A question or tool-confirmation request re-issued on a step
that already had one answered is now answered again instead of being silently
swallowed. No user-visible API change; turns that previously hung now complete.

**Risk.** Very low. The only way to regress is to also drop the `state == 3`
guard in `mark_handled` in the same patch: that would make a request arriving in
a non-WAITING state dispatch a response, which the reader loop is not written
for. `update_state` loses `const`, so any caller in a const context breaks — the
only callers are `src/local.rs:769` and `src/wasm.rs:410`, both in `async fn`
bodies. `StepTracker` is `pub` and re-exported through `pub mod local` /
`pub mod wasm`, so removing `const` is technically a public-API change with no
practical impact.

---

### WI-12 (C2) — A freshly connected connection must report `is_idle == true`

**Defect.** `src/local.rs:699` `let is_idle = Arc::new(AtomicBool::new(false));`
and `:700` `let parent_idle = Arc::new(Mutex::new(false));`.
`src/wasm.rs:350-351` is identical. Upstream sets both idle at construction:
`self._is_idle = asyncio.Event(); self._is_idle.set()`
(`local_connection.py:448-449`) and `self._parent_idle = True` (`:459`). Two
consequences: (a) `Conversation::is_idle()` sees a never-used connection as busy;
(b) **it is the landmine under WI-13** — the drain condition is
`!conn.is_idle()`, so with the current initial value the FIRST `send()` on a
fresh connection would try to drain a turn that never ran and block forever on
`rx.recv()` at `src/local.rs:133`.

**Change.** `src/local.rs:699-700`:

```rust
// Upstream constructs the connection idle: local_connection.py:448-449
// (`self._is_idle = asyncio.Event(); self._is_idle.set()`) and :459
// (`self._parent_idle = True`). `send()` clears both (:503-504), so the
// only window this affects is before the first send.
let is_idle = Arc::new(AtomicBool::new(true));
let parent_idle = Arc::new(Mutex::new(true));
```

`src/wasm.rs:350-351`: identical two-line change. Nothing else moves.

**Verify the invariant that makes this safe.** `LocalConnection::send`
(`src/local.rs:159-164`) stores `is_idle = false` and `parent_idle = false`
*before* returning, and `Conversation::chat` awaits `send()` before constructing
the chunk stream (`src/conversation.rs:282-283`), so there is no window in which
`receive_steps()` is polled with a stale `is_idle == true` during a live turn.
Confirm the same for `src/wasm.rs:1007-1012`. Also verify the sentinel still
fires for turn 1: `TrajectoryStateUpdate(IDLE)` →
`!conn_is_idle.swap(true, SeqCst)` at `src/local.rs:1057` / `src/wasm.rs:696`.
Because `send()` set it to false, `swap` returns false and the sentinel IS sent.

**Call sites.** `src/local.rs:699` · `src/local.rs:700` · `src/wasm.rs:350` ·
`src/wasm.rs:351` · `src/wasm.rs:1535-1631` (the in-file mock integration test
**must** be restructured — see tests) · `docs/connections.md` / `ARCHITECTURE.md`
— any statement that a new connection starts busy.

**Tests.**
- `src/wasm.rs::test_wasm_connection_integration_mock` (`:1535-1631`) currently
  calls `conn.receive_steps()` immediately after `connect()` with no `send()`.
  Under C2 that stream early-returns `None` on its first poll
  (`src/wasm.rs:948-957` `rx.is_empty() && is_idle`) and the test fails.
  **Restructure it to the real protocol order**, which also removes a latent race
  with the queue purge in `send()` (`:1017-1022`): server reads the init frame →
  server reads the user-input frame → server sends `trajectoryStateUpdate
  STATE_RUNNING`, the `stepUpdate`, then `trajectoryStateUpdate STATE_IDLE`.
  Client: `connect()` → `assert!(conn.is_idle())` (the new C2 assertion) →
  `conn.send("hello").await` → `receive_steps()` yields the step → next is
  `None`. This mirrors `src/bin/mock_localharness.rs:73-84`, which already waits
  for the prompt before emitting steps — which is why
  `tests/integration_tests.rs` is unaffected by C2.
- Native equivalent: add to `tests/integration_tests.rs` a check that
  `agent.conversation().is_idle()` is true after `start()` and before the first
  `chat()`.
- Upstream has no direct unit test for the initial value; the closest pins are
  `local_connection_test.py:2660` (`assertFalse(harness.conn._is_idle.is_set())`
  after send) and `:77,:102,:173,:203,:898,:950,:3299` which all *explicitly*
  `_is_idle.clear()` in setup **precisely because the default is set**. Cite
  `local_connection.py:448-449` in the test comment.

**Behaviour change.** `Connection::is_idle()` / `Conversation::is_idle()` return
true on a fresh connection instead of false. `receive_steps()` called before any
`send()` now ends immediately instead of blocking. Migration note: always
`send()` before subscribing — this is the documented upstream order.

**Risk.** Any caller that polls `receive_steps()` before the first `send()` now
gets an immediately-terminating stream instead of a blocking one. That is
upstream's contract (`local_connection.py:550-551` returns immediately when idle
and the queue is empty), but it is a real behaviour change for the one in-tree
caller that does it (the wasm mock test) and potentially for user code that opens
a stream first and sends later. Call it out in the changelog.
`examples/agent_server/src/main.rs:481-493` already sends before
`receive_steps()`, so it is safe.

---

### WI-13 (A5) — `Conversation::send` must drain the previous turn into history

**Depends on WI-12.** Shipping this against `is_idle == false`-at-connect
deadlocks the very first send.

**Defect.** `src/conversation.rs:140-152` — `send()` opens with
`if !self.conn.is_idle() { /* comment only */ }`, an **empty block**. Two things
then go wrong on back-to-back sends: (1) `state.turn_start_indices.push(len)` at
`:147-148` records a boundary at the current history length, but turn 1's steps
were never recorded, so every subsequent turn boundary is wrong;
(2) `state.turn_usage = None` at `:149` discards turn 1's usage before anyone
could read it, and `LocalConnection::send` then hard-purges the channel
(`while rx.try_recv().is_ok() {}`, `src/local.rs:169-174` and
`src/wasm.rs:1017-1022`) so turn 1's steps are **destroyed** rather than
recorded. Upstream drains:
`if self._connection.is_idle is False: try: async for _ in self.receive_steps():
pass; except RuntimeError: await self._connection.wait_for_idle()`
(`conversation.py:125-134` in 0.1.1; `:122-131` in 0.1.9), with the RuntimeError
arm covering the case where another coroutine already owns the iterator.

**All changes in `src/conversation.rs`.**

**(1) Add a receiving flag** to the struct (`:37-41`) and to `Conversation::new`
(`:57-69`). The `new` signature is **unchanged**:

```rust
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Conversation {
    conn: AnyConnection,
    max_history_size: usize,
    state: Arc<Mutex<ConversationState>>,
    /// True while a `receive_steps()` stream returned by this Conversation is
    /// still alive. Stands in for Python's "async generator already running"
    /// RuntimeError (conversation.py:129-134).
    receiving: Arc<AtomicBool>,
}
```

**(2) Guard type** (private, module level):

```rust
struct ReceivingGuard(Arc<AtomicBool>);
impl Drop for ReceivingGuard {
    fn drop(&mut self) { self.0.store(false, Ordering::SeqCst); }
}
```

**(3) `receive_steps` (`:155-228`)** — set the flag eagerly and tie its release
to the returned stream's lifetime. Because `Then<..>` is not `Unpin` (its pending
future is an async block), **box first and then unfold**, which is the only form
that both keeps the guard alive and compiles:

```rust
pub fn receive_steps(&self) -> BoxStream<'static, Result<Step, anyhow::Error>> {
    let conn_stream = self.conn.receive_steps();
    let state = self.state.clone();
    let max_history = self.max_history_size;
    self.receiving.store(true, Ordering::SeqCst);
    let guard = ReceivingGuard(self.receiving.clone());

    let inner = conn_stream
        .then(move |step_res| { /* body unchanged, src/conversation.rs:161-226 */ })
        .boxed();                                   // BoxStream is Unpin

    stream::unfold((inner, guard), |(mut s, guard)| async move {
        s.next().await.map(|item| (item, (s, guard)))
    })
    .boxed()
}
```

`stream` and `StreamExt` are already imported at `src/conversation.rs:10-11`.
`ReceivingGuard` is `Send` because `Arc<AtomicBool>` is, so the stream stays
`BoxStream<'static, _>: Send`.

**(4) `send` (`:140-152`):**

```rust
/// Poll interval for the concurrent-iterator fallback. Replace with
/// `Connection::wait_for_idle()` when A6 lands.
const IDLE_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(10);

pub async fn send(&self, prompt: &str) -> Result<(), anyhow::Error> {
    // Mirrors conversation.py:125-134 (0.1.1) / :122-131 (0.1.9).
    if !self.conn.is_idle() {
        if self.receiving.load(Ordering::SeqCst) {
            // Another receive_steps() stream is live and is already recording
            // steps into history; draining here would steal its steps, so wait
            // for the turn to end instead (upstream's wait_for_idle() arm).
            while !self.conn.is_idle() {
                tokio::time::sleep(IDLE_POLL_INTERVAL).await;
            }
        } else {
            let mut drain = self.receive_steps();
            while let Some(res) = drain.next().await {
                res?;   // upstream lets the drained turn's exception propagate
            }
        }
    }
    let mut state = self.state.lock().await;   // MUST stay after the drain
    let len = state.steps.len();
    state.turn_start_indices.push(len);
    state.turn_usage = None;
    drop(state);
    self.conn.send(prompt).await
}
```

**Ordering is load-bearing:** the drain's per-step closure locks `self.state`, so
acquiring the state lock before the drain self-deadlocks. On the `res?` early
return, `drain` is dropped, which drops the guard — no manual unwind handling
needed.

**(5) Leave the channel purge** in `LocalConnection::send` /
`WasmConnection::send` (`src/local.rs:169-174`, `src/wasm.rs:1017-1022`) in
place. After this fix the only thing it can discard is a sentinel, and it is the
analogue of upstream's `wait_for_idle()` queue drain
(`local_connection.py:625-629`).

**Call sites.** `src/conversation.rs:37-41` (add `receiving`) · `:57-69`
(`Conversation::new` initialises it; signature unchanged) · `~:50` (add
`ReceivingGuard` + `IDLE_POLL_INTERVAL`) · `:140-152` (`send` — the actual fix) ·
`:155-228` (`receive_steps`) · `src/connection.rs:277-284`
(`MockConnection::receive_steps` — change `.clone()` to
`std::mem::take(&mut *lock)` so a second call yields a different turn; every
existing test pushes-then-iterates-once, so this is behaviour-compatible and is
what makes the ported drain test expressible) · `docs/conversation.md` (document
that `send()` drains an unconsumed turn). **No production call site of
`Conversation::send` changes:** `src/conversation.rs:282` (`chat`),
`examples/agent_server/src/main.rs:482` (`agent.send`).

**Tests.**
- `send_drains_previous_turn_into_history` — port of 0.1.1
  `conversation_test.py:964-1029`
  (`ConversationSendDrainTest::test_back_to_back_send_drains_first_turn`; 0.1.9:
  `:971-1036`): mock starts idle → push turn-1 step → `send("first")` (no drain)
  → `mock.is_idle.store(false)` → `send("second")` (drains) → assert
  `history().len() == 1`, `history()[0].content == "first reply"`,
  `turn_count() == 2` → push turn-2 step, iterate `receive_steps()` → assert
  `history().len() == 2` in order. **Requires the `mem::take` change to
  `MockConnection`.**
- `send_waits_instead_of_draining_while_a_stream_is_live` — port of
  `conversation_test.py:1031-1060` (0.1.9: `:1038-1067`,
  `test_send_falls_back_to_wait_for_idle_on_runtime_error`): hold a
  `receive_steps()` stream (do not poll it), `mock.is_idle.store(false)`, spawn
  `send("x")` on a task, assert it has not completed after a short delay,
  `mock.is_idle.store(true)`, assert it completes and that history was not
  double-recorded.
- `send_propagates_error_from_drained_turn` — NEW: the drained stream yields
  `Err`, `send()` returns that `Err` and never calls `conn.send` (assert
  `sent_prompts` is empty). Pins upstream's exception-propagation semantics.
- `first_send_on_fresh_connection_does_not_drain` — **the WI-12 regression
  guard.** With `MockConnection::new` (idle == true) and an empty step script,
  `send()` must return promptly. Wrap in `#[tokio::test]` + a timeout so a
  regression fails instead of hanging CI.

**Behaviour change.** A second `send()` while a turn is still running now blocks
until that turn finishes and records its steps into `history()`, and can now
return an error raised by the *previous* turn. Previously it returned immediately
and destroyed turn 1's steps and usage. `turn_start_indices` (and therefore
`turn_count`) become meaningful across back-to-back sends.

**Risk.**
1. The WI-12 dependency (above).
2. The fallback loop uses `tokio::time::sleep`, which on wasm32 needs a tokio
   timer that `any_spawner::Executor::spawn_local` does not provide — the same
   latent hazard the crate already ships in `src/trigger_helpers.rs:25`, so it is
   not a new dependency, but the concurrent-iterator arm may panic under wasm.
   Mitigate by replacing this loop with `Connection::wait_for_idle()` when A6
   lands; until then, note it in the module docs.
3. An unbounded drain: if the harness never reports idle (e.g. the 0.1.9
   `STATE_FULLY_IDLE` rename, W2), `send()` now blocks where it previously
   returned. That is upstream's behaviour, and W2 must be fixed before pointing
   at a 0.1.9 harness anyway. **Do not paper over it with a timeout**, or the
   drain silently loses steps again.
4. The state lock must be taken **after** the drain, and the guard must be moved
   into the stream (the `unfold` form above), not merely referenced in a `move`
   closure body.

---

### WI-14 (T3) — Delete the `google_search`/`web_search` → `python3` → DuckDuckGo fallback

**Defect.** `src/tools.rs:168` intercepts
`call.name == "google_search" || call.name == "web_search"` *ahead of* the
unknown-tool arm at `:215-222`. On native it calls `builtin_web_search`
(`:232-293`), which spawns `python3 -c` with a 38-line inline script that fetches
`https://html.duckduckgo.com/html/?q=<query>` (`:244`) and regex-parses the HTML
(`:256,:268`); on wasm (`:203-214`) it fabricates a **success** `ToolResult`
carrying `{"results":[],"note":"Search not available in WASM environment"}`.

This is undeclared outbound network egress and an undeclared `python3` runtime
dependency in a crate whose README Option A sells a pure-Rust install; it
fabricates a success where the model asked for a tool that does not exist; and it
shadows the real harness-side `search_web` tool. It exists in **no** upstream
version: grepping every wheel 0.1.1–0.1.9 finds nothing, and upstream's only
handling for an unregistered tool is
`types.ToolResult(name=tc.name, error=f"Unknown tool: '{tc.name}'")` (0.1.9
`tools/tool_runner.py:361-364`; identical at 0.1.1 `:300-303`). Separately the
surviving unknown-tool arm uses the wrong wording: `src/tools.rs:220` produces
`"Tool {name} not found"` where upstream pins `"Unknown tool: '{name}'"`
(asserted by `tool_runner_test.py:399`). Documented as a feature at
`README.md:413-419` and `docs/tools.md:226-231`.

**Change.**
1. Delete `src/tools.rs:168-214` in its entirety — the whole
   `else if call.name == "google_search" || call.name == "web_search"` arm
   including both cfg blocks. The `else { ... }` unknown-tool arm at `:215-222`
   becomes the direct `else` of the `if let Some(tool) = tools.get(&call.name)`
   at `:149`.
2. Delete `src/tools.rs:228-293` — the
   `#[cfg(not(target_arch = "wasm32"))] async fn builtin_web_search` and its doc
   comment. `use tokio::process::Command;` is function-local (`:234`) so no
   module-level import becomes unused; `serde_json::Value` (`:9`) stays in use.
3. `src/tools.rs:220`: `format!("Tool {} not found", call.name)` →
   `format!("Unknown tool: '{}'", call.name)`. Keep `id: Some(call.id)`,
   `result: None` — upstream drops the id (`tool_runner.py:362`) but the Rust
   wire path needs it to address the `ToolResponse` (`src/local.rs:1212`), so
   retaining it is a deliberate, harmless divergence.
4. Docs: delete the Web Search Fallback bullet at `README.md:418` (keep the
   `enable_google_search` bullet at `:417` — that is W19/X8, a separate finding)
   and the whole `## Google Search Fallback` section at `docs/tools.md:226-231`.
5. **Replacement guidance** where the deleted section was in `docs/tools.md`:
   there is no in-crate replacement today. The upstream answer is the
   harness-side `search_web` tool (`BuiltinTools::SEARCH_WEB`), which needs
   `HarnessSideTools.search_web = 12` (W13) and the `BuiltinTools` variant (A13)
   and is therefore gated on the 0.1.9 migration. Until then the user registers
   their own `Tool` — add a ~20-line snippet showing a `struct WebSearchTool`
   whose `call` uses the user's own HTTP client, and **state plainly that the SDK
   makes no network requests of its own**.
6. Optional CI cleanup: `.github/workflows/ci.yml` installs `python3-websockets`
   and nothing uses it (X3(f)); this deletion removes the last plausible reason
   for a `python3` dependency, so drop it in the same commit.

**Call sites.** `src/tools.rs:168-214` · `:220` · `:228-293` · `README.md:418` ·
`docs/tools.md:226-231` · `.github/workflows/ci.yml`.

**Tests.**
- Port 0.1.9 `tools/tool_runner_test.py:382-399`
  (`test_unknown_tool_returns_error_result`) as a `#[tokio::test]` in
  `src/tools.rs`'s test module: an empty `ToolRunner`,
  `process_tool_calls(vec![ToolCall{ id:"1".into(), name:"nonexistent".into(),
  args: json!({}), canonical_path: None }])`, assert `results.len() == 1`,
  `results[0].result.is_none()`, and
  `results[0].error.as_deref() == Some("Unknown tool: 'nonexistent'")`.
- **Regression test pinning the deletion:** the same call with
  `name: "google_search"` and `name: "web_search"`, asserting both now produce
  `error == Some("Unknown tool: 'google_search'")` / `'web_search'` and
  `result.is_none()`. This is the test that fails if anyone re-adds a special
  case.

**Behaviour change.** A tool call named `google_search` or `web_search` with no
such tool registered now returns
`ToolResult{ result: None, error: Some("Unknown tool: '<name>'") }` instead of
scraped DuckDuckGo results (native) or a fabricated empty-success (wasm). Every
other unregistered tool changes its error text from `"Tool <name> not found"` to
`"Unknown tool: '<name>'"`. The crate stops spawning subprocesses and stops
making network requests other than the harness WebSocket.

**Risk.** No compile risk — a deletion plus one string literal. The real risk is
product-level and belongs in the changelog. **Do not compromise by keeping it
behind a cargo feature:** a feature flag preserves the supply-chain surface
(arbitrary `python3` execution, unannounced egress to a third-party site) for
anyone who enables it, and the crate has no `[features]` section at all today
(X9), so adding one for this is the worst possible first feature.

---

### WI-15 (H9) — Contain an erroring `on_tool_error` hook

**Defect.** `src/hooks.rs:276-294` uses `?` inside the loop:
`let (res, val) = hook.on_tool_error(error).await?;`. One hook returning `Err`
aborts the remaining hooks **and** returns `Err` from the dispatcher — which both
call sites then discard: `src/local.rs:921` is
`let _ = runner_clone.dispatch_on_tool_error(&err).await;` and
`src/local.rs:1160` is `if let Ok((res, val)) = ...` (mirrored at
`src/wasm.rs:560` and `:799`). Net effect: a broken error-recovery hook silently
disables error recovery for every hook after it, **with no log line anywhere**.
Upstream has caught this since 0.1.1 — `hook_runner.py:231-242` wraps each hook
in try/except, logs `"Critical failure in OnToolErrorHook"` (`:236`), and returns
`HookResult(allow=False, message=f"Error recovery failed: {e}")` (`:237-242`),
returning immediately rather than continuing. The empty-message fall-through is
`:243`.

> **Citation correction.** The reviewed spec cited `hook_runner.py:246-259` for
> the try/except; `:246-269` is `dispatch_interaction`. The real
> `dispatch_on_tool_error` body is `~:222-243`.

**Change.** Replace the `?` in the loop with a `match` that converts a hook
failure into a definite non-recovery carrying the reason, and returns
immediately:

```rust
match hook.on_tool_error(error).await {
    Ok((res, val)) => { /* existing short-circuit logic, unchanged */ }
    Err(e) => {
        tracing::error!("Critical failure in on_tool_error hook: {e}");
        return Ok((
            HookResult { allow: false, message: format!("Error recovery failed: {e}") },
            None,
        ));
    }
}
```

`tracing::error!` needs no import (macro path resolution); `tracing` is a direct
dependency (`Cargo.toml:20`). The dispatcher **keeps** its
`Result<(HookResult, Option<Value>), anyhow::Error>` return type, so both call
sites and their `if let Ok(..)` / `let _ =` shapes compile untouched. Leave
`src/hooks.rs:290`'s `error.to_string()` fall-through alone — that is H10.

**Call sites.** `src/hooks.rs:276-294` (the only code edit) ·
`src/local.rs:921` (behaviour improves, no code change) · `src/local.rs:1160`
(`if let Ok((res, val))` now always matches; the failure surfaces as
`res.allow == false`, so `result.error` is correctly preserved instead of being
left indeterminate) · `src/wasm.rs:560`, `:799` (identical, no code change) ·
`docs/hooks.md:183` — the dispatch table says `dispatch_on_tool_error`
short-circuits at the first `allow: true`; add that it also short-circuits at the
first hook `Err`, returning `allow: false` with `"Error recovery failed: ..."`.

**Tests.**
- `test_dispatch_on_tool_error_hook_failure_is_contained` — port of 0.1.1
  `hook_runner_test.py:365-381` (byte-identical twin at 0.1.9 `:365-382`, so this
  locks in behaviour across the migration): a hook returning
  `Err(anyhow!("Hook failed"))` must return `Ok`, `!res.allow`, and
  `res.message.contains("Error recovery failed")` — upstream's assertions
  verbatim.
- `test_dispatch_on_tool_error_failure_short_circuits_later_hooks` — hooks
  `[ok_no_recover, failing, recovering]`; assert the recovering hook is NOT
  reached (mirrors upstream, which returns immediately from the except block).
- `test_dispatch_on_tool_error_no_recovery_returns_deny` — port of 0.1.1
  `hook_runner_test.py:383-399` (`test_dispatch_on_tool_error_fall_through`):
  with only non-recovering hooks the result is `allow: false` and the payload is
  `None`. Pins the untouched fall-through at `src/hooks.rs:287-293` so the new
  `Err` arm cannot be confused with it.
- `test_dispatch_on_tool_error_recovery` — port of 0.1.1
  `hook_runner_test.py:184-201`; the positive counterpart, guarding that
  containment did not break the recovery short-circuit already covered by
  `src/hooks.rs:615-646`.
- Port 0.1.1 `local_connection_test.py:3984-4033`
  (`test_on_tool_error_dispatched_for_builtin_error`) as an integration-level
  guard for the built-in path at `src/local.rs:917-922`, where the dispatcher's
  `Result` is thrown away with `let _ =` and a crashing hook is otherwise
  completely invisible.

**Behaviour change.** `HookRunner::dispatch_on_tool_error` no longer returns
`Err` when a hook fails; it returns
`Ok((HookResult{allow:false, message:"Error recovery failed: ..."}, None))` and
logs at ERROR. Migration note: if you call it directly and relied on `Err` to
detect a broken hook, check
`!res.allow && res.message.starts_with("Error recovery failed")` instead.

**Risk.** Very low, self-contained. The one semantic point to be deliberate
about: with the fix, a failing hook **stops** the chain rather than continuing.
That is upstream's choice at `hook_runner.py:237-242` and is the fail-closed
reading — a hook that blew up must not be silently stepped over on the way to
something that might fabricate a success. Note the interaction with WI-27 (H4):
today `allow: true` from a recovery hook erases the tool failure at
`src/local.rs:1161-1165`. This item does not change that; it only guarantees a
hook **crash** can never be mistaken for anything other than non-recovery. The
existing recovery test at `src/hooks.rs:615-646` is unaffected.

---

### WI-16 (H1c) — Dispatch `session_end` from `disconnect()` on both transports

**Defect.** `HookRunner::dispatch_session_end` (`src/hooks.rs:310-316`) has no
production caller. `LocalConnection::disconnect` (`src/local.rs:313-318`) locks
the child and SIGKILLs it; `WasmConnection::disconnect` (`src/wasm.rs:1158-1161`)
is a bare `Ok(())`. Upstream dispatches `session_end` first thing in
`disconnect()`, captures any hook error, proceeds with the full graceful teardown
regardless, and re-raises the hook error at the end (0.1.1
`local_connection.py:681-690`, `:731-733`). `Agent::stop`
(`src/agent.rs:422-425`) → `Conversation::disconnect`
(`src/conversation.rs:321-323`) → `Connection::disconnect`, so the whole
documented shutdown path silently skips the hook.

**Change — `src/local.rs:313-318`:**

```rust
async fn disconnect(&self) -> Result<(), anyhow::Error> {
    // Upstream local_connection.py:681-690: dispatch first, capture the error,
    // tear down regardless, re-raise at the end (:731-733).
    let hook_error = match self.hook_runner {
        Some(ref runner) => runner.dispatch_session_end().await.err(),
        None => None,
    };
    let mut proc = self.process.lock().await;      // existing body, unchanged
    let _ = proc.kill().await;
    drop(proc);
    hook_error.map_or(Ok(()), Err)
}
```

**`src/wasm.rs:1158-1161`:**

```rust
async fn disconnect(&self) -> Result<(), anyhow::Error> {
    match self.hook_runner {
        Some(ref runner) => runner.dispatch_session_end().await,
        None => Ok(()),
    }
}
```

**The `match` form is load-bearing, not stylistic.** Verified empirically in a
probe crate carrying this repo's exact lint table: the `if let`/`else` forms emit
`clippy::option_if_let_else` (nursery → error under `-D warnings`); these two
`match` expressions emit **no lint at all**, and `clippy::manual_ok_or` does not
fire on `map_or(Ok(()), Err)`.

Upstream's `if self._hook_runner.on_session_end_hooks` gate (`:686`) is
deliberately skipped: `HookRunner` stores one untyped `Vec`
(`src/hooks.rs:206`) with no per-category registry, and `Hook::on_session_end`'s
default is `async { Ok(()) }` (`src/hooks.rs:81-86`), so unconditional dispatch
is a no-op loop.

Borrowck: `self.hook_runner` is `Option<HookRunner>` (`src/local.rs:59`,
`src/wasm.rs:921`), `dispatch_session_end(&self)` takes it immutably
(`src/hooks.rs:309-316`), and the guard drops before
`self.process.lock().await` — no conflict.

**Call sites.** `src/local.rs:313-318` · `src/wasm.rs:1158-1161` · **No caller
changes:** `AnyConnection::disconnect` (`src/connection.rs:222-231`),
`Conversation::disconnect` (`src/conversation.rs:321-323`) and `Agent::stop`
(`src/agent.rs:422-425`) already return `Result<(), anyhow::Error>` and
propagate. · `examples/agent_server/src/main.rs:1036` and `:1203` — **no edit
required**, both already `let _ =` the `Result`, so the new `Err` arm is silently
discarded there; listed so a reader auditing shutdown paths knows those two sites
will swallow a failing `session_end` hook. · `docs/hooks.md:119` and
`README.md:238` — remove any implication that `on_session_end` is currently
inert; document that a failing hook makes `Agent::stop()` return `Err`
**after** the connection is torn down.

**Tests.**
- `tests/integration_tests.rs::test_session_end_hook_dispatched_on_disconnect` —
  port of 0.1.1 `local_connection_test.py:2086-2109`: register a hook that pushes
  into a shared `Vec` from `on_session_end`, start an agent against
  `src/bin/mock_localharness.rs`, call `agent.stop()`, assert the `Vec` is
  `["ended"]`.
- `::test_session_end_hook_error_still_kills_process_and_is_returned` — NEW: a
  hook returning `Err` makes `agent.stop()` return `Err`, and the child process
  is nonetheless dead (assert `try_wait()` is `Some`, or that a second connect on
  the same port succeeds). Upstream analogue: the
  `finally: if hook_error is not None: raise hook_error` at
  `local_connection.py:731-733`.
- `src/hooks.rs::test_dispatch_session_end_propagates_first_error` — NEW;
  `src/hooks.rs:310-316` uses `?`, so an erroring hook aborts the remaining
  hooks. Pin that (it is upstream-faithful: 0.1.1 `hook_runner.py:143-146` has no
  guard) so a later refactor does not silently change it.
- `src/hooks.rs` — port 0.1.1 `hook_runner_test.py:102-114`
  (`test_dispatch_session_end`): it pins that all registered hooks are called in
  order, which the negative test above only covers negatively.
- `::test_disconnect_dispatches_session_end_exactly_once` — call `agent.stop()`
  twice and assert the hook fired once per call and never panics on the
  already-killed child; `src/local.rs:315` uses `let _ = proc.kill().await` so a
  second disconnect is currently a silent no-op, and the hook must not become a
  double-dispatch hazard.
- `src/wasm.rs::test_wasm_disconnect_dispatches_session_end` — cloned from
  `:1534-1640`. **The WASM half genuinely has no other behaviour in
  `disconnect`**, so this test is the only thing pinning that the new match arm
  is reached at all.

**Behaviour change.** `on_session_end` now fires on `Agent::stop()` /
`Conversation::disconnect()` (it never did), on both transports. `Agent::stop()`
can now return `Err` where it previously always returned `Ok` — specifically when
a user's own `session_end` hook fails. The connection is torn down either way.

**Risk.** Low. Note `src/agent.rs:294` always registers `PolicyEnforcer`, so
`hook_runner` is effectively always `Some` — this becomes live on every shutdown.
Behavioural risk: if a `session_end` hook blocks forever, `disconnect()` now
hangs where it previously killed immediately — upstream has the same exposure and
does not bound it; if you want a bound, wrap in `tokio::time::timeout` (native
only; there is no wasm equivalent, which is why it is not specced).

---

### WI-17 — `src/hook_dispatch.rs`, the target-neutral module WI-18/WI-19 delegate to

**Defect.** `src/lib.rs:68-71` gates `pub mod local` and `pub mod wasm` against
each other, so neither file can import from the other. Without a neutral home,
the ~45 lines of new shared logic WI-18 and WI-19 need (the `receive_steps`
poll-state enum, the synthetic denied-turn `Step`, and the terminal-model-step
predicate) get copy-pasted a third time and drift.

**New file `src/hook_dispatch.rs`**, containing:

- `RecvState` — `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, the outer poll
  state for `receive_steps`. The `Debug` derive is required
  (`missing_debug_implementations = "warn"`, `Cargo.toml:43`).
- `denied_turn_step(message: String) -> Step` — the single synthetic step a
  denied turn yields. Mirrors 0.1.1 `local_connection.py:541-547`.
- `is_turn_terminal_step(status, source, target) -> bool` — `pub const fn` over
  `matches!` on `Copy` enums. Mirrors 0.1.1 `local_connection.py:596-605`.

**File-level doc comment must cite the real anchors** (three citation errors in
the reviewed spec, corrected here): `Step` derives `Clone` — `src/types.rs:477`
is `#[derive(Debug, Clone, Serialize, Deserialize)]` — and `Default` is a
**hand-written** `impl Default for Step` at `src/types.rs:520-542`;
`StepType::SystemMessage` is `src/types.rs:404` (`:403` is its
`#[serde(rename)]`); `StepStatus::Canceled` is `src/types.rs:467` (`:469` is
`TerminalError`'s serde rename). The functional claims are unaffected —
`..Default::default()` works and `Step` is `Clone`.

**`src/lib.rs`: insert `mod hook_dispatch;` on a NEW line 68** — i.e. after
`pub mod hooks;` (`:67`) and **BEFORE** the `#[cfg(not(target_arch = "wasm32"))]`
attribute currently on `:68`. Inserting it between that attribute and
`pub mod local;` (`:69`) silently reattaches the cfg to the wrong item and breaks
the wasm build.

**Visibility: plain `pub` items inside the private module, not `pub(crate)`.**
Verified empirically in a probe crate with this repo's exact lint table:
`pub(crate) fn` inside a private `mod` emits `clippy::redundant_pub_crate`
(nursery) → hard failure under CI's `-D warnings`
(`.github/workflows/ci.yml:35`, mirrored in `justfile:10`). Plain `pub` items in
a private module produce no lint.

**Call sites.** `src/lib.rs:67` (insert `mod hook_dispatch;` after
`pub mod hooks;`, **no cfg**) · `src/hook_dispatch.rs` (new file) ·
`src/local.rs:8` (add the `use crate::hook_dispatch::{...}` line — `:8` is
currently `use crate::hooks::HookRunner;`) · `src/wasm.rs:24` (same anchor, same
addition).

**Tests.**
- `src/hook_dispatch.rs::tests::test_denied_turn_step_shape` — assert
  `r#type == StepType::SystemMessage`, `source == StepSource::System`,
  `status == StepStatus::Canceled`, `error == message`, and that every other
  field equals `Step::default()`'s. Pins the exact yield that 0.1.1
  `local_connection.py:541-547` constructs and `local_connection_test.py:294-296`
  asserts.
- `::test_is_turn_terminal_step_truth_table` — all four `StepStatus` values ×
  `{Model, System, User}` × `{User, Environment}`. **The negative row
  (Model/DONE/Environment must be false) is exactly** 0.1.1
  `local_connection_test.py:2241-2296`
  (`test_post_turn_hook_not_fired_for_environment_step`) and `:1095-1110`
  (`test_is_complete_response_false_when_target_environment`).

**Behaviour change.** None — new private module, no public surface.

**Risk.** Dead-code analysis holds: all three items are consumed by `local.rs` in
the non-wasm build, by `wasm.rs` in the wasm build, and by both under
`cfg(test)`. Verify with `cargo build` (not just `cargo test`) **and**
`cargo check --target wasm32-unknown-unknown`.

---

### WI-18 (H1a) — Dispatch `pre_turn` from `Connection::send`, with upstream's deny semantics

**Depends on WI-17.**

**Defect.** `HookRunner::dispatch_pre_turn` (`src/hooks.rs:237-249`) has no
production caller. `LocalConnection::send` (`src/local.rs:159-184`) and
`WasmConnection::send` (`src/wasm.rs:1007-1032`) go straight from resetting idle
state to serialising the `InputEvent`. A user who registers the rate-limiting
`pre_turn` hook that `docs/hooks.md:430-467` presents as a **working example**
gets a hook that never runs and a turn that is never denied. Regression against
0.1.1, which dispatches `pre_turn` inside `send()` and honours the denial
(`local_connection.py:507-517`), yielding a CANCELED `SystemMessage` step from
`receive_steps` and returning (`:532-548`). Pinned by
`local_connection_test.py:272-296` and `:298-317`.

**Change.** Three edits per transport: new shared state on the connection struct,
the dispatch + latch in `send()`, and a `RecvState::Fresh` check at the head of
`receive_steps` that consumes the latch and yields `denied_turn_step`.

**`src/local.rs`:** struct fields `:51-62` (`step_trackers` is at `:62`) ·
`receive_steps` `:92-157` (yields at `:124` and `:143`) · `send` `:159-184` (the
`InputEvent` is built at `:176`) · shared state created in `connect` at
`:700-708` · struct literal `:1252-1265`. `LocalConnection` has a hand-written
`Debug` with `finish_non_exhaustive` (`:65-77`) so it needs no edit.

**`src/wasm.rs`:** fields `:915-924` · `receive_steps` `:940-1005` (yields at
`:972`, `:991`) · `send` `:1007-1032` · shared state `:350-359` · struct literal
`:896-907`. `WasmConnection` derives `Debug` (`:913`) and all three new field
types (`Arc<tokio::sync::Mutex<Option<String>>>`, `Arc<AtomicBool>`,
`Arc<Mutex<HashMap<String,String>>>`) are `Debug`, so the derive survives.

**Verified compile properties** (reproduced in a probe crate):
`stream::unfold(RecvState::Fresh, move |state| async move {...})` type-checks
(`RecvState` is `Copy + PartialEq`, unfold's `T` is inferred); the
`if state == RecvState::Fresh { ... }` block scopes its `MutexGuard` so it drops
before the `loop` re-locks `step_rx` (no self-deadlock); `Connection::send` is
declared `-> impl Future + Send` (`src/connection.rs:27-30`) and every guard
taken in the new body is a `tokio::sync::MutexGuard<T: Send>` dropped at
statement end. **No trait signature changes**, so `AnyConnection`'s forwarding
impl (`src/connection.rs:128-137`) and `MockConnection` (`:286-292`) compile
untouched.

**Hoist the latch read.** `let denial = turn_denied.lock().await.take();
if let Some(message) = denial` produces **no** lint; the inline form
`if let Some(m) = turn_denied.lock().await.take()` **does** emit
`clippy::significant_drop_in_scrutinee`.

**Divergence from upstream to avoid:** do **not** set
`*self.parent_idle.lock().await = true;` in the deny block. Upstream 0.1.1
`local_connection.py:502-517` sets only `self._is_idle.set()`; `_parent_idle` is
left at the `False` assigned on `:504`. Harmless in practice (the next send
resets it at `src/local.rs:162-164`) but an unforced divergence.

**Unmentioned interaction to handle:** `Conversation::send` pushes the turn
boundary at `src/conversation.rs:146-150` **before** awaiting
`self.conn.send(prompt)` at `:151`. Today `conn.send` only fails on a
serde/channel error; after this change a hook `Err` makes that path reachable in
normal operation, leaving a dangling entry in `state.turn_start_indices` for a
turn that never happened. **Either reorder the push after the await, or document
the leak.** (If WI-13 has landed, the push already sits after the drain but still
before `conn.send` — the same reorder applies.)

**Call sites.** `src/local.rs:51-62`, `:92-157`, `:159-184`, `:700-708`,
`:1252-1265` · `src/wasm.rs:915-924`, `:940-1005`, `:1007-1032`, `:350-359`,
`:896-907` · `src/hook_dispatch.rs` (`RecvState`, `denied_turn_step`) ·
`src/conversation.rs:146-151` (turn-boundary ordering) · `docs/hooks.md:430-467`
— the rate-limiter example stops being aspirational; delete any "not yet
dispatched" caveat and document the deny contract (one CANCELED `SystemMessage`
step, empty `ChatResponse.text`) · **`src/wasm.rs:1542-1600`** — the mock
WebSocket server inside `test_wasm_connection_integration_mock` asserts a second
inbound frame (`assert!(text2.contains("hello"))` at `:1595-1596`) and is joined
with `server_handle.await.unwrap()` at `:1638`; **a denial test cloned from it
hangs and then panics unless this body is also rewritten to stop after the config
frame.**

**Tests.**
- `tests/integration_tests.rs::test_pre_turn_hook_deny_yields_single_canceled_step`
  — port of 0.1.1 `local_connection_test.py:272-296` (`test_turn_hook_deny`):
  register a `Hook` whose `pre_turn` returns
  `HookResult{allow:false, message:"Denied by hook"}`, drive
  `src/bin/mock_localharness.rs`, call `conversation.send("Hello")`, drain
  `receive_steps()`, assert exactly one step with
  `status == StepStatus::Canceled` and `error == "Denied by hook"`.
- Negative test in the same file: assert the mock harness received **no**
  user-input frame after a denial (the mock at
  `src/bin/mock_localharness.rs:77-84` blocks on the prompt read, so assert via a
  timeout or a counting mock).
- `::test_pre_turn_hook_allows_and_forwards` — an allowing hook lets the normal
  mock turn complete, proving the happy path is unregressed.
- `::test_pre_turn_hook_error_fails_closed` — a `pre_turn` returning `Err` makes
  `Conversation::send` return `Err` and sends nothing. Upstream analogue:
  `dispatch_pre_turn` (0.1.1 `hook_runner.py:152-166`) has no exception guard.
- `::test_pre_turn_dispatched_on_every_send` — port of 0.1.1
  `local_connection_test.py:298-318`
  (`test_send_none_dispatches_turn_hook_with_empty_string`) in its
  Rust-applicable form: two consecutive sends produce two `pre_turn` dispatches,
  proving the latch reset at the top of `send()` works.
- `::test_denial_latch_cleared_by_next_send` — deny turn 1, then allow turn 2,
  and assert turn 2's stream does NOT re-yield a CANCELED step. **This pins a
  deliberate divergence** from upstream (0.1.1 `local_connection.py:541`
  re-yields on every `receive_steps` until the next send resets `_cancelled` at
  `:502`); the divergence is an improvement but must be locked down.
- `src/hooks.rs` — port 0.1.1 `hook_runner_test.py:41-54`
  (`test_dispatch_pre_turn_deny`) and `:27-40`
  (`test_dispatch_pre_turn_allow`) as unit tests if not already covered.
- WASM half: clone `src/wasm.rs:1534-1640` into `test_wasm_pre_turn_hook_deny`,
  passing `hook_runner: Some(..)` at `:1616` **and** truncating the mock server
  body so it stops after the `InitializeConversationEvent` frame, dropping the
  `text2.contains("hello")` assertion at `:1595-1596` that can never be satisfied
  after a denial.

**Behaviour change.** A registered `pre_turn` hook now runs on every
`Conversation::send` / `Agent::chat`, and a denying hook halts the turn (nothing
sent, one CANCELED step, empty `ChatResponse.text`). A hook returning `Err` now
propagates out of `Conversation::send`. Migration note: `pre_turn` was documented
as working (`docs/hooks.md:120`, `README.md:241`) but was dead code; if you
registered a denying `pre_turn` hook it will now deny. `Connection::send` still
returns `Ok` on denial — check
`conversation.history().last().status == StepStatus::Canceled`.

**Note on severity.** This is labelled `behaviour`, not `none`: `Connection::send`
is a public trait method whose observable contract changes twice (hooks now
execute; a new `Err` path appears), `receive_steps` gains a new terminal item, and
because `src/agent.rs:294` unconditionally registers `PolicyEnforcer`, a
`HookRunner` is essentially always present.

**Risk.**
1. **Lock ordering:** `send()` acquires `turn_denied`, `parent_idle`,
   `active_subagent_ids`, `subagent_responses` and `step_rx` each in its own
   scoped block, never two at once, so it cannot deadlock against the reader loop
   (which holds `active_subagent_ids` + `parent_idle` together).
2. The denial latch is only consumed in `RecvState::Fresh`. In the normal flow
   this is safe because `Conversation::chat` (`src/conversation.rs:282-283`)
   awaits `send()` **before** constructing the stream. A stream created before a
   denying `send()` stays blocked — same as today, and the same as upstream,
   where `receive_steps()` is a fresh generator per turn.
3. Setting `is_idle = true` on deny interacts with the known
   `conn_is_idle.swap(true)` sentinel-suppression bug (C3, `src/local.rs:1057`):
   harmless here because a denied turn puts nothing on the wire.
4. `clippy::significant_drop_in_scrutinee` — hoist the `take()` as shown.

**Do NOT add a `prompt` parameter to `Hook::pre_turn`** in this work item unless
H6 is landing in the same release: W11/WP-10 will change `Connection::send` to
take `Content`, and `pre_turn`'s data parameter should be introduced once, as
`&Content`, not twice.

---

### WI-19 (H12) — Dispatch `post_tool_call` when a subagent trajectory completes

**Depends on WI-17, WI-18** (shares the `subagent_responses` plumbing and the
per-turn clear in `send()`).

**Defect.** When a subagent finishes, the harness signals it as a
`TrajectoryStateUpdate` on the subagent's trajectory id, not as a tool response.
`src/local.rs:1048-1054` (and `src/wasm.rs:687-693`) merely removes the id from
`active_subagent_ids` and dispatches nothing, so a `post_tool_call` hook sees a
`START_SUBAGENT` `pre_tool_call` with no matching completion.
**`examples/subagents.rs:37-56` is built on exactly that pairing** — its
`post_tool_call` prints "Subagent Finished" and resets `subagent_active` in the
`START_SUBAGENT` branch at `:38-41`, and it never fires. Upstream dispatches
`post_tool_call` with
`ToolResult(name=START_SUBAGENT, result=<subagent's last model text> or <trajectory id>)`
at 0.1.1 `local_connection.py:906-919`, fed by a `_subagent_responses` map
populated from subagent MODEL steps at `:808-825` and cleared in `send()` at
`:506`.

`BuiltinTools::StartSubagent.as_str()` returns `"START_SUBAGENT"`
(**`src/types.rs:207`** — the reviewed spec cited `:203`, which is `ListDir`) and
matches the name `extract_builtin_tool_call` assigns at `src/local.rs:1304`, so
pre/post pairing by name works.

**Change.** Add a `subagent_responses: Arc<Mutex<HashMap<String, String>>>` to
both connection structs, plumbed exactly like WI-18's state; a capture block in
the reader loop next to the step send; a dispatch block in the STATE_IDLE arm;
and a `.clear()` in `send()`.

**Borrowck verified at the dispatch site** (`src/local.rs:1044-1054`): `sub_id`
is moved by `active_subs.insert(sub_id)` in the STATE_RUNNING arm at `:1046`, but
the STATE_IDLE arm at `:1048-1053` is a **disjoint branch**, so
`active_subs.remove(&sub_id)` and `sub_id.clone()` are legal, and nothing after
the if/else (`:1056-1064`) touches `sub_id`. In the capture block, `traj_id`
(`src/local.rs:744`) is still live at `:877` — the `Step` literal only clones it
at `:873` — and `conn_cascade_id_for_ws` (`:716`) is used by reference at `:754`
and `:1036` so it is not moved. Write
`let learned = conn_cascade_id_for_ws.lock().await.clone();` as a **statement**
(not in a scrutinee) so the guard drops immediately and the nursery drop lints do
not fire.

**Deadlock analysis holds** because the whole reader is a single spawned task:
the TSU arm's `active_subs → parent_idle → subagent_responses` order can never
interleave with the StepUpdate arm's lone `subagent_responses` lock, and `send()`
takes each of the four in its own scoped block (`src/local.rs:160-174`). Mirrors
at `src/wasm.rs:388`, `:367`, `:516`, `:683-693`, `:1007-1032`.

**Call sites.** `src/local.rs:60-63` + `:702` + `:1262`
(`subagent_responses` plumbing, shared with WI-18) · `:159-184` (clear in `send`)
· `:877` (capture block) · `:1048-1054` (dispatch) · `src/wasm.rs:922-925` +
`:353` + `:904` · `:1007-1032` · `:516` · `:687-693` ·
**`examples/subagents.rs:37-56`** — no code change; its `post_tool_call` starts
firing, which is the point. Worth re-reading as the acceptance criterion. ·
`docs/hooks.md:122-125` — document that `post_tool_call` fires for
`START_SUBAGENT` on subagent completion, with `result` = the subagent's last
model text. · **`src/wasm.rs:1542-1600`** — the three-frame subagent fixture must
be added to the mock server body inside `test_wasm_connection_integration_mock`,
whose current body reads exactly two inbound frames and asserts
`text2.contains("hello")` at `:1595-1596`.

**Tests.** Every one needs an explicit rendezvous — the dispatch is
`tokio::spawn`/`crate::spawn_task`, whereas upstream awaits it inline
(`local_connection.py:917`). Have the hook signal a `tokio::sync::Notify` or an
`mpsc`, and assert under `tokio::time::timeout(Duration::from_secs(1), ..)`,
mirroring upstream's own pattern for backgrounded dispatch at
`local_connection_test.py:2320`/`:2349`.

- `test_post_tool_call_hook_on_subagent_trajectory_idle` — port of 0.1.1
  `local_connection_test.py:2399-2467`. Its sequence is the exact fixture to
  reproduce in `src/bin/mock_localharness.rs` behind a prompt keyword: (a) a main
  step with `cascadeId == trajectoryId == "main_traj"` to establish the learned
  cascade id; (b) a subagent step
  `{trajectoryId:"sub_traj", cascadeId:"main_traj", source:SOURCE_MODEL,
  target:TARGET_USER, text:"Here is a poem about nature."}`; (c)
  `{"trajectoryStateUpdate":{"trajectoryId":"sub_traj","state":"STATE_IDLE"}}`.
  Assert one capture with `name == "START_SUBAGENT"` and
  `result == Some(json!("Here is a poem about nature."))` — upstream's assertions
  at `:2463-2466`.
- Port the tail of the same test (`:2468-2490`): the MAIN trajectory going idle
  must NOT fire a subagent `post_tool_call`.
- `test_subagent_post_tool_call_falls_back_to_trajectory_id` — NEW: subagent
  idles with no model text captured; `result` is the trajectory id (upstream
  `response or tsu.trajectory_id` at `:915`).
- `test_send_resets_subagent_tracking` — port of 0.1.1
  `local_connection_test.py:2641-2670` (the direct upstream analogue of the
  invented "clears subagent responses" test), pinning the
  `_subagent_responses.clear()` at `local_connection.py:506`.
- Port 0.1.1 `local_connection_test.py:2525-2567` (`test_subagent_running_tracked`)
  and `:2568-2640` (`test_connection_waits_for_subagents_before_idle`) — both
  exercise the exact STATE_RUNNING/STATE_IDLE arm being edited at
  `src/local.rs:1044-1054` and guard against the new dispatch block breaking the
  idle-sentinel logic at `:1057-1064`.
- Port 0.1.1 `local_connection_test.py:2369-2398`
  (`test_invoke_subagent_step_classified_as_tool_call`) — pins that the
  `START_SUBAGENT` `pre_tool_call` side of the pairing still classifies correctly
  via `extract_builtin_tool_call` (`src/local.rs:1301-1310`), which is what makes
  the `post_tool_call` name match meaningful.
- WASM half: the same three-frame fixture through the `src/wasm.rs:1534-1640`
  mock.

**Behaviour change.** `post_tool_call` now fires once per completed subagent
trajectory, with
`ToolResult{name:"START_SUBAGENT", id: None, result: Some(<last subagent model
text, or the trajectory id>), error: None}`. Hooks that assumed `post_tool_call`
only follows a tool they saw in `pre_tool_call`, or that dereference `result.id`,
must handle this. `examples/subagents.rs` is the reference consumer; branch on
`result.name == "START_SUBAGENT"` to distinguish.

**Risk — state the correctness bound plainly to the implementer.** `is_subagent`
here reuses the existing `learned_cascade` at `src/local.rs:1036-1037`, which C4
shows is only populated from a `StepUpdate` where `cascade_id == trajectory_id`
(`:750-760`). If no such step has arrived — a resumed session, or a subagent
trajectory reporting first — `is_subagent` is false for everything and this hook
does not fire. **That is upstream 0.1.1's exposure too**
(`self._cascade_id and tsu.trajectory_id != self._cascade_id`, `:890-892`), so it
is parity-correct today and improves for free when C4 replaces the heuristic with
`main_trajectory_id` set from the first non-empty `trajectory_id`. **Do not try
to fix C4 here.** Also: the step-capture block runs for every `StepUpdate` and
takes the cascade-id lock once per step — cheap, but it must be a
`.lock().await.clone()` statement, not held across an await.

**Keep the capture block and the dispatch block visually contiguous** and comment
them with the upstream line refs, so WP-8 can remove them as one unit (see §7).

---

### WI-20 (N3) — Structured per-tool results for harness-executed built-in tools

**Defect.** `extract_tool_result` (`src/local.rs:1413-1428`, byte-identical fork
at `src/wasm.rs:1278-1293`) builds a `ToolResult` whose `result` is
`step_update.text.clone().map(Value::String)` for **every** built-in tool — one
opaque display string regardless of which tool ran. Its only consumer is the
built-in `post_tool_call` dispatch at `src/local.rs:905-915`, so a hook auditing
`run_command` sees rendered text instead of the command's combined output, and a
hook inspecting `list_directory` sees a rendered listing instead of entries.

> **Audit correction.** The audit files N3 against 0.1.9
> `connections/local/types.py:21-136` and implies it is 0.1.9-only. The file
> **and** the extractor both exist at 0.1.1 (`types.py:21-107`,
> `local_connection.py:142-207`), which is the version this crate was ported
> from, so N3 is a port regression and is implementable today.

It needs **no proto change** — every field is already in
`proto/localharness.proto`: `ActionRunCommand.combined_output = 5` (`:309`),
`ActionListDirectory.results = 2` (`:256`) with the
`Result{name=1, oneof info{is_directory=2, file_size=3}}` shape (`:247-253`),
`ActionFindFile.output = 3` (`:262`), `ActionSearchDirectory.num_results = 3`
(`:268`), `ActionEditFile.diff_block = 2` (`:302`),
`ActionGenerateImage.image_name = 3` (`:234`).

**New file `src/tool_output.rs`**, `pub mod tool_output;` in `src/lib.rs`
alongside `pub mod tools;`. Seven result structs
(`RunCommandResult{output}`, `ListDirectoryEntry{name,is_directory,file_size}`,
`ListDirectoryResult{entries}`, `SearchDirectoryResult{num_results}`,
`FindFileResult{output}`, `EditFileResult{summary}`,
`GenerateImageResult{image_name, aspect_ratio}`, `TextResult{text}`) each
`#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]`, plus:

```rust
/// The structured result of one harness-executed built-in tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum ToolOutput {
    RunCommand(RunCommandResult),
    ListDirectory(ListDirectoryResult),
    SearchDirectory(SearchDirectoryResult),
    FindFile(FindFileResult),
    EditFile(EditFileResult),
    GenerateImage(GenerateImageResult),
    Text(TextResult),
}

/// Extracts the structured result from a completed `StepUpdate`.
/// Each `StepUpdate` carries at most one action sub-message, so the first
/// match wins — mirroring upstream's elif chain.
pub fn extract_tool_output(
    su: &crate::proto::localharness::StepUpdate,
) -> Option<ToolOutput>;
```

**`Serialize` only on the enum; `Serialize + Deserialize` on the structs.**
`#[serde(untagged)]` serialises the inner object transparently (what we want) but
cannot deserialise unambiguously when every field has a default — and the 0.1.9
path needs per-struct deserialisation keyed by tool name anyway. **Deriving
`Deserialize` on the enum would compile and then silently deserialise everything
as the first variant.**

**Forward-compatible serde aliases, written in now** so the migration is a
no-op on these types: `RunCommandResult.output` gets
`#[serde(default, alias = "combined_output")]`, `ListDirectoryResult.entries`
gets `#[serde(default, alias = "results")]` (both are `AliasChoices` at 0.1.9
`connections/local/types.py:24-27,43-47`), and `GenerateImageResult.aspect_ratio`
is present but `#[serde(default, skip_serializing_if = "String::is_empty")]` —
0.1.9-only (`ActionGenerateImage.aspect_ratio = 4`, W10), inert today.

**`extract_tool_output` body**, ported one-for-one from 0.1.1
`local_connection.py:169-207` (same order, same emptiness guards, first-match-
wins). The `oneof info` in `ActionListDirectory.Result`
(`proto/localharness.proto:247-253`) generates a nested prost enum whose path
must be written out and matched — `is_directory` and `file_size` are **mutually
exclusive on the wire**, so reading both as plain fields is wrong:

```rust
use crate::proto::localharness::action_list_directory::result::Info;
let (is_directory, file_size) = match r.info {
    Some(Info::IsDirectory(b)) => (b, 0),
    Some(Info::FileSize(s))    => (false, s),
    None                       => (false, 0),
};
```

Note `file_size` is `uint64` → `u64` while `num_results` is `int32` → `i32`; **do
not unify the types.** The `edit_file` arm gates on `ef.diff_block` being
non-empty but takes the summary from `su.text`, exactly as upstream does.

`Display` must mirror upstream's `__str__` exactly (0.1.1
`connections/local/types.py:26,43-50,58,67,76,85,94`), because that string is
what a hook renders: `RunCommand`→`output`; `ListDirectory`→ per entry
`"{name}/ (dir)"` or `"{name} ({file_size} bytes)"` joined by `\n` (upstream uses
`os.linesep`; use `\n` and note the divergence); `SearchDirectory`→
`"{n} results"`; `FindFile`→`output`; `EditFile`→`summary`;
`GenerateImage`→`image_name`; `Text`→`text`.

**Call-site change, identical in both transports** (`src/local.rs:1413-1428`,
`src/wasm.rs:1278-1293`) — `extract_builtin_tool_call` stays duplicated for now
(X1 owns unifying it); only the result half is shared:

```rust
    // upstream: `result=extracted or step_obj.content` (local_connection.py:838-842)
    let result = crate::tool_output::extract_tool_output(step_update)
        .as_ref()
        .and_then(|o| serde_json::to_value(o).ok())
        .or_else(|| step_update.text.clone().map(Value::String));
```

**Call sites.** `src/tool_output.rs` (new) · `src/lib.rs:73-78` (`pub mod
tool_output;`) · `src/local.rs:1413-1428` · `src/wasm.rs:1278-1293` ·
`src/wasm.rs:1353-1373` (existing in-file unit test asserting the current
text-only behaviour; it happens to keep passing because its fixture sets
`find_file.output: None`, but **extend it with a populated-output case**) ·
`docs/tools.md` (document `ToolOutput` and what `post_tool_call` now receives).

**Tests.** Port 0.1.1 `local_connection_test.py:3751-3980` — the seven
`test_tool_result_for_*` cases (`run_command` `:3751-3777`, `list_directory`
`:3779-3817`, `find_file` `:3819-3849`, `search_directory` `:3851-3879`,
`edit_file` `:3881-3922`, `generate_image` `:3924-3963`, fallback `:3965`) — as
**unit tests on `extract_tool_output`** in `src/tool_output.rs` rather than full
harness tests. Concretely,
`ActionRunCommand{ command_line: "echo hello", combined_output: "hello\n" }` →
`ToolOutput::RunCommand(RunCommandResult{ output: "hello\n" })`, matching
`:3762-3777`. The `ActionListDirectory` case must carry two `Result` entries, one
with `Info::IsDirectory(true)` and one with `Info::FileSize(42)` — **this asserts
the `oneof` handling that a naive port gets wrong** (upstream test `:3792-3817`).
Emptiness guards: `ActionRunCommand{ combined_output: None }` → `None` → the
caller falls back to `step_update.text`, preserving today's behaviour.
`Display` round-trip test per variant against upstream's `__str__` strings.
**Mirror at least the `run_command` and `list_directory` cases into
`src/wasm.rs`'s existing `#[cfg(test)]` block next to `:1353-1373`**, so the fork
cannot silently diverge.

**Behaviour change.** `post_tool_call` hooks receive structured JSON for
`run_command`, `list_directory`, `find_file`, `search_directory`, `edit_file` and
`generate_image` instead of a single display string, falling back to the display
string when the harness sent no structured data. New public module
`antigravity_sdk_rust::tool_output` (additive).

**Risk.** Behavioural, not structural: hooks that today receive
`ToolResult.result == Value::String(display_text)` will receive
`Value::Object({...})` for six tools. The upstream-faithful `or_else` fallback
keeps `Value::String` whenever the action sub-message is empty, so hooks that
only ever saw text for e.g. `view_file` are unaffected — but this must be in the
changelog and in `docs/tools.md`. Keep `extract_tool_output` total (returns
`Option`, never `Err`); it runs on the reader task.

---

## 3.1 Release 0.2.0 work items

Everything from here changes a public signature. Cargo treats `0.1.x` as a
compatibility range, so any one of these forces `0.2.0` — which is why they are
batched (§4).

---

### WI-21 (S2) — Make pre-tool gating fail closed by construction

**Defect.** Pre-tool gating converts hook failure into ALLOW at five sites.
(a) `src/hooks.rs:251-266` `dispatch_pre_tool_call` uses
`let res = hook.pre_tool_call(tool_call).await?;` at `:257` — one erroring hook
aborts the whole chain and propagates `Err`, so if that hook is registered
*before* the `PolicyEnforcer` (registered last, `src/agent.rs:294`) **the entire
policy system is skipped for that call.** (b) `src/local.rs:1005-1013`
initialises `let mut allow = true;` and overwrites it only inside
`if let Ok(res) = pre_call` — the `Err` arm leaves `allow == true` and
`src/local.rs:1023` sends `accepted: Some(true)`. (c) `src/local.rs:1103` is
`.map_or(true, |res| res.allow)` — an erroring hook executes the custom tool.
(d)+(e) `src/wasm.rs:644-651` and `:742` mirror (b) and (c) verbatim. The `Err`
path is reachable today: `src/policy.rs:457-460` returns
`Err("ASK_USER policy '{}' is missing an ask_user handler")`, and that validation
is currently bypassed at startup (WI-8). A panicking async hook is also
unhandled: the `tokio::spawn` at `src/local.rs:1004` would die without ever
sending a `ToolConfirmation`, **deadlocking the harness**. Separately (H10),
`src/local.rs:1110-1139` discards the hook's `HookResult.message` and hardcodes
`"Execution denied by hook policy"` into both the denied `Step.error` (`:1120`)
and the `ToolResponse.response_json` (`:1129`).

**Upstream.** 0.1.1 `hooks/policy.py:748-775` (`_PolicyDecideHook.run` wraps the
whole bucket walk in `except Exception` → `HookResult(allow=False,
message=f"Internal policy error: {repr(e)}")`); 0.1.1
`local_connection.py:1124-1132` (confirmation path:
`except Exception: ... await self._send_tool_confirmation(step_update, False)`);
`:1236-1243` (tool-call path:
`except Exception as e: ... ToolResult(error=f"Internal SDK error: {e!r}")`);
`:1175-1185` (denial carries `res.message or "No reason provided"`); 0.1.9
`hook_router.py:309-311`. Tests: 0.1.1 `local_connection_test.py:4185-4216`,
`:4219-4249`, `:319-350`.

**(1) `src/hooks.rs:251-266` — change the return type so there is no `Result`
left to mis-handle.**

Before:
```rust
pub async fn dispatch_pre_tool_call(&self, tool_call: &ToolCall)
    -> Result<HookResult, anyhow::Error>
```
After:
```rust
/// Dispatches `pre_tool_call` to every registered hook, fail-closed.
///
/// A hook that returns `Err` or panics is treated as a DENY carrying the
/// failure text; it can never be observed as an ALLOW. Mirrors upstream
/// `_PolicyDecideHook.run` (0.1.1 policy.py:769-775).
pub async fn dispatch_pre_tool_call(&self, tool_call: &ToolCall) -> HookResult {
    let hooks = self.hooks.read().await.clone();
    for hook in &hooks {
        let fut = hook.pre_tool_call(tool_call);
        let res = match AssertUnwindSafe(fut).catch_unwind().await {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => {
                tracing::error!("pre_tool_call hook failed — failing closed: {e:#}");
                HookResult { allow: false,
                             message: format!("Internal policy error: {e:#}") }
            }
            Err(_) => {
                tracing::error!("pre_tool_call hook panicked — failing closed");
                HookResult { allow: false,
                             message: "Internal policy error: pre_tool_call hook panicked."
                                 .to_string() }
            }
        };
        if !res.allow { return res; }
    }
    HookResult { allow: true, message: String::new() }
}
```

Add at the top of `src/hooks.rs`: `use futures_util::FutureExt;` and
`use std::panic::AssertUnwindSafe;` (`futures_util::future::BoxFuture` is already
imported at `:9`). Use `{e:#}` (alternate Display = the anyhow context chain
joined with `": "`), **not** `{e:?}`, so the string stays single-line and
model-safe.

**(2) `src/hooks.rs`, after the `impl HookRunner` block (after `:335`) — the
shared gate, the only thing `local.rs` and `wasm.rs` call:**

```rust
/// Fail-closed pre-tool gate shared by every transport.
///
/// With no `HookRunner` registered the call is allowed, matching upstream's
/// `elif self._hook_runner:` at 0.1.1 local_connection.py:1097.
pub async fn gate_pre_tool_call(runner: Option<&HookRunner>, tool_call: &ToolCall)
    -> HookResult
{
    match runner {
        Some(r) => r.dispatch_pre_tool_call(tool_call).await,
        None => HookResult { allow: true, message: String::new() },
    }
}

/// Formats a denial the way upstream does (0.1.1 local_connection.py:1176-1177).
pub fn denial_message(res: &HookResult) -> String {
    let reason = if res.message.trim().is_empty() { "No reason provided" }
                 else { res.message.trim() };
    format!("Tool execution denied by hook policy: {reason}")
}
```

**(3a) `src/local.rs:1004-1032`** (built-in confirmation path) — replace the
`let mut allow = true;` block with a `gate_pre_tool_call` call whose `decision`
drives both the `pending_calls` insert and `ToolConfirmation.accepted`. The
`None` arm of `extract_builtin_tool_call` stays auto-approve — that is upstream's
`DEFAULT_HOST_TOOL_NAME` pre-request (0.1.1 `local_connection.py:1092-1096`) and
is correct; **leave it alone.** `ToolConfirmation` has no reason field
(`proto/localharness.proto` — only `trajectory_id`/`step_index`/`accepted`), so
log the message via `crate::hooks::denial_message(&decision)`; upstream says the
same at `:1124-1131`.

**(3b) `src/local.rs:1102-1139`** (custom-tool path) — replace
`let allow = ... .map_or(true, |res| res.allow);` and the fixed denial string:

```rust
let decision = crate::hooks::gate_pre_tool_call(hook_runner.as_ref(), &tc).await;
if !decision.allow {
    let err_msg = crate::hooks::denial_message(&decision);
    let denied_step = Step { /* unchanged */ error: err_msg.clone(), /* .. */ };
    let _ = step_tx_clone.send(Ok(denied_step));
    let resp_json = serde_json::to_string(&serde_json::json!({ "error": err_msg }))
        .unwrap_or_else(|_| "{\"error\":\"Tool execution denied by hook policy\"}".to_string());
    /* ... unchanged send ... */
    return;
}
```

`unwrap_or_else`, not `unwrap` — `Cargo.toml:52-55`.

**(3c)+(3d)** Apply 3a to `src/wasm.rs:643-670` and 3b to `src/wasm.rs:741-778`,
character-for-character identical apart from `crate::spawn_task` in place of
`tokio::spawn`.

**Compile notes**, verified by building the exact shapes against this repo's
`target/debug/deps` rlibs (rustc 1.94.1, edition 2024):
`AssertUnwindSafe<BoxFuture<'_, Result<HookResult, anyhow::Error>>>` implements
`Future`; `CatchUnwind` of it is `Send`, so the enclosing
`dispatch_pre_tool_call` future remains `Send` and still satisfies
`tokio::spawn`. `futures_util::FutureExt::catch_unwind` needs the `std` feature,
which is on by default for `futures-util = "0.3"` (`Cargo.toml:22`). `HookRunner`
is not a trait, so there is no object-safety concern; **`DynHook`'s signature is
untouched**, so no user hook impl and no `docs/hooks.md` example needs editing.

**Call sites.** `src/hooks.rs:251-266` · `:9-10` (imports) · `:335` (append the
two free functions) · `src/local.rs:1004-1032` · `:1102-1108` · `:1110-1139` ·
`src/wasm.rs:643-670` · `:741-747` · `:749-778` · `src/hooks.rs:584` (existing
test `test_dispatch_pre_tool_call_deny_short_circuits` — drop the `.unwrap()`) ·
`docs/hooks.md:181-183` (dispatch-semantics table).

**Tests.**
- `src/hooks.rs::test_dispatch_pre_tool_call_erroring_hook_denies` — a hook whose
  `pre_tool_call` returns `Err(anyhow!("boom"))` yields `allow == false` and
  `message.contains("boom")`. Ports the intent of 0.1.1
  `local_connection_test.py:4185-4216` (`test_tool_confirmation_crash_sends_rejection`).
- `::test_dispatch_pre_tool_call_panicking_hook_denies` — an `async fn
  pre_tool_call` that panics yields `allow == false`, `message.contains("panicked")`,
  and the test process does not abort.
- `::test_dispatch_pre_tool_call_error_before_policy_still_denies` — register an
  erroring hook, then a `PolicyEnforcer` built from
  `enforce(vec![allow_all()], None)`; assert `allow == false`. **Proves the chain
  can no longer be skipped past the enforcer.**
- `::test_gate_pre_tool_call_without_runner_allows` — pins parity with 0.1.1
  `local_connection.py:1097`.
- `::test_denial_message_carries_hook_reason` / `::test_denial_message_falls_back`
  — asserts the `"Tool execution denied by hook policy: <reason>"` shape and the
  `"No reason provided"` fallback (0.1.1 `:1176-1177`). **This is the
  unit-testable half of H10 and needs no mock harness.**
- `tests/policy_gating.rs` (new, **blocked on WP-2's mock gaining `toolCall` +
  `toolConfirmationRequest` frames** — `src/bin/mock_localharness.rs:99-150`
  currently emits only `stepUpdate`): port 0.1.1
  `local_connection_test.py:4185-4216`, `:4219-4249`, and `:319-350`
  (`test_tool_hook_deny`, whose `ToolResponse` JSON must contain the hook's own
  message, not a fixed string).
- When this lands, also port 0.1.1 `local_connection_test.py:2815-2860`
  (`test_decide_hooks_run_for_builtin_tools`) and `:4034-4076`
  (`test_denied_builtin_not_tracked`) — they defend the H7-narrow decline
  (§3.2) with tests rather than prose.

**Behaviour change.** A `pre_tool_call` hook that returns `Err` or panics now
DENIES the tool call instead of silently allowing it, and the denial message
shown to the model changes from the fixed `"Execution denied by hook policy"` to
`"Tool execution denied by hook policy: <hook's message>"`.

**Risk.** Public-API break on `HookRunner::dispatch_pre_tool_call` — every
`.await?`/`.unwrap()` on it stops compiling, which is the intended forcing
function; `cargo build` points at all six in-tree sites. **Do not keep the
`Result` for source compatibility:** that reintroduces exactly the shape that
produced the bug. Two behavioural hazards: `catch_unwind` cannot catch a panic
that unwinds across an `.await` inside a *separately spawned* task, only within
the awaited future — sufficient here because `DynHook::pre_tool_call` returns a
`BoxFuture` polled inline; and if the crate is ever compiled with
`panic = "abort"`, `catch_unwind` degenerates to a no-op, so the `Err`-arm
handling is what carries the guarantee — **keep both arms.**

---

### WI-22 (T6) — `ToolCall.server_name`; `ToolResult.{server_name, exception}`

**Defect.** `ToolCall` (`src/types.rs:346-358`) has `id, name, args,
canonical_path` and `ToolResult` (`:360-374`) has `name, id, result, error`. Two
gaps. (a) **No `server_name`:** upstream added it to both in 0.1.6 (0.1.9
`types.py:532` and `:557`; verified absent from `types.py` in 0.1.1–0.1.5) and it
is what `ToolExecutionError`'s third argument and S6's MCP targeting both need.
(b) **`ToolResult` discards the typed error:** `src/tools.rs:159-166` catches the
tool's `anyhow::Error` and keeps only `Some(e.to_string())`, so the original
chain is unrecoverable by the time `on_tool_error` runs. Upstream has carried
`exception: Exception | None = Field(default=None, exclude=True)` since 0.1.1
(`types.py:527`), populated at `tool_runner.py:308-313`, consumed at
`local_connection.py:1195`. **(b) is a port regression.**

**Change — `src/types.rs:346-374`.** Both structs gain `#[non_exhaustive]` plus
constructors:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    #[serde(default = "empty_args")]
    pub args: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_path: Option<String>,
    /// MCP server that owns this tool, when it is an MCP tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
}

fn empty_args() -> Value { Value::Object(serde_json::Map::new()) }

impl Default for ToolCall { /* args = empty_args(), rest None/empty */ }

impl ToolCall {
    /// Creates a tool call with no id, canonical path or server name.
    pub fn new(name: impl Into<String>, args: Value) -> Self {
        Self { name: name.into(), args, ..Self::default() }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ToolResult {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub error: Option<String>,
    /// MCP server that owns this tool, when it is an MCP tool.
    #[serde(default, skip_serializing_if = "Option::is_none")] pub server_name: Option<String>,
    /// The original error returned by the tool implementation. Never serialized.
    #[serde(skip)] pub exception: Option<std::sync::Arc<anyhow::Error>>,
}

impl ToolResult {
    pub fn success(name: impl Into<String>, value: Value) -> Self { /* .. */ }
    pub fn failure(name: impl Into<String>, error: impl Into<String>) -> Self { /* .. */ }
}
```

`#[serde(skip)]` implies `Default::default()` on deserialize, which is why
`Option<Arc<anyhow::Error>>` does not need `Deserialize`. `Clone` survives
because `Arc` is `Clone`. Neither struct derives `PartialEq` today, so nothing
breaks there.

**Why `#[non_exhaustive]`:** this is already a breaking release, and WP-7 will
add at least `ToolResult.media` (C15) and `ToolCall`'s coerced-args plumbing.
`#[non_exhaustive]` forbids downstream struct-literal construction *including*
`..Default::default()`, which is why the constructors ship in the same commit.
Verified affordable: `grep -rn "ToolResult {\|ToolCall {" examples/ tests/`
returns only `examples/leptos_ssr_axum`'s own `MessageBlock::ToolCall`/
`::ToolResult` enum variants (`examples/leptos_ssr_axum/src/types.rs:14,32`) — no
external crate in this repo constructs either struct. **If the maintainer rejects
`#[non_exhaustive]`, drop those two attributes and everything else stands.**

**`src/tools.rs:142-225`** — `process_tool_calls` must propagate both new fields.
`id: Some(call.id)` at `:153` is a partial move of one field;
`call.server_name.clone()` on a different field afterwards is legal. The error
arm at `:159-166`:

```rust
Err(e) => {
    let arc = std::sync::Arc::new(e);
    let msg = arc.to_string();               // MUST precede the move
    results.push(ToolResult {
        id: Some(call.id), name: call.name.clone(), result: None,
        error: Some(msg), server_name: call.server_name.clone(),
        exception: Some(arc),
    });
}
```

**Call sites** (every struct literal — the compiler finds them all, listed so the
reviewer can check the wasm fork was not missed):
`src/types.rs:346-374` · `src/tools.rs:152,160,186,194,205,216` ·
`src/local.rs:906-911` (built-in success), `:1142-1147` (custom-tool seed),
`:1422-1427` (`extract_tool_result`), `:1075-1080` (`tc` construction —
`server_name: None`; **no `server_name` exists on the 0.1.1 `ToolCall` wire
message**, verified against `upstream-0.1.1-localharness.proto:337-342` and
`upstream-0.1.9-localharness.proto:431-436`, neither has the field), `:1186-1191`,
and the nine `extract_builtin_tool_call` returns at
`:1302,1313,1324,1339,1351,1362,1375,1389,1399` · `src/wasm.rs:545-550`,
`:781-786`, `:1287-1292` (three `ToolResult` mirrors) and `:714-719`, `:825-830`,
`:1171,1182,1193,1204,1216,1227,1240,1254,1264` (eleven `ToolCall` mirrors) ·
`src/conversation.rs:520,552` · `src/hooks.rs:578,603` · `src/policy.rs:562`
(`make_tool_call` helper) · `src/types.rs:1004,1028,1042,1056` ·
`docs/tools.md` (the `ToolResult` field table).

**Tests.**
- `src/types.rs::test_tool_call_default_args_is_empty_object` — pins upstream's
  `args: dict[str, Any] = pydantic.Field(default_factory=dict)` (0.1.9
  `types.py:529`): `ToolCall::default().args == json!({})` and
  `from_str::<ToolCall>(r#"{"id":"1","name":"t"}"#)` yields `args == json!({})`
  rather than failing.
- `::test_tool_result_exception_not_serialized` — port of the intent of
  `exclude=True` (0.1.9 `types.py:556`): the serialized JSON contains no
  `exception` key and round-tripping yields `exception: None`.
- `::test_tool_result_server_name_round_trips` — serializes when `Some`, omitted
  when `None` (matching the convention already asserted at `src/types.rs:1017-1023`).
- `src/tools.rs::test_process_tool_calls_failure_carries_exception` — port of
  0.1.1 `tool_runner.py:308-313` plus its consumer test
  `local_connection_test.py:2755-2800` ("the hook should receive the original
  ValueError (not a RuntimeError wrapping the error string) so that
  isinstance-based dispatch works"): assert `result.error == Some("bad input")`
  **and** `result.exception.is_some()` with the same `to_string()`.
- `::test_process_tool_calls_propagates_server_name` — both arms.

**Behaviour change.** `ToolResult` gained a field that is never serialized and a
field that is always `None` until W6/WP-8 land, so nothing observable changes on
the wire or in `ChatResponse`. `ToolCall` deserialized from JSON without an
`args` key now yields `{}` instead of failing. Downstream struct-literal
construction of either type stops compiling.

**Risk.** (1) `#[non_exhaustive]` is a hard downstream break with no
`..Default::default()` escape hatch — **the three constructors must ship in the
same commit or downstream has no construction path at all.** (2)
`src/tools.rs:159-166`: bind the message before moving the error into the `Arc`.
(3) `#[serde(default = "empty_args")]` changes `ToolCall` deserialization from
"missing args is an error" to "missing args is `{}`" — intentional, matches
`default_factory=dict`, belongs in the changelog.

---

### WI-23 (H11) — `ToolExecutionError{message, tool_name, server_name, source}`

**Defect.** `Hook::on_tool_error` receives `&anyhow::Error` (`src/hooks.rs:57-59`,
mirrored on `DynHook` at `:126-129`). At the two dispatch sites the error is
reconstructed from a string with all provenance stripped: `src/local.rs:918`
`let err = anyhow!(err_msg);` (built-in tools — the tool name is right there in
`tc.name` and is discarded) and `src/local.rs:1160`
`dispatch_on_tool_error(&anyhow!(err_str.clone()))` (custom tools — `tc.name`
again discarded). `src/wasm.rs:557` and `:799` are identical. A hook cannot tell
which tool failed, cannot route by MCP server, and cannot recover the original
error type. Upstream passes
`types.ToolExecutionError(error_message, tool_name, server_name)` (0.1.9
`hook_router.py:270`), asserted by `hook_router_test.py:605-609`
(`assertIsInstance(…, types.ToolExecutionError)`, `tool_name == "run_command"`,
`server_name is None`) and `:645-653`. The 0.1.1 baseline also passed a typed
exception — `tool_error = result.exception or RuntimeError(result.error)` (0.1.1
`local_connection.py:1195-1197`), pinned by `local_connection_test.py:2755-2800`.

**Add to `src/error.rs`** (currently 35 lines; its only inhabitant,
`AntigravityError`, has zero references anywhere in `src/`, `examples/` or
`tests/`):

```rust
use std::sync::Arc;

/// Raised when a tool execution fails, carrying the tool's identity.
///
/// Port of upstream `types.ToolExecutionError` (0.1.9 types.py:815-826).
/// `Display` renders the bare message, matching `str(err)` in types_test.py:726.
#[derive(Debug, Clone)]
pub struct ToolExecutionError {
    message: String,
    tool_name: String,
    server_name: Option<String>,
    source: Option<Arc<anyhow::Error>>,
}

impl ToolExecutionError {
    pub fn new(message: impl Into<String>, tool_name: impl Into<String>,
               server_name: Option<String>) -> Self { /* source: None */ }
    pub fn with_source(mut self, source: Arc<anyhow::Error>) -> Self { /* .. */ }
    pub fn message(&self) -> &str { &self.message }
    pub fn tool_name(&self) -> &str { &self.tool_name }
    pub fn server_name(&self) -> Option<&str> { self.server_name.as_deref() }
    pub fn cause(&self) -> Option<&anyhow::Error> { self.source.as_deref() }
}

impl std::fmt::Display for ToolExecutionError { /* write!(f, "{}", self.message) */ }

impl std::error::Error for ToolExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref().map(|e| {
            let e: &(dyn std::error::Error + 'static) = &**e;
            e
        })
    }
}
```

**Add to `src/types.rs:335`** (immediately after
`impl std::error::Error for AntigravityExecutionError {}`):
`pub use crate::error::ToolExecutionError;` — so the upstream import path
`types.ToolExecutionError` has a Rust equivalent.

**Design notes the implementer must not re-derive.**
- Fields are private with accessors, unlike `AntigravityExecutionError`
  (`src/types.rs:322-326`), because `source: Option<Arc<anyhow::Error>>` must not
  be public — WP-8 will construct these from `OnToolErrorArgs` where no source
  exists, and a public field freezes that.
- Hand-written `Display`/`Error` rather than `thiserror`: `#[source]` requires the
  field type to implement `std::error::Error`, and `Arc<anyhow::Error>` does not.
  The coercion in `source()` works because
  `anyhow::Error: Deref<Target = dyn StdError + Send + Sync + 'static>`, so
  `&**e` is `&(dyn StdError + Send + Sync)`, which unsize-coerces to
  `&(dyn StdError + 'static)`.
- `Clone` is required: both dispatch sites move the value into a spawned task;
  `Arc` makes the source cheaply cloneable and `anyhow::Error` is not `Clone`.
- `Send + Sync` are automatic and **load-bearing**: the blanket
  `impl<T: Hook + ?Sized> DynHook for T` (`src/hooks.rs:150`) holds
  `&'a ToolExecutionError` across an await inside a `BoxFuture` that must be
  `Send`, which requires `ToolExecutionError: Sync`.

**Call sites.** `src/error.rs:35` (append) · `src/types.rs:335` (re-export) ·
`src/hooks.rs:6-9` (add the import) · `src/local.rs:917-922`, `:1160` ·
`src/wasm.rs:556-561`, `:799` (the latter two subsumed by WI-27's rewrite) ·
`docs/hooks.md:19,84-90,124,146` ·
`skills/google-antigravity-sdk-rust/references/error_handling.md:68-85,130-147` ·
`skills/google-antigravity-sdk-rust/examples/getting_started/hooks.md:46-61`.

**Tests.**
- `src/error.rs::test_tool_execution_error_basic_construction` — port of 0.1.9
  `types_test.py:722-728`.
- `::test_tool_execution_error_explicit_server_name` — port of `:730-739`.
- `::test_tool_execution_error_source_is_reachable` — no upstream equivalent
  (Python gets this from exception identity). Asserts
  `std::error::Error::source(&err).is_some()` and that
  `cause().map(ToString::to_string)` returns the original message, covering the
  0.1.1 regression that `local_connection_test.py:2755-2800` pins.

**Risk.** The `source()` coercion is the only line with a real compile hazard; if
`&**e` is rejected, the fallback is
`self.source.as_ref().map(|e| AsRef::<dyn std::error::Error + Send + Sync + 'static>::as_ref(e.as_ref()) as &(dyn std::error::Error + 'static))`
— anyhow implements both `AsRef` targets, so the turbofish is required to
disambiguate. Second: `pub use crate::error::ToolExecutionError;` inside
`src/types.rs` while `src/error.rs` does not import from `types` — no cycle
exists today (`src/error.rs` imports only serde and thiserror); **keep it that
way, and do not give `ToolExecutionError` a field typed from `crate::types`.**

---

### WI-24 — `src/tool_wire.rs`, the ungated module both transports delegate to

**Defect.** `src/wasm.rs` is a hand-copied fork of `src/local.rs` (X1). Every fix
in WI-25 … WI-27 has two edit sites: `on_tool_error` handling
(`src/local.rs:1159-1169` vs `src/wasm.rs:798-808`), argument parsing
(`src/local.rs:1074` vs `src/wasm.rs:713`), and three `ToolResponse`
constructions each (`src/local.rs:245,1127,1211` vs
`src/wasm.rs:766,849,1090`). Without a neutral home each is written twice and
drifts again.

**New file `src/tool_wire.rs`**; **new line in `src/lib.rs` immediately after
`pub mod tools;`** (currently `src/lib.rs:75`):
`pub(crate) mod tool_wire;`. Declaring it `pub(crate)` means this item adds
**zero public API**; every item inside is `pub(crate)`.

```rust
//! Transport-agnostic tool-call/tool-result plumbing shared by `src/local.rs`
//! (native) and `src/wasm.rs`. `src/lib.rs:67-71` cfg-gates those two modules
//! against each other, so code used by both must live here.

pub(crate) fn parse_tool_arguments(raw: Option<&str>) -> Result<Value, anyhow::Error>;
pub(crate) fn tool_result_to_response_json(result: &ToolResult) -> String;
pub(crate) fn build_tool_response(id: Option<String>, result: &ToolResult) -> ToolResponse;
pub(crate) fn tool_execution_error(result: &ToolResult) -> ToolExecutionError;
pub(crate) async fn finish_tool_result(runner: Option<&HookRunner>, result: &mut ToolResult);
pub(crate) async fn dispatch_builtin_tool_error(runner: &HookRunner, call: &ToolCall,
                                                message: &str);
```

**Send-ness.** Both call sites run inside `tokio::spawn` (`src/local.rs:1073`) /
`crate::spawn_task` (`src/wasm.rs:712`), and `crate::spawn_task`
(`src/lib.rs:83-96`) requires `F: Future<Output = ()> + Send + 'static` on
**both** cfgs, so the two async helpers must be `Send`. They are: `HookRunner` is
`Clone + Send + Sync` (`Arc<tokio::sync::RwLock<Vec<Arc<dyn DynHook>>>>`, and
`DynHook: Send + Sync` at `src/hooks.rs:101`), `ToolResult`/`ToolExecutionError`
are `Send + Sync` (WI-22/WI-23), and every `dispatch_*` in
`src/hooks.rs:249-334` drops its `RwLockReadGuard` inside the
`let hooks = …read().await.clone();` statement, so **no guard is held across an
await.**

`tool_execution_error(result)` builds from a `ToolResult`:
`ToolExecutionError::new(result.error.clone().unwrap_or_else(|| "Tool failed".to_string()),
result.name.clone(), result.server_name.clone())` then `.with_source(exc)` when
`result.exception` is `Some`. Upstream's default `"Tool failed"` is at 0.1.9
`hook_router.py:262`.

**Call sites.** `src/lib.rs:75` (insert after `pub mod tools;`; keep the
alphabetical block, and note it is **ungated**, unlike the `local`/`wasm` pair at
`:67-71`) · `src/tool_wire.rs` (new file).

**Tests.** `src/tool_wire.rs`'s `#[cfg(test)] mod tests` — the individual test
names are given under WI-25, WI-26 and WI-27. Add
`#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` at the top,
matching `src/hooks.rs:338-344`.

**Behaviour change.** None — pure scaffolding.

**Risk.** (a) `pub(crate)` items will trip `dead_code` if any function ends up
unused under a given cfg. Every function here is called from both `src/local.rs`
and `src/wasm.rs`, and both compile on the host under `cfg(test)`, so under
`cargo test` all are live; under a plain non-test host build only `local`
compiles and it calls all six. **Verify with `cargo build` (not just
`cargo test`) and with `cargo check --target wasm32-wasip1`.** (b)
`src/tool_wire.rs` imports `crate::proto::localharness::ToolResponse`, generated
into `OUT_DIR` by `build.rs` and **ungated** — confirmed by `src/lib.rs:42-60`,
which declares `pub mod proto` with no cfg.

---

### WI-25 (T5) — Absent or empty `arguments_json` must decode to `{}`; malformed must error

**Defect.** `src/local.rs:1074` (mirrored verbatim at `src/wasm.rs:713`):

```rust
let args: Value = serde_json::from_str(
    &tool_call.arguments_json.clone().unwrap_or_default()).unwrap_or(Value::Null);
```

(a) **Zero-argument tools:** the harness omits `argumentsJson` from the protojson
frame, prost gives `None`, `unwrap_or_default()` gives `""`, `from_str("")`
errors, and `unwrap_or` yields `Value::Null`. Every tool that does
`args.get("x")` or `args.as_object()` then sees `Null` —
`examples/custom_tools.rs` is the shape that breaks. Upstream is
`args = json.loads(tool_call.arguments_json or "{}")` in **both** versions: 0.1.1
`local_connection.py:1151` and 0.1.9 `event_processor.py:696` (also `:239` and
`:249` for the MCP/custom-tool StepUpdate paths). Python's `or` treats `""` as
falsy, which is exactly the empty-string case. **Port regression.**
(b) **Malformed JSON** is silently swallowed into `Null` and the tool runs with
garbage; upstream lets `json.loads` raise, the outer `except` catches it, and a
`ToolResult(error=f"Internal SDK error: {e!r}")` goes back to the model (0.1.9
`event_processor.py:733-740`; 0.1.1 `local_connection.py:1236-1243`).

**Add to `src/tool_wire.rs`:**

```rust
/// Decodes `ToolCall.arguments_json`.
///
/// Mirrors upstream `args = json.loads(tool_call.arguments_json or "{}")`
/// (0.1.9 event_processor.py:696, 0.1.1 local_connection.py:1151): an absent or
/// empty string decodes to an empty object, and anything that is not a JSON
/// object is an error — upstream's `types.ToolCall.args` is typed
/// `dict[str, Any]` (0.1.9 types.py:529), so a non-dict raises there too.
pub(crate) fn parse_tool_arguments(raw: Option<&str>) -> Result<Value, anyhow::Error> {
    let text = raw.filter(|s| !s.is_empty()).unwrap_or("{}");
    let value: Value = serde_json::from_str(text)
        .map_err(|e| anyhow::anyhow!("invalid arguments_json: {e}"))?;
    if !value.is_object() {
        anyhow::bail!("arguments_json must decode to a JSON object");
    }
    Ok(value)
}
```

(`Option::<&str>::filter` hands the closure a `&&str`; `s.is_empty()`
auto-derefs.)

**Replace `src/local.rs:1074` and `src/wasm.rs:713`** with a `match` whose `Err`
arm builds a failed `ToolResult`, emits one synthetic ERROR `Step`, sends the
`ToolResponse` via `crate::tool_wire::build_tool_response`, and `return`s.
**Ordering matches upstream:** the parse happens *before* the ACTIVE step is
enqueued (0.1.9 `event_processor.py:696-708`), so a malformed frame produces no
ACTIVE step. Emitting the single ERROR step is a Rust-only addition — upstream
has no synthetic steps at all — justified because this crate's consumers (e.g.
`examples/leptos_ssr_axum`) drive their UI off these steps and would otherwise
see a tool card that never resolves. `tool_call.arguments_json` is
`Option<String>`, so `.as_deref()` is the right adaptor.

**The error branch needs `counter`, `step_tx_clone`, `conn_ws_tx` and
`learned_id_clone` in scope**; all four are cloned into the task at
`src/local.rs:1068-1072` (and `src/wasm.rs:706-711`). But `synth_idx` is
currently bound *after* this point at `:1088`, so the early-return branch must
fetch its own index (`counter.fetch_add(1, Ordering::SeqCst)`) rather than
reference the later binding.

**Call sites.** `src/tool_wire.rs` · `src/local.rs:1074` (inside the
`tokio::spawn` opened at `:1073`) · `src/wasm.rs:713` (inside `crate::spawn_task`
opened at `:712`) · `src/local.rs:1075-1080` / `src/wasm.rs:714-719` (the
following `ToolCall` literal keeps `args: args.clone()`; no change beyond WI-22's
`server_name: None`).

**Tests** (all in `src/tool_wire.rs`; upstream has no dedicated test — the
behaviour is pinned by the expression at `event_processor.py:696` /
`local_connection.py:1151`):
`::test_parse_tool_arguments_absent_yields_empty_object` (`None` → `json!({})`) ·
`::_empty_string_yields_empty_object` (`Some("")` → `json!({})`, Python's falsy-
`""`) · `::_object_round_trips` (`Some(r#"{"a":5}"#)`, the shape at 0.1.9
`hook_router_test.py:978`) · `::_malformed_is_error` (`Some("{not json")`) ·
`::_non_object_is_error` (`Some("5")`, `Some("null")`, `Some("[1,2]")`, matching
`args: dict[str, Any]` at 0.1.9 `types.py:529`).

An end-to-end guard is worth adding only against a mock that answers the
handshake; **the existing `src/wasm.rs:1536-1591` mock is finding X12** (a closed
Rust→Rust loop encoding 0.1.1 semantics) and should not be extended. Keep the
coverage at the helper.

**Behaviour change.** Zero-argument tools now receive `{}` instead of `null` —
this **fixes** them rather than changing anything that worked. A tool call whose
`argumentsJson` is malformed (or not a JSON object) now returns
`Internal SDK error: …` to the model and emits an ERROR step, instead of silently
invoking the tool with `null` arguments.

**Risk.** `Step { .. ..Default::default() }` is fine because `Step` has a
hand-written `Default` (`src/types.rs:520-542`); `ToolCall`/`ToolResult` get
theirs from WI-22, which is why WI-22 must land first. Rejecting non-object args
is stricter than today's `Value::Null`; **if the maintainer judges it too
aggressive, drop the `is_object` check and the two `_non_object_is_error`
assertions — everything else stands unchanged.**

---

### WI-26 (W8) — Route all six `ToolResponse` constructions through one function

> **Correction to the audit.** This is **not** a defect against the pinned
> harness. `ToolResponse.error_message = 5` was added upstream in **0.1.6** —
> decoded from each wheel's `localharness_pb2.py` serialized descriptor,
> `ToolResponse` is `[id=1, response_json=2, supplemental_media=3, response=4]`
> in 0.1.1 through 0.1.5 and gains `error_message=5` in 0.1.6, 0.1.7 and 0.1.9.
> Upstream 0.1.1 encoded a failed tool exactly the way we do: `_tool_result_to_dict`
> returns `{"error": result.error}` and that becomes `response_json`
> (`local_connection.py:1243-1246`). The harness's Go protojson runs with
> `DiscardUnknown=false` (audit §2.1), so **emitting `errorMessage` at the 0.1.1
> pin would get the frame rejected and the socket closed.** The genuine defect
> today is only *duplication*: six copies of a rule the migration must edit in one
> place, plus one site (`src/local.rs:1127`) that bypasses the rule entirely with
> a string literal.

**Add to `src/tool_wire.rs`:**

```rust
/// Serialises a `ToolResult` into `ToolResponse.response_json`.
///
/// Mirrors upstream `_tool_result_to_dict` (0.1.1 local_connection.py:1243-1258,
/// unchanged as `tool_result_to_dict` at 0.1.9 event_processor.py:742-765): a
/// failed result serialises as `{"error": <msg>}`; a successful non-object
/// result is wrapped as `{"result": <value>}` because the Go harness requires an
/// object.
pub(crate) fn tool_result_to_response_json(result: &ToolResult) -> String {
    if let Some(err) = result.error.as_deref() {
        return serde_json::json!({ "error": err }).to_string();
    }
    match result.result.as_ref() {
        Some(v) if v.is_object() =>
            serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string()),
        Some(v) => serde_json::json!({ "result": v }).to_string(),
        None => "{}".to_string(),
    }
}

/// Builds the `ToolResponse` frame for a completed tool call.
pub(crate) fn build_tool_response(id: Option<String>, result: &ToolResult) -> ToolResponse {
    if result.error.is_some() {
        // W8 / 0.1.9 MIGRATION POINT — replace these two field values with
        //     response_json: None,
        //     error_message: result.error.clone(),
        // once `ToolResponse.error_message = 5` exists in proto/localharness.proto.
        // Upstream: 0.1.9 event_processor.py:778-782. Field 5 was added in 0.1.6;
        // the pinned 0.1.1 harness rejects unknown protojson fields, so it cannot
        // be emitted today (0.1.1 encoded the same failure as `{"error": ...}` in
        // response_json — local_connection.py:1243-1246).
        return ToolResponse {
            id,
            response_json: Some(tool_result_to_response_json(result)),
            supplemental_media: Vec::new(),
            response: None,
        };
    }
    ToolResponse {
        id,
        response_json: Some(tool_result_to_response_json(result)),
        // C15/WP-7 attaches extracted media here (event_processor.py:783-799).
        supplemental_media: Vec::new(),
        response: None,
    }
}
```

**The two arms are deliberately written out rather than collapsed:** the failure
arm is the one the migration edits, and having it already isolated is the whole
point.

**Rewrite the six sites.**
1. `src/local.rs:229-250` / `src/wasm.rs:1077-1100` (`send_tool_response`) — body
   becomes `let resp = crate::tool_wire::build_tool_response(Some(id.to_string()),
   &result);` plus the existing `InputEvent` wrap and
   `self.ws_tx.send(serde_json::to_string(&input_event)?)?`.
2. `src/local.rs:1127-1132` / `src/wasm.rs:766-771` (policy denial) — build a
   `denied` `ToolResult` carrying
   `error: Some("Execution denied by hook policy".to_string())` and call
   `build_tool_response`. **Keep the literal wording.** Replacing it with the
   hook's own `HookResult.message` — upstream formats
   `f"Tool execution denied by hook policy: {reason}"` at 0.1.1
   `local_connection.py:1176-1178` — is **H10**, delivered by WI-21; doing it here
   would make this item's behaviour-change surface non-empty for no reason. The
   `denied` `ToolResult` introduced here is exactly the value WI-21 mutates, so
   H10 becomes a one-line change too.
3. `src/local.rs:1195-1216` / `src/wasm.rs:836-856` (runtime result) — delete the
   ~14-line `let resp_json = if … else …` block (`:1197-1208`) and the literal at
   `:1211-1216`, replacing with
   `let resp = crate::tool_wire::build_tool_response(tool_call.id.clone(), &result);`.

**`proto/localharness.proto:415-420` — NO CHANGE in this item.**
`error_message = 5` is added by WP-1's regeneration; adding it early would let a
caller emit a field the pinned harness rejects.

**Call sites.** `src/tool_wire.rs` · `src/local.rs:229-250`, `:1127-1132`,
`:1195-1216` · `src/wasm.rs:1077-1100`, `:766-771`, `:836-856` ·
`docs/connections.md` (if it documents the `ToolResponse` shape).

**Tests** (all in `src/tool_wire.rs`):
`::test_tool_result_to_response_json_error_uses_error_key` — port of 0.1.1
`local_connection.py:1243-1246` · `::_error_wins_over_result` — the ordering
change this item introduces; upstream's `_tool_result_to_dict` checks `error`
first unconditionally · `::_wraps_non_object` —
`ToolResult::success("t", json!(42))` → `{"result":42}`; object results pass
through unwrapped (preserves the behaviour the comment at `src/local.rs:230-233`
describes) · `::_empty_is_empty_object` — neither set → `{}` (preserves
`src/local.rs:242-243`) ·
`::test_build_tool_response_failure_sets_response_json_at_current_pin` — asserts
the **current** wire encoding explicitly so the migration has to consciously flip
it: `response_json == Some(r#"{"error":"boom"}"#)`, `supplemental_media.is_empty()`,
`response.is_none()`. **Add a comment naming the 0.1.9 assertion that replaces it**
(`error_message == Some("boom")`, `response_json.is_none()`), citing 0.1.9
`event_processor.py:778-782`. · `::test_build_tool_response_preserves_id`.

**Behaviour change.** None at the current pin, with one exception: a `ToolResult`
carrying **both** a `result` and an `error` is now encoded as the error rather
than the result. No public signature changes; `Connection::send_tool_response`
keeps its `src/connection.rs:52-56` signature.

**Risk.** (1) `tool_result_to_response_json` is infallible where the current
`send_tool_response` propagates `serde_json::to_string` failures with `?`
(`src/local.rs:235,238,241`) — serialising an already-parsed `Value` cannot fail
in practice, and `unwrap_or_else` keeps `unwrap_used = "deny"` satisfied. The
`Result` return is retained only for the `to_string(&input_event)?` at the end.
(2) The error/result precedence **flips**: today a `ToolResult` carrying both is
sent as a success (`src/local.rs:229-243` checks `result` first). Post-WI-27 that
combination can arise, so the flip is required, not cosmetic. (3) **Do not add
`error_message` to the proto in this item** — the whole justification for its
pinned-harness safety is that the field is not emitted.

> **Sequencing option.** `tool_result_to_response_json` reads only `error` and
> `result`, and `build_tool_response` reads only `error` — neither actually needs
> WI-22's new fields. If the maintainer wants the migration seam early, WI-24 +
> WI-26 can be hoisted into 0.1.15 with a smaller diff (drop the `server_name`/
> `exception` initialisers from the denial `ToolResult`). The plan defaults to the
> reviewed dependency order.

---

### WI-27 (H4) — Narrow `on_tool_error` to a message transform

**Depends on WI-22, WI-23, WI-24.**

**Defect.** `src/local.rs:1159-1169`:

```rust
if let (Some(err_str), Some(runner)) = (result.error.as_ref(), hook_runner.as_ref()) {
    if let Ok((res, val)) = runner.dispatch_on_tool_error(&anyhow!(err_str.clone())).await {
        let allow_error = res.allow;
        if allow_error {
            result.result = val;      // hook's arbitrary JSON becomes the tool's output
            result.error = None;      // the failure is erased
        }
    }
} else if let Some(runner) = hook_runner.as_ref() {
    let _ = runner.dispatch_post_tool_call(&result).await;
}
```

Four distinct defects on seven lines.
**(H4)** A failed tool is reported to the model as a genuine success:
`result.error = None` makes `src/local.rs:1172` pick `StepStatus::Done` and
`:1195-1216` emit the hook's `Value` as `responseJson`. Upstream narrowed this in
0.1.6 — `OnToolErrorHook` "cannot fix or retry the tool call on its own" (0.1.9
`hooks/hooks.py:175-190`), only a non-empty stripped `str` is honoured, and it
becomes `OnToolErrorResult.custom_error_message`, the **only** field of that
message (`upstream-0.1.9-localharness.proto:558-560`;
`hook_router.py:279-290`).
**(H8)** `post_tool_call` is in the `else` arm, so it never sees a failed result.
**(H9)** `if let Ok(...)` silently discards an erroring hook — fixed by WI-15,
and made unrepresentable here.
**(H11)** the typed error is thrown away — fixed by WI-23.
`src/wasm.rs:798-808` is byte-identical.

**Provenance.** 0.1.1 genuinely did substitute a recovery result
(`local_connection.py:1207-1219`), so the port copied its source faithfully — but
0.1.1 emitted **no** synthetic terminal step (it enqueues only the ACTIVE step at
`:1155-1164`), so the `StepStatus::Done` downgrade at `src/local.rs:1170-1176` is
a **Rust-only invention** layered on top.

**(1) New outcome type in `src/types.rs`, adjacent to `HookResult`:**

```rust
/// Outcome of dispatching `on_tool_error` across the registered hooks.
///
/// Lossless mapping of upstream's `(HookResult, Any)` return from
/// `hook_runner.dispatch_on_tool_error` (0.1.9 hooks/hook_runner.py:234-259):
/// `allow=True` + non-empty string -> `CustomMessage`; the
/// `"Error recovery failed: …"` branch -> `HookFailed`; the `allow=False`
/// fall-through -> `Unchanged`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnToolErrorOutcome {
    /// No hook offered a replacement; the original error message stands.
    Unchanged,
    /// A hook returned a non-empty message; it replaces the error text.
    CustomMessage(String),
    /// A hook itself failed. The tool error still stands.
    HookFailed(String),
}
```

**(2) Trait, `src/hooks.rs:55-69`:**

```rust
// BEFORE
fn on_tool_error<'a>(&'a self, error: &'a anyhow::Error)
    -> impl Future<Output = Result<(HookResult, Option<serde_json::Value>), anyhow::Error>> + Send
{ async move { Ok((HookResult { allow: false, message: error.to_string() }, None)) } }

// AFTER
/// Invoked when a tool fails, allowing the hook to shape the failure message.
///
/// Return `Some(message)` to replace the text the model sees. The hook cannot
/// fix or retry the tool call: the tool still failed either way
/// (0.1.9 hooks/hooks.py:175-190). An empty or whitespace-only string is
/// ignored, as upstream ignores it at hook_router.py:280-284.
fn on_tool_error<'a>(&'a self, _error: &'a ToolExecutionError)
    -> impl std::future::Future<Output = Result<Option<String>, anyhow::Error>> + Send
{ async { Ok(None) } }
```

**(3) `DynHook`, `src/hooks.rs:125-129`:**
```rust
fn on_tool_error<'a>(&'a self, error: &'a ToolExecutionError)
    -> BoxFuture<'a, Result<Option<String>, anyhow::Error>>;
```
The blanket impl at `:173-178` keeps its one-line body
`Box::pin(async move { self.on_tool_error(error).await })`.

**(4) `HookRunner`, replacing `src/hooks.rs:276-294` wholesale — now
infallible**, which is what makes the H9 shape unrepresentable:

```rust
/// Dispatches `on_tool_error`, short-circuiting on the first non-empty message.
pub async fn dispatch_on_tool_error(&self, error: &ToolExecutionError)
    -> OnToolErrorOutcome
{
    let hooks = self.hooks.read().await.clone();
    for hook in &hooks {
        match hook.on_tool_error(error).await {
            Ok(Some(msg)) => {
                let trimmed = msg.trim();
                if !trimmed.is_empty() {
                    return OnToolErrorOutcome::CustomMessage(trimmed.to_string());
                }
            }
            Ok(None) => {}
            Err(e) => {
                tracing::error!("Critical failure in on_tool_error hook: {e}");
                return OnToolErrorOutcome::HookFailed(
                    format!("Error recovery failed: {e}"));
            }
        }
    }
    OnToolErrorOutcome::Unchanged
}
```

The `trim()`+empty check is `hook_router.py:280-284`
(`isinstance(recovery_val, str) and recovery_val.strip()` then
`custom_error_message=recovery_val.strip()`); the error branch is
`hook_runner.py:237-242`; the `tracing::error!` mirrors its `logging.exception`.
**WI-15's containment behaviour survives verbatim** — only the return type
changes.

**(5) Shared application logic in `src/tool_wire.rs`:**

```rust
pub(crate) async fn finish_tool_result(runner: Option<&HookRunner>,
                                       result: &mut ToolResult) {
    let Some(runner) = runner else { return };
    if result.error.is_some() {
        let err = tool_execution_error(result);
        match runner.dispatch_on_tool_error(&err).await {
            // H4: replace the message only. Never clear `error`; never move the
            // value into `result.result`.
            OnToolErrorOutcome::CustomMessage(msg) => result.error = Some(msg),
            OnToolErrorOutcome::Unchanged => {}
            OnToolErrorOutcome::HookFailed(msg) => tracing::warn!("{msg}"),
        }
    }
    // H8: post_tool_call sees every completed tool, success or failure
    // (0.1.9 hook_router.py:230-252).
    let _ = runner.dispatch_post_tool_call(result).await;
}

pub(crate) async fn dispatch_builtin_tool_error(runner: &HookRunner,
                                                call: &ToolCall, message: &str) {
    let err = ToolExecutionError::new(message, call.name.clone(),
                                      call.server_name.clone());
    match runner.dispatch_on_tool_error(&err).await {
        // The harness executes built-in tools; at the pinned 0.1.1 wire there is
        // no channel to return a message for one (ToolResponse correlates only
        // with client-side ToolCall ids). WP-8 turns this into
        // OnToolErrorResult.custom_error_message on the CallHookResponse.
        OnToolErrorOutcome::CustomMessage(m) =>
            tracing::debug!("on_tool_error message for built-in {} dropped: {m}",
                            call.name),
        OnToolErrorOutcome::Unchanged => {}
        OnToolErrorOutcome::HookFailed(m) => tracing::warn!("{m}"),
    }
    let failed = ToolResult { /* name, id, result: None, error: Some(message.to_string()),
                                server_name: call.server_name.clone(), exception: None */ };
    let _ = runner.dispatch_post_tool_call(&failed).await;
}
```

Borrow note: `tool_execution_error(result)` returns an **owned** value, so the
immutable borrow ends before `result.error = Some(msg)`, and `&err` is dropped
before the mutation. This compiles.

**(6) Call sites.** `src/local.rs:1159-1169` and `src/wasm.rs:798-808` both
collapse to one line:
`crate::tool_wire::finish_tool_result(hook_runner.as_ref(), &mut result).await;`
(`hook_runner` is `Option<HookRunner>` in both spawned tasks). `src/local.rs:916-923`
and `src/wasm.rs:555-562` become a spawned
`crate::tool_wire::dispatch_builtin_tool_error(&runner_clone, &call, &err_msg).await;`.

**`src/local.rs:1170-1176` and `src/wasm.rs:810-815` need NO edit:** because
`result.error` stays `Some`, the existing `if result.error.is_some()` already
yields `StepStatus::Error`. **The "never downgrade the step status" requirement
is satisfied by construction, not by a second guard.**

**Migration note for downstream implementors** (goes into `docs/hooks.md` and the
release notes verbatim):

> `Hook::on_tool_error` now takes `&ToolExecutionError` (carrying `tool_name()`
> and `server_name()`) and returns `Result<Option<String>, anyhow::Error>`. The
> hook can no longer substitute a result: **a failed tool stays failed.** Return
> `Some(msg)` to replace the message the model sees, `None` to leave it alone.
> Mechanical rewrite: `Ok((HookResult { allow: true, .. }, Some(json!(v))))`
> becomes `Ok(Some(v.to_string()))` — and if you were relying on the tool
> appearing to succeed, that behaviour is gone; move the fallback into the tool
> implementation itself, which is what upstream tells you to do (`hooks.py:185-189`:
> "To customize failure messages or provide recovery fallbacks for your tool, you
> can either return a custom error string from this hook or do so directly within
> your tool implementation"). `Ok((HookResult { allow: false, .. }, None))`
> becomes `Ok(None)`. Returning `Err(e)` now yields `Error recovery failed: {e}`
> in the log and leaves the tool's own error intact, instead of aborting the
> remaining hooks.

**Call sites.** `src/types.rs` (add `OnToolErrorOutcome`) · `src/hooks.rs:6-9`
(imports: add `ToolExecutionError` and `OnToolErrorOutcome`; `serde_json::Value`
may become unused in this file — **check before deleting**, `clippy::all` is deny)
· `:55-69` · `:125-129` · `:173-178` · `:276-294` · `:407-430`
(`TrackerHook::on_tool_error` in the module's own tests) · `:615-646`
(`test_dispatch_on_tool_error_recovery_short_circuits`, rewritten) ·
`src/tool_wire.rs` · `src/local.rs:916-923`, `:1159-1169` ·
`src/wasm.rs:555-562`, `:798-808` · `docs/hooks.md:19,80-90,124,146,183,516-559` ·
`skills/.../references/error_handling.md:68-85,130-147` ·
`skills/.../examples/getting_started/hooks.md:46-61` (all documentation edits are
WI-28).

**Tests.**
- `src/hooks.rs::test_dispatch_on_tool_error_custom_message_short_circuits` —
  replaces `src/hooks.rs:615-646`. Port of 0.1.9 `hook_runner_test.py:184-200`
  and `hook_router_test.py:659-698` (`test_on_tool_error_with_recovery`, which
  asserts `resp.on_tool_error_result.custom_error_message == "fallback value"`).
  Register h1/recover/h2; assert `CustomMessage("recovered")` and that only
  `["h1:on_tool_error", "recover:on_tool_error"]` ran.
- `::test_dispatch_on_tool_error_none_falls_through` — port of
  `hook_runner_test.py:384-400` and `hook_router_test.py:572-613`
  (`test_on_tool_error_no_recovery`, asserting `empty_result`). Assert
  `Unchanged`.
- `::test_dispatch_on_tool_error_no_hooks_registered` — port of
  `hook_router_test.py:699-726`. Empty runner; assert `Unchanged`.
- `::test_dispatch_on_tool_error_hook_error_is_contained` — port of 0.1.1
  `hook_runner_test.py:365-382` and its identical 0.1.9 twin. Must produce
  `HookFailed(msg)` where `msg.contains("Error recovery failed")`, and — the part
  upstream's test does not cover but our `?` bug required — **a later registered
  hook must NOT have run.**
- `::test_dispatch_on_tool_error_blank_message_is_ignored` — no upstream test;
  pinned by the `recovery_val.strip()` guard at `hook_router.py:280-284`.
  `Some("   ")` → `Unchanged`; `Some("  padded  ")` → `CustomMessage("padded")`.
- `::test_dispatch_on_tool_error_receives_tool_identity` — port of
  `hook_router_test.py:605-609` and `:645-653`.
- `src/tool_wire.rs::test_finish_tool_result_replaces_message_and_keeps_failure`
  — **the H4 regression guard, and the only test that would have caught the
  original bug.** Given `ToolResult { error: Some("boom"), .. }` and a hook
  returning `Some("try again with -f")`: assert
  `result.error == Some("try again with -f")`, `result.result.is_none()`, and
  that `result.error.is_some()` still holds — the predicate `src/local.rs:1172`
  branches on.
- `::test_finish_tool_result_dispatches_post_tool_call_on_failure` — H8. Port of
  0.1.9 `hook_router_test.py:540-563`, which asserts the PostToolCall hook
  receives `error == "Command failed"` and `result is None`.
- `::test_finish_tool_result_success_path_dispatches_post_only` — `on_tool_error`
  is not called when `result.error.is_none()`.
- `::test_dispatch_builtin_tool_error_carries_tool_name` — port of 0.1.1
  `local_connection_test.py:3984-4032`
  (`test_on_tool_error_dispatched_for_builtin_error`: STATE_ERROR with
  `error_message="Permission denied"` reaches the hook), upgraded to the 0.1.9
  assertion that the error is a `ToolExecutionError` whose `tool_name()` is the
  built-in's name.

**Behaviour change.** Three visible changes. (a) A tool that fails and whose
`on_tool_error` hook returns a recovery value is now reported to the model as
**failed**, with the hook's string as the error message, and its synthetic step is
`StepStatus::Error` rather than `StepStatus::Done`. (b) `post_tool_call` now fires
for failed tools as well as successful ones. (c) A hook that returns `Err` no
longer aborts the remaining `on_tool_error` hooks and no longer disappears
silently.

**Risk.** Compile hazards, in order of likelihood. (1) `Hook` is not object-safe
(RPITIT, `src/hooks.rs:16`) — only `DynHook` is dispatched dynamically, so the
trait, the `DynHook` mirror **and** the blanket impl at `:150-201` must change
together or the blanket impl fails to satisfy `DynHook`. (2) The `BoxFuture` in
the blanket impl must be `Send`; holding `&'a ToolExecutionError` across the await
requires `ToolExecutionError: Sync`, which WI-23 guarantees. (3)
`dispatch_on_tool_error` losing its `Result` breaks all four production call sites
plus the test — a missed site is a compile error, not a silent bug, which is the
point. (4) `serde_json::Value` may become an unused import in `src/hooks.rs`.
**Semantic hazard:** `finish_tool_result` now calls `dispatch_post_tool_call` on
paths that previously skipped it, so any existing `post_tool_call` hook starts
seeing failed results — a hook that does `result.result.unwrap()` will newly
panic. That is the intended H8 forward-port and belongs in the release notes.

---

### WI-28 — Rewrite the `on_tool_error` documentation

**Defect.** Five documents present the pre-WI-27 contract as working design, and
one presents it as a recommended pattern. `docs/hooks.md:19` classifies
`on_tool_error()` as returning "recovery data"; `:80-90` reproduces the old
default body; `:124` tabulates the return as `(HookResult, Option<Value>)`
short-circuiting on "first `allow: true`"; `:146` reproduces the old `DynHook`
signature; `:183` repeats the short-circuit rule; and **`:516-559` is a worked
"### 4. Error Recovery Hook" example whose whole point is returning
`Some(json!({...}))` so a failing tool looks like it succeeded.**
`skills/google-antigravity-sdk-rust/references/error_handling.md:68-85` and
`:130-147` and
`skills/google-antigravity-sdk-rust/examples/getting_started/hooks.md:46-61`
reproduce the same signature.

This matters more than usual because **CI never compiles any of it**:
`.github/workflows/ci.yml:38` runs `cargo test --all-targets`, which excludes
doctests, and `grep -rn include_str src/` is empty (X3) — `src/lib.rs:1-39` has
no `#![doc = include_str!]`. A stale example survives green CI indefinitely,
which is precisely how `docs/hooks.md:516-559` came to teach a behaviour the SDK
is about to remove. The skill files are consumed by an agent, so a stale
signature there produces confidently wrong generated code.

**Change — `docs/hooks.md`:**
- `:19` — Transform row becomes: `` `OnToolErrorHook`, `OnInteractionHook` |
  `on_tool_error()` replaces the failure message; `on_interaction()` answers
  questions ``.
- `:80-90` — replace the excerpt with WI-27's new default body, keeping the
  upstream citation (0.1.9 `hooks/hooks.py:175-190`) in the doc comment.
- `:124` — new row:
  `` | `on_tool_error` | Transform | `Option<String>` | Yes — first non-empty message | ``.
- `:146` — new `DynHook` line:
  `fn on_tool_error<'a>(&'a self, error: &'a ToolExecutionError) -> BoxFuture<'a, Result<Option<String>, anyhow::Error>>;`
- `:183` — new row: `` | `dispatch_on_tool_error` | **Short-circuits** at the
  first non-empty message; a hook returning `Err` stops the chain and yields
  `Error recovery failed: …` | ``.
- `:516-559` — replace "### 4. Error Recovery Hook" with **"### 4. Error Message
  Hook"**, a `rust,no_run` example whose `on_tool_error` returns
  `Ok(Some(format!("`{}` could not reach the network. Retry, or use cached data.",
  error.tool_name())))` for timeout-shaped messages and `Ok(None)` otherwise,
  followed by: *"The hook cannot fix or retry the tool call. The tool still
  failed; only the text the model sees changes. To provide a fallback value,
  return it from the tool implementation itself."* — paraphrasing 0.1.9
  `hooks/hooks.py:180-189`.
- Add a **"Migration: `on_tool_error` (0.x)"** subsection carrying WI-27's
  migration note verbatim.

**`skills/.../references/error_handling.md:68-85` and `:130-147`, and
`skills/.../examples/getting_started/hooks.md:46-61`** — replace the RPITIT-style
signature blocks with the new one (these files use the explicit
`fn … -> impl Future` form rather than `async fn`, so **keep that style**), and
fix their `use` lines to import `ToolExecutionError` from
`antigravity_sdk_rust::types` (the re-export added by WI-23).

**NO CHANGE needed** — verified: `README.md:235-253` (`impl Hook for MyHook`) does
not override `on_tool_error`; `docs/agent.md:436` (`impl Hook for AuditHook`)
likewise; `skills/.../references/architecture.md:12` only lists hook names. The
four `impl Hook for` sites in the tree are `src/policy.rs:374`,
`examples/agent_server/src/main.rs:88`, `examples/subagents.rs:14`,
`src/hooks.rs:354`.

**Optional but recommended in the same commit** (closes the mechanism that let
this rot): add `#![doc = include_str!("../docs/hooks.md")]` behind a
`#[cfg(doctest)] mod doc_tests {}` shim, or add `cargo test --doc` to
`.github/workflows/ci.yml`. That is finding X3 and can be deferred, but without
it these files drift again at the next signature change.

**Call sites.** `docs/hooks.md:19,80-90,124,146,183,516-559` ·
`skills/google-antigravity-sdk-rust/references/error_handling.md:68-85,130-147` ·
`skills/google-antigravity-sdk-rust/examples/getting_started/hooks.md:46-61` ·
`.github/workflows/ci.yml` (optional `cargo test --doc` step).

**Tests.** No unit tests. Verification is `cargo test --doc` if the optional CI
step lands; otherwise a manual grep gate:
`grep -rn 'HookResult, Option<serde_json::Value>' docs/ skills/ README.md` must
return nothing after the change.

**Behaviour change.** None — documentation only. **Risk:** only that it is
skipped.

---

### WI-29 (H1b) — Dispatch `post_turn`; change `Hook::post_turn` to take `&str`

**Depends on WI-17, WI-18.**

**Defect.** `HookRunner::dispatch_post_turn` (`src/hooks.rs:319-325`) has no
production caller. Upstream 0.1.1 fires it on the terminal model step addressed
to the user (`local_connection.py:596-610`). Separately, the Rust signature
`post_turn(&self, response: &ChatResponse)` (`src/hooks.rs:88-93`) is
**unimplementable at the only place the data exists**: the WS reader loop has a
single `StepUpdate`, not a `ChatResponse` (no session history, no cumulative
usage). Upstream passes a plain string in every version — 0.1.1
`dispatch_post_turn(turn_context, step_obj.content or "")`
(`local_connection.py:607-608`), 0.1.9
`dispatch_post_turn(turn_ctx, req.post_turn_args.response_text)`
(`hook_router.py:169`; `response_text` is read at `:164-165`).

**Change.** `&'a ChatResponse` → `&'a str` at four sites (trait `:88-93`, DynHook
`:141-144`, blanket impl `:191-196`, dispatcher `:319-325`) — shape-identical,
keeps `DynHook` object-safe, and `BoxFuture<'a,..>` stays `Send` because
`str: Sync`. Dispatch from the reader loop after
`let _ = step_tx.send(Ok(step));` (`src/local.rs:877`, `src/wasm.rs:516`), gated
on `crate::hook_dispatch::is_turn_terminal_step(status, source, target)` and a
`post_turn_pending` `AtomicBool` latch set by `send()` and cleared with
`compare_exchange` (upstream's once-per-turn guard is the
`_current_turn_context = None` at `local_connection.py:610`).

`source`/`status`/`target` are computed at `src/local.rs:812-832` and
`src/wasm.rs:451-471`, i.e. **in scope**. `AtomicBool`/`Ordering` are imported at
`src/local.rs:35` and `src/wasm.rs:12`. Placement before the system-error break at
`src/local.rs:880-887` and the TERMINAL_ERROR break at `:890-897` correctly
mirrors upstream, which dispatches `post_turn` at `local_connection.py:604-610`
before raising `AntigravityExecutionError` at `:615-618`. The `step` must be
cloned before `step_tx.send` (the send moves it) — cheap, `Step` is plain data.

**Divergence to state explicitly in a code comment:** upstream **awaits**
`post_turn` inline (0.1.1 `local_connection.py:607`, 0.1.9
`hook_router.py:169`); this crate **spawns** it so a slow user hook cannot stall
the WS reader. Consequence: `post_turn` is not ordered against subsequent steps
and may run after `disconnect()` returns. *If ordering matters more than reader
liveness, await it inline instead.*

**Call sites.** `src/hooks.rs:6-8` (imports: drop `ChatResponse`, add `Step` — see
WI-30's interlock) · `:88-93` · `:141-144` · `:191-196` · `:319-325` · `:346`
(test import — `UsageMetadata` exists solely for `:717`) · `:460-466`
(`TrackerHook` impl) · `:713-719` (test body constructing a `ChatResponse`) ·
`src/local.rs:877`, `:702` + `:1262` (`post_turn_pending` plumbing, shared with
WI-18), `:159-184` (send sets the latch) · `src/wasm.rs:516`, `:353` + `:904`,
`:1007-1032` · `docs/hooks.md:62,121,149,180,421-423` (`response.text.len()` →
`response_text.len()`) · `docs/agent.md:442-445` — the `AuditHook` example prints
`response.usage_metadata.total_token_count`; **that field no longer exists on the
parameter** — rewrite to print the response text and point at
`Conversation::last_turn_usage` for tokens · `README.md:244`.
**No example implements `post_turn`** — verified; none of the four
`impl Hook for` sites overrides it, so nothing in `examples/` or `tests/` breaks.
`README.md`'s block is ```rust,no_run``` but README is **not** wired into the
crate docs and CI excludes doctests, so it is a documentation-only edit.

**Tests.** **Every post_turn test needs an explicit rendezvous** — the dispatch is
`tokio::spawn`, so "run one chat then assert" is a race. Have the hook send on a
`tokio::sync::mpsc` or set a `tokio::sync::Notify`, and assert under
`tokio::time::timeout(Duration::from_secs(1), ..)`, mirroring upstream's
`asyncio.Event` + `wait_for` at `local_connection_test.py:2320`/`:2349`.

- `tests/integration_tests.rs::test_post_turn_hook_dispatched_on_final_step` —
  port of 0.1.1 `local_connection_test.py:2120-2172`. The existing
  `src/bin/mock_localharness.rs` already emits a matching frame at `:131-145`
  (step 2, SOURCE_MODEL, TARGET_USER, STATE_DONE, text
  `"Hello from mock harness!How can I help you today?"`). Assert exactly one
  capture equal to that text.
- `::test_post_turn_hook_not_fired_for_environment_step` — port of `:2241-2296`.
  Requires a mock branch emitting a TARGET_ENVIRONMENT DONE step first
  (`src/bin/mock_localharness.rs` already has a prompt-keyword branch pattern at
  `:97`).
- `::test_receive_steps_includes_target_environment` — port of `:2174-2240`,
  alongside the negative test: it proves TARGET_ENVIRONMENT steps still reach the
  consumer while not firing `post_turn`, **so the new predicate cannot be
  implemented by filtering the stream.**
- `::test_post_turn_fires_once_per_turn` — NEW; two consecutive terminal
  TARGET_USER model steps in one turn produce exactly one dispatch (pins the
  `compare_exchange` latch).
- `::test_post_turn_not_fired_on_active_step` — the mock's step 1
  (`src/bin/mock_localharness.rs:115-127`, STATE_ACTIVE, SOURCE_MODEL,
  TARGET_USER) must not fire; pins that `is_turn_terminal_step` excludes
  `StepStatus::Active`, which the deliberately-unused `is_complete` flag at
  `src/local.rs:842-847` would have conflated.
- `::test_post_turn_hook_error_is_logged_not_fatal` — a hook returning `Err` does
  not terminate the step stream. Upstream analogue: `hook_router.py:307-311`.
- `src/hooks.rs` — port 0.1.1 `hook_runner_test.py:316-329`
  (`test_dispatch_post_turn`) as the replacement for `:705-723`; it dispatches a
  plain string, which is exactly the post-`&str` shape and removes the
  `ChatResponse`/`UsageMetadata` construction.
- WASM half: extend `src/wasm.rs:1534-1640`'s mock with a STATE_DONE/TARGET_USER
  step and a `hook_runner`, asserting the same capture.

**Behaviour change.** `post_turn` starts firing (it never did). Its parameter
changes from `&ChatResponse` to `&str`. Migration: replace
`async fn post_turn(&self, r: &ChatResponse)` with
`async fn post_turn(&self, response_text: &str)`; `r.text` becomes
`response_text`; `r.usage_metadata` and `r.steps` are no longer available on the
parameter — read them from `Conversation::last_turn_usage()` /
`Conversation::history()`. **Compile error, not a silent change.**

**Risk.** Dropping `ChatResponse` from the `src/hooks.rs` import is mandatory;
leaving it fails CI on `unused_imports` under `-D warnings`. The spawned task
captures `HookRunner` (Clone, holds `Arc<tokio::sync::RwLock<Vec<Arc<dyn DynHook>>>>`)
and a `String` — both `Send + 'static`, so `tokio::spawn` and `crate::spawn_task`
(`src/lib.rs:84-96`) both accept it.

**This `&str` signature is chosen to be the LAST one.** 0.1.9's
`PostTurnArgs.response_text` delivers exactly this, so making the change now
avoids breaking users twice.

---

### WI-30 (H1d + H16) — Dispatch `on_compaction`; change the parameter to `&Step`

**Depends on WI-17, WI-29** (shares the `src/hooks.rs` import edit).

**Defect.** Two defects at one site. **(1)** `HookRunner::dispatch_on_compaction`
(`src/hooks.rs:328-334`) has no production caller, although the reader loop
already classifies compaction steps at `src/local.rs:786-787` /
`src/wasm.rs:425-426`. Upstream 0.1.1 dispatches from the reader loop on every
COMPACTION step (`local_connection.py:800-806`). **(2) (H16)** the Rust callback
takes `&str` (`src/hooks.rs:95-100`) where upstream passes the compaction Step
object — `dispatch_compaction(self._get_turn_context(), step_obj)` at
`local_connection.py:803-804`, and 0.1.1's own test asserts
`assertIsInstance(captured[0], local_connection.LocalConnectionStep)` and reads
`.content` (`local_connection_test.py:2351-2353`). With `&str` the hook cannot
see `step_index`, `trajectory_id`, `usage_metadata` or `status`.

**Change.** `&'a str` → `&'a Step` at trait `:95-100`, `DynHook` `:147`, blanket
impl `:198-200`, dispatcher `:328-334`. Keeps `DynHook` object-safe and keeps
`BoxFuture<'a,..>` `Send` because `Step` is `Sync` (all owned data plus
`Option<serde_json::Value>`, and `Value` is `Send + Sync`;
`src/types.rs:478-518`). Dispatch from the reader loop inside the same hook block
as WI-29 (`src/local.rs:877`, `src/wasm.rs:516`), gated on
`step.r#type == StepType::Compaction`. `StepType` is imported at
`src/local.rs:24` and `src/wasm.rs:40`. The frame survives to that point: a
SOURCE_SYSTEM/STATE_DONE compaction step trips neither the http-code break at
`src/local.rs:880-887` nor the TERMINAL_ERROR break at `:890-897`, and
`pending.remove(&key)` at `:902` yields `None`.

Add a one-line note that **the hook receives a CLONE of the `Step`**, so mutating
it is impossible and the hook is observe-only — which is what upstream means by
"observe-only" at `local_connection.py:800`, and what makes the extra
`step_for_hooks.clone()` unavoidable rather than merely cheap.

**Call sites.** `src/hooks.rs:6-8` (add `Step` — **in the SAME edit as WI-29's
removal of `ChatResponse`**, otherwise one of the two intermediate states fails to
compile) · `:95-100` · `:147` · `:198-200` · `:328-334` · `:468-474`
(`TrackerHook` impl) · `:742` (test body) · `src/local.rs:877` ·
`src/wasm.rs:516` · `docs/hooks.md:108,126,150,185` · `README.md:253`.
**No example or test implements `on_compaction`** — verified;
`examples/leptos_ssr_axum/src/app.rs:2455,2783` are a DOM `EventListener` for a
"compaction" browser event in a separate workspace crate, entirely unrelated.

**Tests.** Same rendezvous requirement as WI-29 — upstream's test explicitly
synchronises (`local_connection_test.py:2320` `event = asyncio.Event()`, `:2326`
`event.set()` inside the hook, `:2349` `await asyncio.wait_for(event.wait(),
timeout=1.0)` before the assertions at `:2351-2353`). Porting the assertions
without porting the wait produces a flaky test. The negative test needs a bounded
wait that expects a timeout, not an immediate `assert!(captured.is_empty())`.

- `tests/integration_tests.rs::test_compaction_step_dispatches_hook` — port of
  0.1.1 `local_connection_test.py:2317-2353`. Add a prompt-keyword branch to
  `src/bin/mock_localharness.rs` (same pattern as the `trigger_terminal_error`
  branch at `:97-108`) emitting
  `{"stepUpdate":{"stepIndex":1,"text":"Context compaction","state":"STATE_DONE","source":"SOURCE_SYSTEM","target":"TARGET_USER","compaction":{}}}`;
  assert the hook received exactly one `Step` with
  `r#type == StepType::Compaction` and `content == "Context compaction"` —
  upstream's assertions verbatim, **including its isinstance check, which is what
  H16 exists to make expressible.**
- `::test_on_compaction_not_fired_for_ordinary_step` — a plain text step fires
  nothing.
- `::test_compaction_step_still_reaches_receive_steps_and_history` — the
  compaction step must both fire the hook AND land in `Conversation` history;
  `src/conversation.rs:168-171` pushes its index into `compaction_indices`, so a
  regression that swallows the step to feed the hook would be caught.
- Port 0.1.1 `local_connection_test.py:2879-2905`
  (`test_compaction_hooks_no_longer_raise`): a compaction hook that throws must
  not kill the connection. **This is the exact guarantee the `tracing::warn!`
  inside the spawned task provides, and nothing else pins it.**
- `src/hooks.rs` — port 0.1.1 `hook_runner_test.py:202-217`
  (`test_dispatch_compaction`) as the replacement for `:726-746`.
- WASM half: same fixture through the `src/wasm.rs:1534-1640` mock.

**Behaviour change.** `on_compaction` starts firing (it never did) and its
parameter changes from `&str` to `&Step`. Migration: the old `summary` is
`step.content`. Compile error, not a silent change.

**Risk.** `step_for_hooks.clone()` inside the spawn is a second clone of the
`Step` — negligible, and required because the same binding also feeds WI-29's
block. The spawned future captures `HookRunner` + `Step`; `Step` derives
`Clone`/`Serialize`/`Deserialize` and contains only owned data, so it is
`Send + 'static`. The compaction predicate depends on `StepType::Compaction` being
assigned at `src/local.rs:786-787`, which requires the proto to still carry
`StepUpdate.compaction` — it does, unchanged in 0.1.9.

---

### WI-31 (A2) — `ChatResponse.usage_metadata` must report the last turn, not the session total

**Defect.** `src/conversation.rs:307` — `let usage_metadata = self.total_usage().await;`,
where `total_usage()` (`:116-118`) returns `state.cumulative_usage`, accumulated
over the whole session at `:173-180` and reset only by `clear_history()` (`:131`).
So the **second `chat()` of a session reports turn 1 + turn 2 tokens as if they
were turn 2's.** Upstream returns the per-turn figure: `ChatResponse.usage_metadata`
is `return self._conversation.last_turn_usage` (0.1.1 `types.py:964-967`) /
`self._conversation._last_turn_usage` (0.1.9 `types.py:1025-1028`), and `send()`
resets `_turn_usage = None` every turn (`conversation.py:136` / `:133`).
**The per-turn value is ALREADY tracked correctly in Rust** at
`src/conversation.rs:182-189` and exposed at `:121-123` — only the wiring in
`chat_to_completion` is wrong. Upstream also returns `None` when no step reported
usage; Rust returns an all-zero struct, indistinguishable from a genuine zero.

**Change.**
1. `src/types.rs:609`: `pub usage_metadata: UsageMetadata,` →
   `pub usage_metadata: Option<UsageMetadata>,` (matches upstream's
   `UsageMetadata | None`).
2. `src/conversation.rs:307`: `self.total_usage().await` →
   `self.last_turn_usage().await` (already returns `Option<UsageMetadata>`; no
   other change at `:308-313`).
3. `src/interactive.rs:83-89`: `let usage = &response.usage_metadata;` →
   ```rust
   if let Some(usage) = response.usage_metadata.as_ref() {
       if usage.total_token_count > 0 { println!(...); }
   }
   ```
4. `src/hooks.rs:717` (unit-test fixture): `usage_metadata: UsageMetadata::default(),`
   → `usage_metadata: None,` (and drop the `UsageMetadata` import if it becomes
   unused).

Leave `Conversation::total_usage()` (session cumulative, upstream's `total_usage`
property at `conversation.py:314-321`) and `Conversation::last_turn_usage()`
public and unchanged.

**Explicitly out of scope:** `ChatResponse.steps` still carries the whole session
(A19). Do not fix it here — separate finding, and the `turn_start_indices` slicing
needs its own test.

**Non-breaking alternative** if you must avoid the semver bump: keep the field as
`UsageMetadata` and write `self.last_turn_usage().await.unwrap_or_default()`. This
fixes the cumulative-vs-turn bug but keeps zero-vs-absent ambiguous. **Prefer the
`Option` form** and bundle it with the other breaks.

**Call sites.** `src/types.rs:609` · `src/conversation.rs:307` ·
`src/interactive.rs:83-89` · `src/hooks.rs:717` · `docs/agent.md:267,381,443` ·
`docs/conversation.md:107,333`. **No `examples/` call site:** grep for
`response.usage_metadata` across `examples/` returns nothing
(`examples/agent_server/src/main.rs:628` reads `step.usage_metadata`, unrelated
and already `Option`).

**Tests.**
- Port 0.1.1 `conversation_test.py:916-938`
  (`test_chat_returns_accumulated_usage_metadata`; 0.1.9 `:902-924`) — two steps
  in one turn, assert `usage_metadata` sums to prompt 300 / candidates 80 /
  total 380.
- Port `:940-951` (`test_chat_returns_none_usage_when_absent`; 0.1.9 `:926-937`)
  — no step reports usage, assert `response.usage_metadata.is_none()`.
- `chat_usage_is_per_turn_not_cumulative` — **NEW; this is the actual
  regression**, and upstream has no direct analogue because its property was
  always per-turn. Turn 1 reports prompt=100, turn 2 reports prompt=150; assert
  `response2.usage_metadata.unwrap().prompt_token_count == 150` while
  `conv.total_usage().await.prompt_token_count == 250`. The second half is
  upstream `conversation_test.py:866-888`
  (`test_total_usage_accumulates_across_turns`; 0.1.9 `:852-874`), which must keep
  passing.
- Requires the same `MockConnection` `mem::take` change described in WI-13 to
  script two distinct turns; if WI-31 ships first, add it here.

**Behaviour change.** `ChatResponse.usage_metadata` becomes
`Option<UsageMetadata>` and reports only the turn that `chat()` just ran, instead
of the session total. Migration: for the previous value use
`conversation.total_usage().await`; for the new value with a non-`Option` shape
use `response.usage_metadata.unwrap_or_default()`.

**Risk.** Purely a call-site sweep; the compiler finds every site. The one
behavioural trap: code that did `response.usage_metadata.total_token_count` to
bill a whole session now gets a per-turn number — the point of the fix, but it
belongs in the changelog under **BREAKING**, not under fixes. Confirm
`Conversation::clear_history` (`:126-133`) still zeroes both counters; it does.

**Do NOT apply A18** (making `Conversation::last_turn_usage` private, as 0.1.4
did) before this lands — it is currently the only way to obtain per-turn usage.

---

### WI-32 (T7 + T8) — Reject duplicate tool names; make the registry insertion-ordered

**Depends on WI-14** (the deleted web-search arm sits inside the method being
rewritten).

**Defect.** Two defects in one data structure. **(T7)** `ToolRunner::register`
(`src/tools.rs:134-139`) is `pub async fn register(&self, tool: Arc<dyn DynTool>)`
and does a bare `HashMap::insert`, so registering two tools with the same `name()`
**silently drops the first**. Upstream raises:
`if tool_name in self._tools: raise ValueError(f"Tool '{tool_name}' is already registered.")`
(0.1.9 `tool_runner.py:183-184`; identical at 0.1.1 `:164-165`).
**(T8)** Tools live in
`pub tools: Arc<tokio::sync::RwLock<HashMap<String, Arc<dyn DynTool>>>>`
(`src/tools.rs:114`) and the `HarnessConfig.tools` list is built by iterating
`tools.values()` (`src/local.rs:515`, `src/wasm.rs:151`).
`std::collections::HashMap` iteration order is randomised per process by the
default `RandomState` hasher, so **the tool list handed to the model differs on
every run of the same program** — non-reproducible prompts, non-reproducible model
tool-selection, and a diff-unstable init frame that makes the mock harness
impossible to assert against byte-for-byte. Upstream's `self._tools` is a Python
`dict`, insertion-ordered by language guarantee since 3.7, and `tool_names`
returns `list(self._tools.keys())` (`tool_runner.py:207-209`) — order is part of
the contract (`tool_runner_test.py:53`, `:65` assert exact list equality).

**Replace the storage.**

```rust
/// One registered tool plus its pre-parsed parameter schema.
#[derive(Clone)]
struct ToolEntry {
    tool: Arc<dyn DynTool>,
    /// `parameters_json_schema()` parsed once at registration; `None` when it
    /// is not valid JSON. Used by argument coercion (WI-34).
    schema: Option<Arc<Value>>,
}

#[derive(Default)]
struct Registry {
    /// Registration order — drives `HarnessConfig.tools` and `tool_names()`.
    order: Vec<String>,
    by_name: HashMap<String, ToolEntry>,
}

#[derive(Clone, Default)]
pub struct ToolRunner {
    inner: Arc<tokio::sync::RwLock<Registry>>,
    // context slot added by WI-33
}
```

New/changed methods:

| Method | Signature | Upstream |
|---|---|---|
| `register` | `pub async fn register(&self, tool: Arc<dyn DynTool>) -> Result<(), anyhow::Error>` | `tool_runner.py:168-190` |
| `register_as` | `pub async fn register_as(&self, name: String, tool: Arc<dyn DynTool>) -> Result<(), anyhow::Error>` | `:182` (`name` override) |
| `unregister` | `pub async fn unregister(&self, name: &str) -> Result<(), anyhow::Error>` | `:192-204`, KeyError at `:201-202` |
| `tool_names` | `pub async fn tool_names(&self) -> Vec<String>` | `:206-214` |
| `descriptors` | `pub async fn descriptors(&self) -> Vec<crate::types::ToolDescriptor>` | replaces direct map iteration |

Error strings verbatim: `"Tool '{name}' is already registered."` and
`"Tool '{name}' is not registered."`. `unregister` must also
`reg.order.retain(|n| n != name)`.

**New public type in `src/types.rs`** (next to `ToolCall` at `:346-358`):

```rust
/// Static description of a registered client-side tool, as advertised to the harness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    pub parameters_json_schema: String,
}
```

**Update the manual `Debug` impl at `src/tools.rs:117-123`:**
`.field("tools_count", &self.inner.try_read().map_or(0, |r| r.order.len()))`.

**Both `HarnessConfig` build sites become identical** (`src/local.rs:512-523` and
`src/wasm.rs:148-159` — the two bodies are already byte-identical, so this is a
two-site copy; X1's `src/harness_config.rs` later collapses it to one):

```rust
let mut proto_tools = Vec::new();
if let Some(ref runner) = self.tool_runner {
    for d in runner.descriptors().await {
        proto_tools.push(ProtoTool {
            name: Some(d.name),
            description: Some(d.description),
            parameters_json_schema: Some(d.parameters_json_schema),
            response_json_schema: None,
        });
    }
}
```

`src/agent.rs:299` becomes `self.tool_runner.register(tool.clone()).await?;` — the
enclosing closure is `Box::pin(async move { ... }) -> Result<Agent<Started>, anyhow::Error>`
(`src/agent.rs:183-184`), so `?` compiles in place.

**`ToolEntry` is deliberately private:** making it public would put
`Arc<dyn DynTool>` back in the public API surface and re-create the same semver
trap.

**Fallback if the maintainer will not cut 0.2.0 yet:** keep `pub tools` as the
`HashMap` and add a private `order: Arc<RwLock<Vec<String>>>` alongside, drive
`descriptors()` from it, and have `register` return `Result`. That fixes T8
non-breakingly but leaves a public map a user can mutate behind the invariant's
back, and does not fix T7 (inherently a signature change). **Recommend the clean
version in 0.2.0.**

**Call sites.** `src/tools.rs:110-115` (`pub tools` field removed) · `:117-123`
(Debug impl) · `:125-139` (`new`, `register`; add the four new methods) ·
`:148-149` (`process_tool_calls` lookup — see WI-35) · `src/types.rs:346` (add
`ToolDescriptor`) · `src/agent.rs:299` (`.await` → `.await?`) ·
`src/local.rs:512-523` · `src/wasm.rs:148-159` · `docs/tools.md:100`
(`runner.register(...).await;` → `.await?;`) · `docs/tools.md:109`
("Tools are stored behind `Arc<RwLock<Vec<Arc<dyn DynTool>>>>`" — **currently
wrong in both directions**; state the new accessor API instead) ·
`tests/documentation_examples.rs:32` (constructs `ToolRunner::new()`; unaffected,
but recompile-check it).

**Tests.**
- Port 0.1.9 `tool_runner_test.py:67-78` (`test_register_duplicate_raises`):
  register the same tool twice, assert the second call returns `Err` whose message
  is exactly `"Tool 'lookup_fruit_sku' is already registered."`, and assert
  `tool_names().await.len() == 1` (**the first registration survives**).
- Port `:43-53` (`test_register_and_list`) and `:55-65`
  (`test_register_with_custom_name`, exercising `register_as`): assert
  `tool_names().await == vec!["..."]` exactly.
- Port `:80-91` (`test_unregister`) and `:93-103` (`test_unregister_missing_raises`,
  message `"Tool 'nonexistent' is not registered."`); also assert `unregister`
  removes the name from `order` so `descriptors()` no longer lists it.
- **NEW ordering test** (no upstream equivalent because Python dicts make it
  trivially true): register three tools A, B, C in that order, call
  `descriptors().await` **50 times in a loop**, assert every result is `[A, B, C]`.
  Without this the `HashMap` regression is invisible in a single-run test.
- Integration-level: extend `tests/integration_tests.rs` so the mock harness
  captures the `InitializeConversationEvent` JSON and the test asserts
  `harnessConfig.tools[*].name` equals the registration order (needs the mock
  change already required by WI-36).

**Behaviour change.** Two tools registered under the same name — via
`Agent::builder().tools(vec![a, b])` where `a.name() == b.name()`, or via
`register_tool` twice — now make `Agent::start()` return
`Err("Tool '<name>' is already registered.")` instead of silently keeping only the
last one. `HarnessConfig.tools` is emitted in registration order on every run
instead of a per-process random order. `ToolRunner.tools` is no longer a public
field; `ToolRunner::register` returns `Result`. Migration:
`runner.register(t).await;` → `runner.register(t).await?;`, and anything reading
`runner.tools` uses `runner.descriptors().await` or `runner.tool_names().await`.

**Risk.** Compile fan-out is the whole risk and it is bounded and mechanical:
removing `pub tools` breaks exactly the two reader sites found by
`grep -rn '\.tools\.read()\|\.tools\.write()\|tools\.try_read' src/ examples/ tests/ docs/`
(`src/local.rs:514`, `src/wasm.rs:150`, plus the Debug impl at `src/tools.rs:120`),
and `register` returning `Result` breaks exactly one call site
(`src/agent.rs:299`) plus one doc line. **No example or test calls either.**
`ToolRunner` must keep `#[derive(Clone, Default)]` —
`Arc<RwLock<Registry>>` where `Registry: Default` satisfies both — and `ToolEntry`
must derive `Clone` (both fields are `Arc`) so WI-35 can snapshot entries out of
the read guard cheaply. Use `serde_json::from_str(..).ok()` for the schema parse,
never `unwrap`. `nursery` warns on `significant_drop_tightening`, so scope the
write guard with a block in `register_as`.

---

### WI-33 (T1 + T9) — Construct and inject the `ToolContext`, via a `WeakConnection`

**Depends on WI-32.**

**Defect.** `ToolContext` is unreachable code.
`grep -rn 'ToolContext|call_with_context|needs_context|set_context' --include=*.rs .`
returns hits only in `src/tools.rs` (the trait declarations at `:34-50` and the
`DynTool` mirrors at `:69-77`, `:97-107`) and `src/tool_context.rs`. `ToolRunner`
(`src/tools.rs:110-115`) holds only the tool map — no context field, no
`set_context`. `process_tool_calls`, the sole dispatcher (called from
`src/local.rs:1151`, `src/wasm.rs:790`), unconditionally calls
`tool.call(call.args)` at `src/tools.rs:150` and **never consults
`needs_context()`**. `Agent::start` (`src/agent.rs:297-300`) only registers tools.
Upstream wires it at 0.1.9 `agent.py:136-140` and at 0.1.1 `agent.py:176-180`, so
**this is a regression against the port baseline.** Consequence: a tool returning
`true` from `needs_context()` silently has `call_with_context` bypassed and gets
`call` instead; state it wrote via `ctx.set_state` is never visible;
`ctx.conversation_id()` is never reachable.

**(a) `WeakConnection` in `src/connection.rs` — required, not cosmetic.**
`ToolContext::new` takes an `AnyConnection` (`src/tool_context.rs:27,33`), which is
`Arc<LocalConnection>` (`src/connection.rs:72-79`); `LocalConnection` holds a
`ToolRunner` (`src/local.rs:58`) whose context slot would hold that same `Arc`.
**The strong count never reaches zero**, so the connection, its
`Arc<Mutex<tokio::process::Child>>` (`src/local.rs:53`), its `child_stdin`, its
`UnboundedSender` and its step channel leak once per `Agent` for the process
lifetime. Upstream has the identical cycle and Python's cycle collector reclaims
it; `Arc` does not.

```rust
/// A non-owning handle to an [`AnyConnection`].
///
/// [`ToolContext`](crate::tool_context::ToolContext) holds one of these rather
/// than an `AnyConnection`, because the context is stored inside the
/// [`ToolRunner`](crate::tools::ToolRunner) that the connection itself holds a
/// clone of — a strong handle would form an `Arc` cycle and leak the connection
/// together with its child process.
#[derive(Clone)]
pub enum WeakConnection {
    #[cfg(not(target_arch = "wasm32"))]
    Local(std::sync::Weak<crate::local::LocalConnection>),
    #[cfg(target_arch = "wasm32")]
    Wasm(std::sync::Weak<crate::wasm::WasmConnection>),
    #[cfg(test)]
    Mock(std::sync::Weak<MockConnection>),
}

impl std::fmt::Debug for WeakConnection { /* mirror the AnyConnection impl at :81-92 */ }
impl AnyConnection { pub fn downgrade(&self) -> WeakConnection { /* Arc::downgrade per arm */ } }
impl WeakConnection { pub fn upgrade(&self) -> Option<AnyConnection> { /* per arm */ } }
```

**The cfg arms must mirror `AnyConnection`'s exactly** (`src/connection.rs:73-79`):
`Local` on `not(wasm32)`, `Wasm` on `wasm32` **only** (note `src/lib.rs:70`
compiles `mod wasm` under `any(wasm32, test)` but the enum arm is `wasm32`-only —
**do not widen it**), `Mock` on `test`. A manual `Debug` impl is mandatory
(`missing_debug_implementations = "warn"`).

**(b) `src/tool_context.rs`** — field type `AnyConnection` → `WeakConnection`;
`ToolContext::new(connection: AnyConnection)` **signature unchanged** (it
downgrades internally). `conversation_id` returns **owned**:

```rust
// BEFORE: pub fn conversation_id(&self) -> &str
// AFTER:  returns owned, because upgrading yields a temporary Arc.
//         Empty string once the connection has been torn down.
pub fn conversation_id(&self) -> String {
    self.connection.upgrade().map(|c| c.conversation_id().to_string()).unwrap_or_default()
}
```

Owned return is **unavoidable and correct**: the id is *dynamic*.
`LocalConnection::conversation_id` (`src/local.rs:80-86`) falls back to
`learned_id` — an `Arc<OnceLock<String>>` (`:52`) populated only when the first
`StepUpdate` arrives (`:748-760`) — so any design that snapshots the id at
`Agent::start` time yields `""` forever on the default path where the caller
supplies no `conversation_id`. That is precisely the case WI-36's end-to-end test
covers.

**T9, strongly recommended in the same commit:** upstream **deleted** `is_idle`
and `send` from `ToolContext` in 0.1.4 (0.1.3 `tools/tool_context.py:71-84` has
both; 0.1.4 has neither, and the ctor moved from `Connection` to `Conversation`).
Because T1 means nothing can currently reach these methods, **deleting them now
costs nothing; deleting them after T1 ships makes them a real semver break on a
live API.** Do it here. `get_state`/`set_state` (`:57-73`) are unchanged.

**(c) `src/tools.rs` — the shared slot.** It must be `Arc`-wrapped:
`LocalConnectionStrategy::new` receives `Some(self.tool_runner.clone())` at
`src/agent.rs:362` *before* `connect()` runs, `connect()` clones again into
`LocalConnection.tool_runner` (`src/local.rs:1260`) and again into the reader task
(`:710`). `#[derive(Clone)]` copies `Arc` fields, so an `Arc`-wrapped slot set
later is visible to every one of those clones; **a plain
`Option<ToolContext>` field would be visible to none.**

```rust
context: Arc<std::sync::RwLock<Option<Arc<ToolContext>>>>,

/// Sets the context injected into tools that return `true` from
/// [`Tool::needs_context`]. Visible to every existing clone of this runner.
pub fn set_context(&self, ctx: ToolContext) {
    let mut slot = self.context.write().unwrap_or_else(std::sync::PoisonError::into_inner);
    *slot = Some(Arc::new(ctx));
}

fn context(&self) -> Option<Arc<ToolContext>> {
    self.context.read().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
}
```

Use **`std::sync::RwLock`, not tokio's**: it keeps `set_context` synchronous
(matching upstream `tool_runner.py:160-166`) and the guard is never held across an
`await` — `context()` clones the `Arc` out and drops the guard immediately. The
`unwrap_or_else(PoisonError::into_inner)` idiom is already used in this crate
(`src/connection.rs:281`) and satisfies `unwrap_used = "deny"`.

Dispatch, inside the per-call future in `process_tool_calls` (see WI-35):

```rust
let out = if entry.tool.needs_context() {
    match ctx.as_deref() {
        Some(c) => entry.tool.call_with_context(args, c).await,
        None    => entry.tool.call(args).await,
    }
} else {
    entry.tool.call(args).await
};
```

with `let ctx: Option<Arc<ToolContext>> = self.context();` hoisted once before the
batch. The future stays `Send`: `&ToolContext` is `Send` because `ToolContext` is
`Sync` (`std::sync::Mutex<HashMap<..>>` plus `WeakConnection`, which is
`Send + Sync` because `Connection: Send + Sync` at `src/connection.rs:16`).

**(d) `src/agent.rs` — the wiring, both branches.** Insert immediately after the
`Conversation` is built and **before** the trigger runner starts: wasm branch after
`:326` and before `:328-334`; native branch after `:372` and before `:374-381`.

```rust
// Wire the ToolContext so tools that request it can reach conversation state.
self.tool_runner
    .set_context(crate::tool_context::ToolContext::new(conversation.connection()));
```

`Conversation::connection()` (`src/conversation.rs:72-74`) already returns the
`AnyConnection` by value that `ToolContext::new` takes.

**Deliberate ordering divergence — comment it, or a future parity pass will
silently reintroduce the race.** Upstream sets the context *after* starting
triggers (`agent.py:126-140`); we set it **before**, because `TriggerRunner::start`
(`src/agent.rs:332`, `:379`) can fire a `send_trigger_notification` immediately,
which can produce a tool call before `start()` returns.

**Call sites.** `src/connection.rs:79` (add `WeakConnection` + `Debug` impl) ·
`:94` (add `impl AnyConnection { pub fn downgrade }` — a **new inherent impl
block**, not part of the `Connection` trait impl) · `src/tool_context.rs:26-29`,
`:33-38`, `:41-43`, `:45-53` (delete per T9) · `src/tools.rs:110-115`, `:125-131`,
`:150` · `src/agent.rs:326`, `:372` · `docs/tools.md:146-180` (the `ToolContext API`
block documents `conversation_id(&self) -> &str` and `is_idle`/`send`).

**Tests.** Must be **unit tests in the lib crate, not in `tests/`**:
`AnyConnection::Mock` is `#[cfg(test)]`-gated at `src/connection.rs:77-78` and is
therefore invisible to integration-test crates.
- Port 0.1.9 `tool_runner_test.py:682-708` (`test_process_tool_calls_with_context`):
  build `AnyConnection::Mock(Arc::new(MockConnection::new("test-id")))`,
  `runner.set_context(ToolContext::new(conn))`, register a tool whose
  `needs_context()` is `true` and whose `call_with_context` records
  `ctx.conversation_id()` and calls `ctx.set_state`, drive `process_tool_calls`,
  assert the recorded id is `"test-id"` and the state round-trips.
- Port `:586-597` (`test_tool_without_context_works_normally`).
- Port `:642-660` (`test_no_context_set_skips_injection`): falls back to `call`
  and does not panic.
- **NEW leak regression** (no upstream analogue — Python cannot have this bug):
  construct `Arc<MockConnection>`, wrap it in `AnyConnection::Mock`,
  `set_context` on a runner, drop every strong handle the test holds, assert
  `Weak::upgrade().is_none()`. **This is the test that fails if someone changes
  `WeakConnection` back to a strong handle.**
- End-to-end (**the test that would actually have caught T1** — delivered by
  WI-36): start an `Agent` against the mock harness with **no** `conversation_id`
  configured, register a context-aware tool, drive a turn that produces a
  `toolCall`, and assert the tool observed a non-empty `conversation_id` equal to
  the learned trajectory id. **A snapshot-based context design passes the unit
  tests above and fails this one.**

**Behaviour change.** `Tool::needs_context()`/`Tool::call_with_context()` start
working. `ToolContext::conversation_id()` returns `String` instead of `&str`.
`ToolContext::is_idle()` and `ToolContext::send()` are removed. **Nothing that
compiles today breaks in behaviour, because nothing today can obtain a
`ToolContext` at all** — which is exactly why these signature changes are free now
and expensive later.

**Risk.** (1) The `Arc` cycle above is the reason for `WeakConnection`; reviewing
this patch, check that nothing reintroduces a strong `AnyConnection` inside
`ToolContext`. (2) `std::sync::RwLock` guards must never cross an `await` —
`context()` returns a cloned `Arc` precisely so this is structurally impossible;
**do not "optimise" it into returning a guard.** (3) `Tool` is not object-safe
(RPITIT at `src/tools.rs:29-32`), which is why `DynTool` exists; both
`needs_context` and `call_with_context` are **already** on `DynTool` (`:70-77`)
with a blanket impl (`:97-107`), so **no trait changes are needed** and no external
`impl Tool` breaks. The existing blanket impl relies on param-env method
resolution preferring the `T: Tool` where-clause over the `DynTool` blanket impl
(that is why `:82` `self.name()` is not infinite recursion while `:98` explicitly
writes `Tool::needs_context(self)`) — **do not restructure those bodies.**

---

### WI-34 (T4) — Coerce model-supplied arguments against the tool's JSON Schema

**Depends on WI-32** (the pre-parsed `ToolEntry.schema`).

**Defect.** Upstream runs every argument through
`pydantic.TypeAdapter(annotation).validate_python(value)` before calling the tool
(`_coerce_args`, 0.1.9 `tool_runner.py:272-311`, called from `process_tool_calls`
at `:366` and `execute` at `:333`), retaining the original value when validation
fails (`:303-304`). The Rust runner has no equivalent: `src/tools.rs:150` calls
`tool.call(call.args)` with the raw `serde_json::Value`. Models routinely emit
numeric arguments as JSON strings, so `{"count": "5"}` against a schema declaring
`"count": {"type": "integer"}` fails `Value::as_u64` and the tool errors out.
**Not hypothetical:** `examples/custom_tools.rs:88-91` is exactly that code —
`args.get("count").and_then(Value::as_u64).ok_or_else(|| anyhow!("Missing count"))?`.
Introduced in 0.1.8 (`grep -c _coerce_args` is 0 for 0.1.1–0.1.7, 3 for 0.1.8/0.1.9),
so **not a regression — a real usability defect.**

**New private module `src/coerce.rs`**, `mod coerce;` in `src/lib.rs`
(crate-private, adds no public surface):

```rust
/// Coerces `args` against `schema`. Returns `args` unchanged when `schema` is
/// `None`, when `args` is not an object, or when nothing matches.
pub fn coerce_args(schema: Option<&Value>, args: Value) -> Value;
fn coerce_value(schema: &Value, value: Value) -> Value;
fn coerce_to_type(ty: &str, value: Value) -> Option<Value>;
```

**Algorithm.**
`coerce_args`: for each `(k, v)` in the object, if `schema["properties"][k]`
exists, replace `v` with `coerce_value(prop_schema, v)`; else if
`schema["additionalProperties"]` is an object, coerce against that; else leave
untouched. **Keys absent from `properties` are always retained** — upstream
explicitly keeps extra arguments (`tool_runner.py:306-309`).

`coerce_value`:
1. `Value::Null` → unchanged (upstream `:296-298`).
2. `anyOf`/`oneOf` array → for each subschema in declaration order, if
   `coerce_value(sub, value.clone())` differs from `value`, return it; else return
   `value`. Reproduces `test_coerce_args_optional_and_union`
   (`tool_runner_test.py:291-304`): `Optional[int]` with `"10"` → `10`;
   `Union[int, float]` with `"3.14"` → int fails, float succeeds → `3.14`.
3. `schema["type"]` may be a string **or an array of strings** (JSON Schema
   nullable form). Normalise to a slice, skip `"null"`, try each in order via
   `coerce_to_type`, first `Some` wins, else return unchanged.
4. `"object"` → if `value` is an object, recurse via `coerce_args(Some(schema), value)`.
5. `"array"` → if `value` is an array and `schema["items"]` is an object, map
   `coerce_value(items, elem)`. Ignore the tuple form of `items`.
6. Unknown or absent `type` → unchanged.

`coerce_to_type` — returns `None` when not applicable, in which case the caller
keeps the original:
- `"integer"`: `Value::String(s)` where `s.trim().parse::<i64>()` succeeds (also
  accept `u64` above `i64::MAX`); `Value::Number(n)` where `n.is_f64()` with a
  zero fractional part. **Deliberately NOT coercing `Value::Bool`** — a model
  sending `true` for an integer field is a genuine error and preserving it
  surfaces that. This is the one place pydantic's lax mode is more permissive than
  is useful here; **call it out in the module doc comment as intentional.**
- `"number"`: `Value::String(s)` parsing to a finite `f64` → `Number::from_f64(f)`
  (which returns `Option` — **never `unwrap`**).
- `"boolean"`: trimmed, lowercased string in `["true","t","yes","y","on","1"]` →
  `true`; in `["false","f","no","n","off","0"]` → `false`; else `None`.
  `Value::Number` equal to 1 or 0 → `true`/`false`.
- `"string"`: **`None`** — pydantic v2 lax mode does **not** coerce int/float/bool
  to str, and neither do we. **Do not add it.**
- `"null"`: `None`.

**Wiring** in `process_tool_calls`, inside the per-call future (see WI-35):

```rust
// Upstream's ToolCall.args defaults to `{}` (types.py:529); the wire path can
// hand us Null when `arguments_json` is absent.
let raw = if call.args.is_null() { Value::Object(Map::new()) } else { call.args };
let args = crate::coerce::coerce_args(entry.schema.as_deref(), raw);
```

`entry.schema` is the `Option<Arc<Value>>` parsed once at registration by WI-32, so
**no per-call schema parse**. `as_deref()` on `Option<Arc<Value>>` yields
`Option<&Value>`.

**Call sites.** `src/coerce.rs` (new) · `src/lib.rs:73-78` (`mod coerce;` —
crate-private, **not `pub`**) · `src/tools.rs:150` · `src/tools.rs` (register:
`ToolEntry.schema` pre-parse, delivered by WI-32) ·
`examples/custom_tools.rs:88-91` (superseded by WI-36's rewrite).

**Tests.**
- Port 0.1.9 `tool_runner_test.py:275-289` (`test_coerce_args_basic_types`):
  schema `{"type":"object","properties":{"a":{"type":"integer"},"b":{"type":"number"},"c":{"type":"boolean"},"d":{"type":"string"}}}`,
  args `{"a":"5","b":"2.5","c":"true","d":"hello"}` →
  `{"a":5,"b":2.5,"c":true,"d":"hello"}`, asserting the JSON **kinds**
  (`is_i64`, `is_f64`, `is_boolean`, `is_string`), not just equality.
- Port `:291-304` (`test_coerce_args_optional_and_union`), including
  `{"x":null,"y":42}` → unchanged (null-passthrough).
- Port `:306-314` (`test_coerce_args_fallback_on_error`): `{"count":"not_an_int"}`
  → unchanged.
- Port `:422-438` (`test_missing_args_defaults_to_empty`) as a
  `process_tool_calls` test: a zero-argument tool whose `call` asserts
  `args == json!({})`, invoked with `ToolCall{ args: Value::Null, .. }`.
- NEW (no upstream analogue — upstream has no schema path): unparseable
  `parameters_json_schema` → `ToolEntry.schema == None` → args pass through
  untouched; nested `{"type":"object","properties":{...}}` recursion;
  `{"type":"array","items":{"type":"integer"}}` with `["1","2"]` → `[1,2]`; a key
  absent from `properties` survives untouched.
- End-to-end (WI-36): the mock emits
  `argumentsJson: "{\"sku\":\"SKU-1\",\"count\":\"5\"}"` against a tool whose
  schema declares `count` as `integer`; assert the tool received `5` as a number.

**Behaviour change.** Arguments the model encodes as JSON strings now arrive at
`Tool::call` already converted to the JSON type the tool's schema declares. A tool
defensively accepting both keeps working; a tool that *depended* on receiving the
string form would change behaviour, **which is why coercion is strictly
schema-driven and never applied to keys the schema does not declare.**
Zero-argument tools now receive `{}` instead of `null`.

**Risk.** The semantic hazard is being *more* permissive than pydantic, which
turns a model error into a silently wrong argument. Two places to hold the line:
**do not coerce anything → `"string"`, and do not coerce `Bool` → `"integer"`.**
Both should carry a comment citing this decision, because they are the two rules a
future contributor will be tempted to "improve". `Number::from_f64` returns
`Option` and `unwrap_used`/`expect_used`/`panic` are all deny, so every parse must
be `.ok()`/`if let`. `coerce_value` recurses on nested objects and arrays — **bound
it** (a depth counter capped at 32) so a maliciously nested schema cannot blow the
stack. Keep `coerce_args` **total**: it must never return `Err` and never panic,
because it runs on the reader task's spawned future.

**Note:** WI-25 fixes the `Value::Null` at the two *wire* sites for the benefit of
any other consumer of `ToolCall.args`; the normalisation here covers only the path
through `process_tool_calls`. Both are wanted.

---

### WI-35 (T10) — Execute a tool-call batch concurrently

**Depends on WI-32, WI-33, WI-34** (its body is where all three land).

**Defect.** `process_tool_calls` (`src/tools.rs:142-225`) is a `for call in calls`
loop that awaits each tool to completion before starting the next (`:147-150`), and
it re-acquires the registry read lock inside the loop body on every iteration
(`:148`). Upstream executes the batch with `asyncio.gather` —
`return list(await asyncio.gather(*[_execute_one(tc) for tc in tool_calls]))`
(0.1.9 `tool_runner.py:377`; identical at 0.1.1 `:315`) — wrapping each call in its
own try/except so a failure cannot cancel its siblings (`:358-359` states this
verbatim). **Second, subtler defect:** the read guard at `:148` is held across
`tool.call(...).await` at `:150`, so a tool implementation that calls back into
`ToolRunner::register` **deadlocks** against tokio's fair `RwLock`.

**Rewrite the body; the signature is unchanged.**

```rust
/// Executes a batch of tool calls **concurrently** and maps their outputs to
/// [`ToolResult`](crate::types::ToolResult)s, one per input call, in the same
/// order.
///
/// Tools execute concurrently; callers must not depend on sequential
/// side-effect ordering.
pub async fn process_tool_calls(&self, calls: Vec<ToolCall>) -> Vec<ToolResult> {
    // Snapshot the registry and release the lock before executing anything,
    // so a tool that registers another tool cannot deadlock.
    let entries: Vec<Option<ToolEntry>> = {
        let reg = self.inner.read().await;
        calls.iter().map(|c| reg.by_name.get(&c.name).cloned()).collect()
    };
    let ctx = self.context();                       // Option<Arc<ToolContext>>, WI-33
    let ctx_ref = ctx.as_deref();

    let futures = calls.into_iter().zip(entries).map(|(call, entry)| async move {
        let name = call.name.clone();
        let id = call.id;
        let Some(entry) = entry else {
            return ToolResult { id: Some(id), name: name.clone(), result: None,
                                error: Some(format!("Unknown tool: '{name}'")),  // WI-14
                                server_name: None, exception: None };
        };
        let raw = if call.args.is_null() { Value::Object(serde_json::Map::new()) }
                  else { call.args };
        let args = crate::coerce::coerce_args(entry.schema.as_deref(), raw);   // WI-34
        let outcome = if entry.tool.needs_context() {                          // WI-33
            match ctx_ref {
                Some(c) => entry.tool.call_with_context(args, c).await,
                None    => entry.tool.call(args).await,
            }
        } else {
            entry.tool.call(args).await
        };
        match outcome {
            Ok(val) => ToolResult { id: Some(id), name, result: Some(val), error: None,
                                    server_name: None, exception: None },
            Err(e)  => { let arc = std::sync::Arc::new(e); let msg = arc.to_string();
                         ToolResult { id: Some(id), name, result: None, error: Some(msg),
                                      server_name: None, exception: Some(arc) } }
        }
    });

    futures_util::future::join_all(futures).await
}
```

`join_all` preserves input order in its output, which is what upstream's docstring
guarantees (`tool_runner.py:353-354`) and what `src/local.rs:1152`
(`results.into_iter().next()`) relies on. Because each per-call future returns a
`ToolResult` rather than propagating an error, a failing tool cannot affect
siblings — the Rust analogue of upstream's per-task try/except.

`futures_util` is already a direct dependency (used for `BoxFuture` at
`src/tools.rs:8`), and `future::join_all` is a pure combinator — **it spawns
nothing**, so it works identically on the wasm path where `crate::spawn_task` maps
to `any_spawner::Executor::spawn_local`. No new dependency, no cfg split.

Update the method doc comment and the `//! ... concurrent execution` claim in the
module header (`src/tools.rs:5`), which is currently false.

**Call sites.** `src/tools.rs:141-225` (replace the whole body) · `:5` (module
doc) · `docs/tools.md:105` (documents `runner.process_tool_calls(&tool_calls)` —
the signature takes `Vec<ToolCall>` **by value**, not a slice; fix while here,
T12).

**Tests.**
- Port 0.1.9 `tool_runner_test.py:484-518`
  (`test_mixed_batch_failure_does_not_swallow_successes`): a three-call batch
  [good, bad, good] returns three results in order, with
  `results[0].result == Some(10)`, `results[1].error == Some("kaboom")` and
  `results[1].result.is_none()`, `results[2].result == Some(20)`.
- Port `:364-380` (`test_multiple_tool_calls`).
- **NEW concurrency assertion** (upstream's equivalent is implicit in
  `asyncio.gather`): two tools that each `tokio::time::sleep(150ms)` in a single
  batch complete in well under 300 ms wall-clock. Gate generously (assert
  < 250 ms) so it is not flaky on loaded CI.
- **NEW deadlock regression:** a tool whose `call` awaits
  `runner.register(other_tool)` on a cloned `ToolRunner` must complete rather than
  hang. Wrap in `tokio::time::timeout` so a regression fails instead of hanging the
  suite.

**Behaviour change.** A multi-call batch runs its tools concurrently, so side
effects interleave. Results are still returned one per input call in input order,
and a failing tool still cannot affect siblings. The registry read lock is no
longer held during tool execution. **No change at all for the wire path, which
passes batches of one** (`src/local.rs:1151` and `src/wasm.rs:790` both build
`vec![tc]` from one `OutputEvent::ToolCall`) — say so in the changelog rather than
overstating the impact.

**Risk.** Borrow-checker shape is the thing to get right:
`entry.tool.call_with_context(args, c)` returns a `BoxFuture<'a, _>` borrowing both
the local `entry.tool` and `*ctx`; that borrow lives across an `await` inside an
`async move` block, which is legal, **but `ctx` must be a local of
`process_tool_calls` (not a temporary inside the closure) so it outlives the
`join_all`.** Move `call.args` only after cloning `call.name` and moving `call.id`.
`Send`-ness survives: `DynTool::call` returns
`BoxFuture = Pin<Box<dyn Future + Send>>`, and `&ToolContext` is `Send` given
`ToolContext: Sync`, so the whole `join_all` future stays `Send` and the
`tokio::spawn` at `src/local.rs:1073` still compiles. Clippy `nursery` may warn
`significant_drop_tightening` on the snapshot block; the explicit `{ }` scope shown
above is the fix.

---

### WI-36 (X19) — Exercise context-aware tools in an example, a unit test and an end-to-end test

**Depends on WI-33, WI-34, WI-14.** This is the coverage gap that hid T1.

**Defect.** No example and no test anywhere in the repo constructs a
`ToolContext`, returns `true` from `needs_context()`, or calls
`call_with_context`. The two unit tests at `src/tool_context.rs:88-111` test a bare
`Mutex<HashMap>` and never touch `ToolContext` itself — `:86-87` says so in a
comment. Worse, **the flagship example demonstrates the anti-pattern**:
`examples/custom_tools.rs:53-55` hand-rolls
`inventory: Arc<Mutex<HashMap<String, u32>>>` and threads it in through the struct,
doing by hand the exact job `ToolContext` exists for. The mock harness
(`src/bin/mock_localharness.rs:66-168`) never emits a `toolCall` event at all, so
no integration test can reach `process_tool_calls` end-to-end.

**(a) Rewrite `RecordFruitTool` in `examples/custom_tools.rs:53-107`** to use the
context, and make the absence of a context **loud** rather than silent:
`needs_context()` returns `true`; `call` returns
`Err(anyhow!("record_fruit requires a ToolContext"))` — *"Without a context there
is nowhere to keep the inventory, so this fails loudly. If ToolContext injection
ever regresses, this example breaks instead of silently losing state."*;
`call_with_context` reads `ctx.get_state("inventory")`, updates, calls
`ctx.set_state`, and returns a message including `ctx.conversation_id()`. Drop the
`inventory` binding at `:132`, the `.clone()` at `:145-147`, and the now-unused
`use std::sync::{Arc, Mutex}`/`HashMap` imports at `:5-6` (`Arc` is still needed
for `.tools(vec![Arc::new(...)])`). **Add a comment noting that
`get_state`/`set_state` is a read-modify-write and therefore racy under concurrent
tool calls (WI-35 makes that reachable), and that upstream's atomic `update_state`
(0.1.7, finding H13) is the future fix — do not silently ship a racy pattern
without saying so.**

**(b) Unit test in `src/tools.rs`'s `#[cfg(test)] mod tests`** — the port of
`tool_runner_test.py:682-708` described under WI-33. **It must live in the lib
crate** (`AnyConnection::Mock` is `#[cfg(test)]`-gated).

**(c) Teach the mock harness to issue a tool call** —
`src/bin/mock_localharness.rs`, in the prompt branch at `:113-147`. When the
prompt contains `"trigger_tool_call"`, after `step1` (which is what populates
`learned_id`, since it sets `cascadeId == trajectoryId == "test_traj"`, matching
`src/local.rs:750-760`) emit:

```rust
let tool_call = serde_json::json!({
    "toolCall": { "id": "tc_1", "name": "record_fruit",
                  "argumentsJson": "{\"sku\":\"SKU-1\",\"count\":\"5\"}" }
});
```

(field names per `proto/localharness.proto:408-413` — `arguments_json` → protojson
`argumentsJson`; the oneof arm is `tool_call = 12` at `:174`). Then read the
client's next WS text frame, parse it as
`{"toolResponse":{"id":..,"responseJson":..}}` (`proto:415-420`,
`input_event.tool_response = 3` at `:342`), and echo the decoded `responseJson`
back as the `text` of a final `STATE_DONE` step. Then send the idle
`trajectoryStateUpdate` as today (`:150-158`).
**`"count":"5"` as a string is deliberate: that single character is the WI-34
end-to-end assertion.**

**(d) End-to-end tests in `tests/integration_tests.rs`**, modelled on
`test_agent_chat_integration` (`:14-84`) but with **`config.conversation_id = None`**
so the context must resolve the *learned* id:
- register a context-aware tool that records `ctx.conversation_id()` and the
  received `count` into an `Arc<Mutex<_>>` the test owns;
- `config.policies = Some(vec![policy::allow_all()])`, `config.binary_path` from
  `CARGO_BIN_EXE_mock_localharness`;
- `agent.chat("trigger_tool_call").await?`;
- assert `call_with_context` ran, **not** `call` (the `call` body returns an error,
  so a T1 regression shows up as a failed tool rather than a silent fallback);
- assert the captured `conversation_id` is `"test_traj"` — **the assertion a
  snapshot-based context design fails**;
- assert the captured `count` deserialised as `5u64` (WI-34);
- assert the echoed `responseJson` reached the response text.

Add `test_unknown_tool_reports_upstream_message` using the same mock path with an
unregistered tool name, asserting the `responseJson` carries
`Unknown tool: '<name>'` (WI-14).

**Call sites.** `examples/custom_tools.rs:5-6`, `:53-107`, `:132`, `:145-147` ·
`src/tools.rs` (new `#[cfg(test)]` cases) · `src/bin/mock_localharness.rs:113-147`
· `tests/integration_tests.rs` · `docs/tools.md:113-180` (`Context-Aware Tools`
section — align with the rewritten example and the `-> String` signature) ·
`examples/README.md`.

**Risk.** The mock-harness change is the fiddly part: `handle_ws_connection`
(`src/bin/mock_localharness.rs:67-168`) is a strictly sequential script that reads
exactly two frames up front (`:73-84`) and then writes. Adding a read *between*
writes means the mock now blocks waiting for the client's `toolResponse`; **if the
client never sends one — exactly what a T1/WI-14 regression could cause — the mock
hangs and the test hangs with it.** Wrap that read in `tokio::time::timeout` and,
on timeout, fall through to the idle update so the test fails with an assertion
rather than a hang. Second: the client emits its `toolCall` handling from a spawned
task (`src/local.rs:1073`) that races the step stream, so the mock must **read
frames in a loop until one deserialises with a `toolResponse` arm**. Third: keep
`config.conversation_id = None`, otherwise `LocalConnection::conversation_id`
(`src/local.rs:80-86`) short-circuits to the caller-supplied id and the test stops
proving anything. Note that the existing `src/wasm.rs:1536-1591` in-file mock test
(finding X12) is a closed Rust→Rust loop and **will keep passing regardless — do
not treat it as coverage.** `cargo build --examples` must be part of the gate;
CI's `cargo test --all-targets` does build root-level `examples/*.rs`, and
`examples/custom_tools.rs` is a root example, so it is covered.

---

### WI-37 (A10) — Narrow the trigger surface to a one-method `TriggerContext`

**Defect.** `src/triggers.rs:11-32` — `Trigger::run(&self, connection: AnyConnection)`
and the object-safe mirror `DynTrigger::run` hand every registered trigger the full
`Connection` surface (`src/connection.rs:16-68`): `send_halt_request`,
`send_tool_response`, `send_tool_confirmation`, `send_question_response`,
`disconnect`, plus `receive_steps` (which would **steal steps from the real
consumer**). A background trigger can forge a tool result, answer a user question,
halt the turn, or kill the session. The only capability any trigger needs is
`send_trigger_notification` — `src/trigger_helpers.rs:26` is the proof. **Original
port omission:** 0.1.1 already wrapped the connection in `TriggerContext`, whose
only public method is `send()` (`triggers/triggers.py:28-50`), with
`Trigger = Callable[[TriggerContext], Awaitable[None]]` at `:53-54` and
per-trigger construction at `trigger_runner.py:64-67`.

**Change — `src/triggers.rs`:**

```rust
use crate::connection::{AnyConnection, Connection as _};

/// Handle handed to every trigger at startup. Its only capability is pushing a
/// message into the agent's event stream.
/// Mirrors 0.1.1 triggers/triggers.py:28-50.
#[derive(Clone, Debug)]
pub struct TriggerContext { conn: AnyConnection }

impl TriggerContext {
    /// Creates a context bound to a live connection. Public so user code can
    /// unit-test its own triggers (upstream's ctor is public too —
    /// helpers_test.py:33).
    pub const fn new(conn: AnyConnection) -> Self { Self { conn } }

    /// Sends a message to the agent. Upstream: triggers.py:41-50.
    pub async fn send(&self, content: &str) -> Result<(), anyhow::Error> {
        self.conn.send_trigger_notification(content).await
    }
}
```

`#[derive(Clone, Debug)]` is valid: `AnyConnection` derives `Clone`
(`src/connection.rs:71`) and has a manual `Debug` (`:81-92`). `TriggerContext` is
`Send + Sync` because `AnyConnection` is.

Then change both trait signatures (`:17-20`, `:28-31`) and the blanket impl
(`:34-41`) from `connection: AnyConnection` to `ctx: TriggerContext`. **Object
safety is preserved:** `DynTrigger` still takes an owned, sized, `Send` parameter
and returns `BoxFuture`, so `Arc<dyn DynTrigger>` (`src/agent.rs:39`) is
unaffected.

**Deliberately NOT added:** `is_idle`, `receive_steps`, `wait_for_idle`. Upstream
0.1.1's `TriggerContext` has exactly one method; 0.1.5 only re-typed its ctor to a
`TriggerConnection` protocol.

**Call sites.** `src/triggers.rs:11-41` · `src/trigger_helpers.rs:22-29`
(`impl Trigger for PeriodicTrigger` — parameter type; folded into WI-39) ·
`src/triggers.rs` `TriggerRunner::start` body (WI-38) ·
`src/connection.rs:235-241` + `:294-296` (`MockConnection`: add
`pub sent_trigger_notifications: std::sync::Mutex<Vec<String>>` and record into it,
so the context can be asserted in tests) · `docs/triggers.md:20-42` (Trigger trait
block, incl. the `use antigravity_sdk_rust::connection::AnyConnection;` at `:27`
and the signature at `:36`) · `:46-66` (DynTrigger block, signature at `:58`) ·
`:225-260` (the `DiskSpaceMonitor` example, `use` at `:227`, signature at `:249`) ·
`skills/google-antigravity-sdk-rust/examples/getting_started/periodic_trigger.md:9,23-30,48-55,103`
· `ARCHITECTURE.md:58` and `README.md:491`.

**Tests.**
- `trigger_context_send_forwards_to_send_trigger_notification` — build
  `TriggerContext::new(AnyConnection::Mock(mock.clone()))`, call `ctx.send("ping")`,
  assert `mock.sent_trigger_notifications == ["ping"]` **and `mock.sent_prompts` is
  empty** (i.e. it is NOT a user turn). Rust form of upstream's
  `TriggerContext(connection=conn)` construction at `helpers_test.py:29-33`.
- Compile-level narrowing proof: a doc-test or unit test whose trigger body calls
  `ctx.send(...)`. **No trybuild negative test is needed** — the point is that
  `TriggerContext` has exactly one inherent method and does not implement
  `Connection`. Assert that in a doc comment.

**Behaviour change.** Triggers can no longer touch the connection. Migration:
`async fn run(&self, connection: AnyConnection)` →
`async fn run(&self, ctx: TriggerContext)`, and
`connection.send_trigger_notification(msg).await` → `ctx.send(msg).await`. **Any
trigger that called `disconnect`/`send_halt_request`/`send_tool_response` was doing
something the Python SDK never permitted and has no replacement by design.**

**Risk.** Breaking for every out-of-tree `impl Trigger`. `Trigger::run` is RPITIT
(`-> impl Future + Send`) while impls use `async fn run(&self, ctx: TriggerContext)`;
that pairing already works in this crate (`src/trigger_helpers.rs:23`). The blanket
`impl<T: Trigger + ?Sized> DynTrigger for T` **must keep taking `ctx` by value** or
object safety breaks. Watch `clippy::missing_const_for_fn` on
`TriggerContext::new` and the `Connection as _` import being flagged unused if
`send_trigger_notification` is called through a fully-qualified path.

---

### WI-38 (A1) — `TriggerRunner::{stop, is_running}` + double-start guard; `Agent::stop` calls it

**Depends on WI-37.**

**Defect.** `src/triggers.rs:64-74` — `TriggerRunner::start(&self, connection: &AnyConnection)`
calls `crate::spawn_task(...)` per trigger and **discards everything**; the struct
(`:44-47`) holds only `triggers`. There is no `stop()`, no `is_running`, and no
already-started guard (upstream raises
`RuntimeError("TriggerRunner is already started.")` at `trigger_runner.py:61-62`).
`Agent<Started>::stop()` (`src/agent.rs:422-425`) calls only
`conversation.disconnect()` and **never touches `self.state.trigger_runner`**,
which is assigned at `:333`/`:380` and never read again. Result: after
`agent.stop()` the trigger tasks keep looping and keep calling
`send_trigger_notification` on a killed harness (`src/local.rs:313-318` SIGKILLs
the process), producing an unbounded stream of send errors, or on wasm an unbounded
stream of writes to a dead socket. Upstream enters the runner on the
`AsyncExitStack` so teardown cancels every task and awaits it.

**Mechanism.** `crate::spawn_task` (`src/lib.rs:84-96`) returns `()` on both
targets and wasm uses `any_spawner::Executor::spawn_local`, so a
`Vec<JoinHandle>` cannot compile. `tokio-util` (`CancellationToken`) is in neither
`Cargo.toml` nor `Cargo.lock` and is native-only in practice. **Use
`tokio::sync::watch` (stop signal) + `tokio::sync::mpsc` (completion signal)** —
both are under the `sync` feature, which `Cargo.toml` enables for wasm32
(`features = ["sync", "macros", "time", "rt"]`) as well as native. **No new
dependency.**

```rust
pub struct TriggerRunner {
    pub triggers: Vec<Arc<dyn DynTrigger>>,
    /// Stop signal for the current run; `None` when not started.
    stop: Mutex<Option<watch::Sender<bool>>>,
    /// Closes once every spawned task has dropped its sender.
    done: Mutex<Option<mpsc::UnboundedReceiver<()>>>,
    /// Tasks that have not finished yet.
    live: Arc<AtomicUsize>,
}
struct LiveGuard(Arc<AtomicUsize>);
impl Drop for LiveGuard { fn drop(&mut self) { self.0.fetch_sub(1, Ordering::SeqCst); } }
```

| Method | Signature | Upstream |
|---|---|---|
| `start` | `pub async fn start(&self, connection: AnyConnection) -> Result<(), anyhow::Error>` | `trigger_runner.py:48-72`; double-start `Err("TriggerRunner is already started.")` per `:61-62`; per-trigger `TriggerContext` per `:64-67` |
| `stop` | `pub async fn stop(&self)` — idempotent; after `stop()` the runner can be started again | `:74-89` |
| `is_running` | `pub fn is_running(&self) -> bool` — `self.live.load(SeqCst) > 0` | `:110-113` |

Each spawned task holds a `done_tx` clone and a `LiveGuard`, and selects the
trigger future against the watch receiver:

```rust
let run = tr.run(ctx);                                  // BoxFuture: Unpin
let stop = Box::pin(async move { let _ = stop_rx.changed().await; });
match select(run, stop).await {
    Either::Left((Err(e), _)) => tracing::error!("Trigger execution failed: {e:?}"),  // :136-139
    Either::Left((Ok(()), _)) => {}
    Either::Right(((), _))    => tracing::info!("Trigger cancelled."),                // :133-135
}
```

**Why `futures_util::future::select` and not `tokio::select!`:** it needs both
futures `Unpin`, which is satisfied (`DynTrigger::run` returns `BoxFuture`, and the
stop side is `Box::pin`ned), and it avoids depending on tokio's `macros` feature
resolution on wasm. Borrowck: `let run = tr.run(ctx);` borrows `tr`, which lives in
the same async block as an upvar and is never moved afterwards, so the borrow is
legal across the await.

`start` drops the original `done_tx` after the loop so only task-held clones
remain; `stop` takes the `watch::Sender` out of the mutex, sends `true`, drops it,
takes the `done_rx`, and drains it until `recv()` returns `None`.

**`src/agent.rs`:**
- `:332` (wasm) `runner.start(&conversation.connection());` →
  `runner.start(conversation.connection()).await?;`
- `:379` (native) same. (Both currently pass `&temporary`, which the by-value
  signature removes.)
- `:422-425`:
  ```rust
  pub async fn stop(&self) -> Result<(), anyhow::Error> {
      // Upstream tears down in LIFO order: the trigger runner is entered on the
      // exit stack after the conversation (agent.py:129-134), so it stops first.
      if let Some(runner) = self.state.trigger_runner.as_ref() {
          runner.stop().await;
      }
      self.state.conversation.disconnect().await?;
      Ok(())
  }
  ```
  `Agent::stop` keeps `&self`; `TriggerRunner::stop(&self)` uses interior
  mutability precisely so it does not need `&mut`.

**Divergence to document:** the double-start guard keys on "has been started", not
on "has live tasks", so `start()` twice on an **empty** trigger list errors where
upstream's `if self._tasks:` would not. That matches upstream's own docstring
("Raises RuntimeError: If the runner is already started") and is the safer reading.

**Call sites.** `src/triggers.rs:44-75` · `:49-55` (Debug impl — add `is_running`;
it must not lock, so read the atomic) · `src/agent.rs:331-334`, `:378-381`,
`:422-425` · `docs/triggers.md:70-95` (add `stop()`, `is_running()`, note `start`
is now async/fallible) · `ARCHITECTURE.md:58`.

**Tests.** All need a bounded `tokio::time::timeout` so a regression fails rather
than hanging CI.
- Port 0.1.1 `trigger_runner_test.py:35-54` (`test_start_runs_triggers`).
- Port `:56-75` (`test_stop_cancels_all_triggers`): Rust has no `CancelledError`,
  so observe cancellation with a `Drop` guard inside the trigger's future
  incrementing an `Arc<AtomicUsize>`; assert it fires for both triggers after
  `stop()`.
- Port `:77-104` (`test_exception_in_trigger_does_not_crash_others`).
- Port `:106-118` (`test_start_twice_raises`) → second `start()` returns `Err`.
- Port `:120-125` (`test_stop_when_not_started_is_noop`) and `:127-133`
  (`test_empty_triggers_list`) — note the documented divergence for a second start
  on an empty list.
- Port `:135-151` (`test_trigger_receives_context_with_connection`) — folds into
  WI-37's test.
- Port `:153-176` (`test_restart_after_stop`).
- **NEW** (no upstream analogue; upstream gets it from the exit stack):
  `agent_stop_stops_triggers` — build an `Agent` with one trigger via
  `AnyConnection::Mock`, `stop()`, then assert no further
  `sent_trigger_notifications` accumulate after a delay.

**Behaviour change.** `agent.stop()` now cancels background triggers and waits for
them before disconnecting. `TriggerRunner::start` changes from
`fn start(&self, &AnyConnection)` to
`async fn start(&self, AnyConnection) -> Result<()>` and errors on a second call.
Migration: `runner.start(&conn)` → `runner.start(conn).await?`.

**Risk.** (1) `stop()` awaits task completion, so a trigger that never reaches an
await point (a busy `loop {}`) hangs `Agent::stop` forever — upstream's
`await asyncio.gather(...)` has the identical hazard; document it. (2) On wasm,
`stop()` only completes if the `any_spawner` executor is being driven while
`stop()` is awaited. (3) Adding private fields to `TriggerRunner`, which has a
public field (`pub triggers`), breaks any out-of-tree struct-literal construction
`TriggerRunner { triggers }`; `TriggerRunner::new` is unaffected. (4) `start()`
becomes `async` + `Result`, so a missed `?` turns a double-start into a silently
ignored `Result` (`unused_must_use` catches it). (5) **`Agent` still has no
`Drop`-based teardown** — a user who never calls `stop()` still leaks the tasks;
note it in docs rather than attempting async `Drop`.

---

### WI-39 (A11) — `every()` must invoke a callback and reject a non-positive interval

**Depends on WI-37.**

**Defect.** `src/trigger_helpers.rs:16-45` —
`PeriodicTrigger { interval, message: String }` and
`every(interval, message: impl Into<String>)`; the loop at `:23-28` sleeps and then
unconditionally calls `connection.send_trigger_notification(&self.message)` with a
hard-coded string. Upstream's `every(interval_seconds, callback)`
(`helpers.py:39-72`) takes an async callback that receives the `TriggerContext` and
decides what (if anything) to send — its docstring is explicit: *"Async function
called each interval. Receives the TriggerContext — use ctx.send() inside the
callback to push messages to the agent."* The Rust version can therefore express
only "ping the agent with a constant", not "poll something and notify only when it
changed", which is the entire point. Separately, `every(Duration::ZERO, ..)` is
accepted and produces a hot loop; upstream raises `ValueError` at `:60-63`.

**Rewrite `src/trigger_helpers.rs:11-45`.** `panic = "deny"` rules out upstream's
raise-on-construct, so validation returns `Result`:

```rust
type TriggerCallback =
    Box<dyn Fn(TriggerContext) -> BoxFuture<'static, Result<(), anyhow::Error>>
        + Send + Sync>;

/// A trigger that invokes a callback at regular intervals. Created via [`every`].
pub struct PeriodicTrigger { interval: Duration, callback: TriggerCallback }

impl std::fmt::Debug for PeriodicTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PeriodicTrigger").field("interval", &self.interval)
            .finish_non_exhaustive()
    }
}

impl Trigger for PeriodicTrigger {
    async fn run(&self, ctx: TriggerContext) -> Result<(), anyhow::Error> {
        loop {
            tokio::time::sleep(self.interval).await;   // first fire AFTER one interval
            (self.callback)(ctx.clone()).await?;       // an Err ends the trigger; the
        }                                              // runner logs it (WI-38)
    }
}

/// Creates a trigger that runs `callback` every `interval`.
/// The first invocation happens after the first interval elapses, not immediately.
/// Mirrors 0.1.1 triggers/helpers.py:39-72.
///
/// # Errors
/// Returns an error if `interval` is zero (helpers.py:60-63).
pub fn every<F, Fut>(interval: Duration, callback: F)
    -> Result<PeriodicTrigger, anyhow::Error>
where
    F: Fn(TriggerContext) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), anyhow::Error>> + Send + 'static,
{
    if interval.is_zero() {
        return Err(anyhow::anyhow!("interval must be positive, got {interval:?}"));
    }
    Ok(PeriodicTrigger { interval, callback: Box::new(move |ctx| Box::pin(callback(ctx))) })
}

/// Convenience wrapper: sends a fixed notification on each tick.
/// Rust-only ergonomic shim over [`every`].
pub fn every_message(interval: Duration, message: impl Into<String>)
    -> Result<PeriodicTrigger, anyhow::Error>
{
    let message = message.into();
    every(interval, move |ctx: TriggerContext| {
        let message = message.clone();
        async move { ctx.send(&message).await }
    })
}
```

**Type-check notes.** The boxed callback is `Fn` (not `FnOnce`), so it can fire
every tick; `every_message`'s closure **clones the `String` per call to stay `Fn`**;
the returned async block is `Send + 'static`, satisfying the `Fut` bound;
`PeriodicTrigger` stays `Send + Sync` because the box is, which the
`Trigger: Send + Sync` supertrait requires. `interval.is_zero()` is the only
reachable check — `Duration` cannot be negative, so **upstream's `< 0` arm has no
Rust analogue**; say so in a comment.

**Decide explicitly whether to keep `every_message`.** For: `docs/triggers.md:107-114,196-206,275-285,301-308,319-327`
and `README.md:317-321` all use the message form, and it is a two-line wrapper with
no hidden behaviour (unlike WI-14's invented web search). Against: it is not
upstream API. **Recommendation: keep it, clearly marked as a Rust-only
convenience.**

**Call sites.** `src/trigger_helpers.rs:11-45` (whole rewrite) · `:33-39` (the
`no_run` doc example — becomes
`every(Duration::from_secs(30), |ctx| async move { ctx.send("check_status").await })?`
or `every_message(...)?`) · `:47-66` (existing unit tests read the now-private
`interval`/`message` fields — replace them) · `README.md:317-321` ·
`docs/triggers.md:101-119` (`every()` section and the pseudo-code loop at
`:117-124`) · `:126-145` (`PeriodicTrigger` Internals — the struct listing is now
wrong) · `:192-206`, `:275-285`, `:301-308`, `:319-327` ·
`skills/google-antigravity-sdk-rust/examples/getting_started/periodic_trigger.md`
(whole file).

**Tests.**
- Port 0.1.1 `helpers_test.py:35-51` (`test_every_calls_callback_on_interval`):
  `every(Duration::from_millis(10), cb)` where `cb` increments an
  `Arc<AtomicUsize>`; drive via `TriggerRunner::start` (or poll `Trigger::run` in a
  task), sleep 50 ms, `stop()`, assert count >= 2.
- Port `:53-61` (`test_every_rejects_non_positive_interval`):
  `assert!(every(Duration::ZERO, cb).is_err())`. Upstream's `-1` case is
  unrepresentable in `Duration`; say so in a comment.
- **NEW** `every_callback_receives_a_context_it_can_send_on` — the callback calls
  `ctx.send("tick")` and the test asserts
  `MockConnection::sent_trigger_notifications` grows. **This is the behaviour A11
  exists to restore.**
- **NEW** `every_stops_when_the_callback_errors` — the callback returns `Err` on
  the second tick; assert the trigger future resolves to that `Err` (the runner
  then logs it, per WI-38's port of `trigger_runner_test.py:77-104`).
- Upstream's `test_every_sets_name` (`helpers_test.py:63-68`) has no Rust analogue
  (no `__name__`); **skip it rather than inventing a `name()` method.**

**Behaviour change.** `every()` now takes a callback and returns
`Result<PeriodicTrigger, _>`; `PeriodicTrigger`'s fields become private. Migration:
`every(d, "msg")` → `every_message(d, "msg")?`, or
`every(d, |ctx| async move { ctx.send("msg").await })?`. A zero interval is now
rejected at construction instead of hot-looping.

**Risk.** Breaking for anyone calling `every(dur, "msg")`, and now fallible, so
every call site needs `?` (`expect_used = "deny"`, so examples must use `?`).
Compile hazards: the **`Fn` (not `FnOnce`) bound** is what allows repeated
invocation — a closure that moves a captured `String` without cloning will fail to
satisfy it; and `#[derive(Debug)]` must be replaced by the manual impl or
`missing_debug_implementations` plus the boxed `dyn Fn` will fight you. Runtime
hazard (**pre-existing, not introduced**): `tokio::time::sleep` inside a trigger
spawned through `any_spawner::Executor::spawn_local` on wasm has no tokio timer —
the shipped code at `src/trigger_helpers.rs:25` already has this, so it is
unchanged, but **do not add new tokio-timer uses on the wasm path** without
addressing it.

---

### WI-40 — Map `StepUpdate.finish` to a `FINISH` `ToolCall`

**Depends on WI-2 (the extractor refactor), WI-5 (`read_only()` gains FINISH),
WI-9 (`safe_defaults`).**

**Defect.** `src/local.rs:1294-1408` and its mirror `src/wasm.rs:1165-1275` extract
eight built-in actions and return `None` for anything else. Upstream's
`_BUILTIN_TOOL_PROTO_FIELDS` (0.1.1 `local_connection.py:105-116`) has a tenth
entry, `FINISH: "finish"`. `optional ActionFinish finish = 31;` exists in this
repo's schema (`proto/localharness.proto:224`) and is already read for step
classification (`src/wasm.rs:427-428`, `:488`). Because the extractor misses it, a
FINISH tool-confirmation request falls into the `None` arm at `src/local.rs:1006`
and is **auto-approved as if it were upstream's `pre_request_host_tool_request`** —
no policy is consulted, and no `post_tool_call` hook is dispatched for it.

**Change.** After WI-2's refactor, add one arm to both extractors, positioned after
`generate_image` and before the final `return None`:

```rust
} else if step_update.finish.is_some() {
    ("FINISH", serde_json::json!({
        "output_string": step_update.finish.as_ref().and_then(|f| f.output_string.clone()),
    }))
}
```

`ActionFinish` has exactly one field (`proto/localharness.proto:237-239`:
`optional string output_string = 1`). No wire-path key is present, so
`canonical_path_from_args` yields `None` — correct.

**Sequence this AFTER WI-5 and WI-9.** Landed on its own it makes FINISH
policy-evaluated while `read_only()` still excludes it, and `deny_all()`-based
policy sets — the shape `tests/documentation_examples.rs:95-96` and
`examples/custom_tools.rs:149-152` teach — would start denying the finish tool,
**which can prevent a turn from ever completing.**

**Call sites.** `src/local.rs:1400-1408` (insert before the trailing `None`) ·
`src/wasm.rs:1265-1275` (mirror) · `docs/hooks.md:305,357,375` and
`examples/custom_tools.rs:149-152` / `examples/structured_output.rs:95-96`
(deny_all-based examples now need an explicit `policy::allow("FINISH")`).

**Tests.**
- `src/local.rs::tests::test_extract_builtin_tool_call_finish` — a `StepUpdate`
  carrying `finish: Some(ActionFinish{output_string: Some("done")})` extracts to
  `ToolCall{name: "FINISH", ..}`. Mirror in `src/wasm.rs`.
- `tests/integration_tests.rs::test_deny_all_plus_finish_completes` — a
  `deny_all() + allow("FINISH")` agent completes a turn; the same agent without the
  FINISH allow does not. **Documents the trap.**
- Upstream pin: 0.1.1 `policy_test.py:666-675` (`test_allows_other_tools_by_default`)
  iterates every `types.BuiltinTools` member **including `FINISH`** through the
  enforcer, which only makes sense because FINISH really is policy-evaluated
  upstream.

**Behaviour change — a DENY change.** The finish tool is now evaluated against the
policy set instead of being auto-approved. Agents whose policies are `deny_all()`
plus a specific allow-list will deny FINISH unless `policy::allow("FINISH")` is
added. Migration: add `policy::allow("FINISH")`, or use
`AgentBuilder::read_only()`, which covers it after WI-5.

**Risk. This is the one item in the plan that can hang a turn.** Under a
deny-by-default policy set that does not explicitly allow FINISH, the finish tool
is rejected and the agent may never terminate the turn — a worse failure mode than
a denied file read. Mitigate by landing strictly after WI-5, calling it out in the
CHANGELOG, and updating the two deny_all-based examples in the same commit. **If
that sequencing is not acceptable, an alternative is to special-case FINISH as
always-approved in the extractor — but that diverges from upstream and should be
avoided.**

---

### WI-41 — Record the two declines in code

Two comment-only edits so the next reader does not re-file them.

**H8** (`src/local.rs:1159-1169`, `src/wasm.rs:798-808`) — add a comment citing
0.1.1 `local_connection.py:1207-1227` and noting the identical structure persists
at 0.1.2 `:1237/:1253`, 0.1.3 `:1265/:1281`, 0.1.4 `:1338/:1354`, 0.1.5
`:1363/:1379`, and that 0.1.6+ has no such branch at all (the only dispatch sites
are `hook_router.py:252` and `:270/:275`). Name the one genuine 0.1.1 divergence at
this site so it is not confused with H8: upstream at `:1216` requires **both**
`recovery_res.allow` **and** `recovery_val is not None` before substituting, where
`src/local.rs:1161-1165` substitutes on `res.allow` alone and can install
`result.result = None` while clearing `result.error`. That is H4 (WI-27).

**H7-narrow** (`src/local.rs:998-1030`, `:899-925`; `src/wasm.rs:638-671`,
`:538-564`) — add a comment stating that the built-in `post_tool_call`/
`on_tool_error` gate on a prior `tool_confirmation_request` is **structurally
forced**: upstream 0.1.1 writes `_pending_builtin_tool_calls` in exactly one place
(`local_connection.py:1106-1121`, guarded by
`if allow and tc.name != DEFAULT_HOST_TOOL_NAME and self._hook_runner`) and reads
it in exactly one place (`:827-859`), and there is no second channel to gate on
before WP-8 introduces `CallHookRequest`. **Any heuristic stopgap — e.g. gating on
`StepUpdate` action fields — would have to synthesise information the 0.1.1 wire
does not carry, and would be undone.** Also update `docs/upstream-parity.md:162`
(H8 row) and `:379-381` (§4.3 H7 entry) to record that the narrow half is
non-divergent against the **pinned** 0.1.1 harness, not merely against a 0.1.9
harness taking the legacy path.

Two incidental observations worth a comment while in the file, neither a defect:
`src/local.rs:901` gates on `state_val == 2 || state_val == 4 || state_val == 5`
while `src/wasm.rs:540` uses `2 || 4` — the `5` arm is unreachable in `local.rs`
because the TERMINAL_ERROR check at `:890-897` `break`s first; and
`src/local.rs:1014-1017` inserts into `pending_builtin_tool_calls` without a
`hook_runner` check (upstream requires one at `local_connection.py:1108`), with the
entry still reclaimed because `:903`'s tuple pattern evaluates the `remove`
unconditionally.

**No new tests** beyond those already assigned: WI-21 ports 0.1.1
`local_connection_test.py:2815-2860` and `:4034-4076`, which defend the H7-narrow
decline; and WI-27's `test_finish_tool_result_dispatches_post_tool_call_on_failure`
is where the H8 contract flips deliberately.
