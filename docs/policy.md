# Policy

Policies decide whether a tool call runs. They are evaluated by a
`PolicyEnforcer`, which the agent registers as a hook, so every gated tool call
passes through them before execution.

`docs/agent.md` has linked this page for some time without it existing.

## Tool identifiers are SCREAMING_SNAKE

This is the single most common mistake, because it fails **silently**: a policy
whose tool name does not match anything simply never applies, and a trailing
wildcard then decides the call.

```rust
policy::deny("RUN_COMMAND")   // ✅ matches
policy::deny("run_command")   // ❌ matches nothing — the wildcard decides
```

The accepted spellings are exactly `BuiltinTools::as_str()`:

| Variant | Identifier |
|---|---|
| `CreateFile` | `CREATE_FILE` |
| `EditFile` | `EDIT_FILE` |
| `FindFile` | `FIND_FILE` |
| `ListDir` | `LIST_DIR` |
| `RunCommand` | `RUN_COMMAND` |
| `SearchDir` | `SEARCH_DIR` |
| `ViewFile` | `VIEW_FILE` |
| `StartSubagent` | `START_SUBAGENT` |
| `GenerateImage` | `GENERATE_IMAGE` |
| `AskQuestion` | `ASK_QUESTION` |
| `Finish` | `FINISH` |

Prefer `BuiltinTools::RunCommand.as_str()` over a string literal so a rename is
a compile error rather than a policy that stops matching.

> **Divergence from the Python SDK.** Upstream's identifiers are lowercase
> (`run_command`, `list_directory`, `search_directory`). A Python policy ported
> verbatim will not match here. This is a **standing divergence**, not a pending
> change: the 0.1.9 audit recorded it as S8 and 0.2.0 shipped without aligning
> it, so the uppercase spellings are the ones to write against.

## Decisions

| Decision | Effect |
|---|---|
| `Approve` | The call runs. |
| `Deny` | The call is blocked and the model is told why. |
| `AskUser` | The `ask_user` handler decides. A policy without a handler is a config error. |

## Precedence

Policies are bucketed, and **the first match in the highest-priority bucket
wins** — not the last policy in the list. Specificity beats order:

1. A specific tool with a `when` predicate
2. A specific tool without one
3. An MCP `server/tool` target
4. An MCP `server/*` wildcard
5. The global `*` wildcard

So `[allow_all(), deny("RUN_COMMAND")]` denies `RUN_COMMAND` even though the
wildcard comes first. This is why `workspace_only()`'s DENY rules survive being
prepended to a policy set containing `allow_all()`.

## Builders

| Builder | Produces |
|---|---|
| `allow(tool)` / `deny(tool)` | One policy for one tool |
| `ask_user(tool, handler)` | One policy that prompts |
| `allow_all()` / `deny_all()` | The `*` fallback |
| `confirm_run_command(handler)` | Ask (or deny) on `RUN_COMMAND`, allow the rest |
| `safe_defaults(handler)` | Approve read-only tools, ask for everything else |
| `workspace_only(dirs)` | Deny path-carrying tools outside `dirs` |
| `workspace_only_for(tools, dirs)` | The same, for an explicit tool list |
| `allow_mcp` / `deny_mcp` / `ask_user_mcp` | A group for an MCP server |

Group builders return `Vec<Policy>`; compose them with
`AgentBuilder::policy_groups`:

```rust
let agent = Agent::builder()
    .policy_groups([
        policy::workspace_only(vec!["/srv/app".to_string()]),
        vec![policy::deny("RUN_COMMAND"), policy::allow_all()],
    ])
    .build();
```

To attach a predicate or a name to a whole group, map over it:

```rust
let policies: Vec<Policy> = policy::deny_mcp(&server, None)
    .into_iter()
    .map(|p| p.with_name("no_mcp_writes"))
    .collect();
```

## Workspace scoping

`workspace_only(dirs)` denies a tool whose `canonical_path` resolves outside
every directory in `dirs`. **Applied unconditionally** — including when the
policy set contains `allow_all()`, which upstream documents as the way to get
autonomous shell access *while* file tools stay scoped. The opt-out is
`workspaces(vec![])`, not a policy.

Containment is decided after resolution: `.` and `..` are collapsed and symlinks
are followed before comparison, so `<workspace>/../../etc/passwd` is outside.
Resolution failures are treated as outside — it fails closed. A tool call
carrying no path is unaffected.

The agent's `app_data_dir` joins the allow-list so the agent can reach its own
state directory.

> **Divergence, deliberately.** This scopes six tools where upstream scopes
> three (`VIEW_FILE`, `CREATE_FILE`, `EDIT_FILE`). `list_directory` enumerates
> the filesystem and `search_directory` returns matching file *content*, so
> leaving them unscoped would let the model read anywhere on disk. For upstream's
> exact scope: `workspace_only_for(&BuiltinTools::file_tools(), dirs)`.
> `FIND_FILE` is unscoped, matching upstream.

## Predicates

`when` narrows a policy to calls matching a condition:

```rust
policy::deny("RUN_COMMAND").when(|tc| {
    tc.args.get("command_line")
        .and_then(|v| v.as_str())
        .is_some_and(|cmd| cmd.contains("rm -rf"))
})
```

A predicate that panics is treated as **matching** — a policy that cannot decide
must not silently allow the call.

## MCP targets

MCP tools are addressed as `server/tool`, with `server/*` for a whole server.
Registering MCP policies without registering the servers they name is a
config error rather than a silent no-op.

## Validation at startup

`Agent::start` rejects:

- an `AskUser` policy with no handler
- MCP policies with no registered MCP servers
- write tools enabled with no policies at all

## See also

- `docs/agent.md` — agent configuration
- `docs/hooks.md` — the hook system policies are built on
- `docs/upstream-parity.md` — the divergences noted above, with upstream
  references (historical; frozen at 0.2.0)
