# Implementation plan — Phase A (finish the connection)

Written to be executed without re-deriving anything. Batch definitions and the
rest of the roadmap live in `docs/remaining-work.md` §6; this document carries
the code-level context for the five batches on the critical path.

**Why Phase A is the milestone:** after it, the SDK completes a real turn
against a 0.1.9 harness. Everything before it fixed what we *send* (WP-1) and
the handshake (WP-6 core); what remains is the turn actually *terminating*.

**Standing rules for every batch here**

- Verify with CI's toolchain, not the container default:
  `cargo +1.97.1 clippy --all-targets --all-features -- -D warnings`,
  `cargo +1.97.1 fmt --all -- --check`, `cargo +1.97.1 test --all-targets --all-features`.
  The default is 1.94 and misses lints CI enforces.
- **Every change lands in both `src/local.rs` and `src/wasm.rs`.** They are
  forks of each other. CI compiles neither the wasm target nor the docs, so the
  wasm half is unverified until B7 lands — read it, do not assume.
- Upstream sources are extracted per release under the session scratchpad
  (`ag/0.1.1` … `ag/0.1.9`); `scripts/probe_harness.py` drives a real harness.
- The crate forbids `unsafe_code` and denies `unwrap_used` / `expect_used` /
  `panic` outside `#[cfg(test)]` blocks that opt out.

---

## A2 — Sentinel restructure, and C2 with it

**Why paired:** C2 was attempted alone this session and reverted. Flipping the
initial `is_idle` to `true` makes `receive_steps()` return `None` on its first
poll, because that path treats "idle and queue empty" as end-of-stream before
any step arrives. The two only work together.

### Current shape

- Channel: `mpsc::UnboundedSender<Result<Step, anyhow::Error>>`
  (`src/local.rs:64`, receiver stored as
  `Arc<Mutex<Option<UnboundedReceiver<..>>>>`).
- Idle is signalled by a **magic step id**: a `Step { id: "IDLE_SENTINEL", .. }`
  pushed at `src/local.rs:1170` / `src/wasm.rs:707`, matched at
  `src/local.rs:140,159` and `src/wasm.rs:1000,1019`.
- `receive_steps` (`src/local.rs:114`, `src/wasm.rs:974`) **returns** on the
  first sentinel seen while idle, so anything queued behind it is dropped.
- The idle arm uses `conn_is_idle.swap(true, ..)` and only emits a sentinel on
  the *first* idle — every later idle is silent.

### Target shape

Upstream's loop (`local_connection.py:338-360`) is the specification:

```python
while True:
  if self.is_idle and self._processor.step_queue.empty():
    return
  step_obj = await self._processor.step_queue.get()
  if step_obj is IDLE_SENTINEL:
    continue          # <-- re-evaluate the head condition, do not return
  if step_obj is None:
    return
  yield step_obj
```

1. Replace the magic id with an enum carried on the channel:

   ```rust
   pub(crate) enum StepEvent {
       Step(Box<Step>),          // Step is large; box it to keep the enum small
       Error(anyhow::Error),
       Idle,
       Close,
   }
   ```

   Channel becomes `UnboundedSender<StepEvent>`. A real step can then never
   collide with the sentinel — the current design breaks if a harness ever emits
   a step whose id is literally `IDLE_SENTINEL`.

2. Rewrite `receive_steps` as upstream's loop: on `Idle`, **continue** and
   re-check `is_idle && rx.is_empty()`; terminate only when both hold. Drop the
   `checked_initial_idle` special case — the head condition subsumes it.

3. Idle arm: `store(true, ..)` instead of `swap`, and emit `StepEvent::Idle`
   **every** time (`event_processor.py:552-568`).

4. Only then flip the initial `is_idle` to `true` (both sites carry a `NOTE`
   explaining why it is currently `false`).

### Verify

- Port upstream's regression `local_connection_test.py:295-341`: queue
  `[Idle, Step, Idle]` and assert the step after the first idle is yielded.
  Today it is dropped — this is the test that proves the batch.
- `test_wasm_connection_integration_mock` must still pass; it is what caught the
  C2 attempt.
- A turn with two idles emits two sentinels.

---

## A3 — Cancellation

`STATE_CANCELLED = 3` exists in the regenerated proto and nothing handles it.

- Add the arm alongside FULLY_IDLE: push
  `AntigravityExecutionError { message: tsu.error.unwrap_or("Turn cancelled") }`,
  set idle, emit `StepEvent::Idle` — `event_processor.py:560-568` verbatim.
- Add `Connection::cancel()` and a `client_cancelled: AtomicBool`. Upstream
  raises `AntigravityCancelledError` at each termination point when the flag is
  set (`local_connection.py:340-344,352-355`), so a cancelled turn is
  distinguishable from a completed one.
- New error variant in `src/error.rs`. `AntigravityCancelledError` is 0.1.2
  upstream; the crate has no equivalent.

**Depends on A2** — it emits the same sentinel A2 restructures.

---

## A4 — Turn-level errors

- `TrajectoryStateUpdate.error` (field 4) is in the schema and unread. Push it
  as an error before the idle transition (`event_processor.py:554-557`), so a
  turn failing server-side surfaces instead of ending silently.
- `src/local.rs` reads only `step_update.error_message`; fall back to
  `step_update.error.error_message` so a step carrying `http_code=403` with an
  empty top-level message is not reported as empty (audit C14).
- Retain a bounded stderr tail (`VecDeque`, cap ~100) from the reader at
  `src/local.rs:719-730`, currently logged and discarded, and attach it to the
  error on an unexpected close (audit C7). Native only — the wasm transport has
  no subprocess.

---

## A5 — WP-6 remainder

- **Seed `Conversation` from the replayed history.** `connect()` already parses
  it and exposes `LocalConnection::initial_history()`; `Conversation::new`
  cannot accept it. Thread it through, replaying compaction indices and
  cumulative usage.
- **`env` passthrough** — `AgentConfig.env` → `InputConfig.env` (field 5, on the
  *binary* handshake, so map encoding is exercised) **and** `Command::envs()`
  merged over the existing SHELL/PATH base.
- **Prompt sanitization** — strip control characters, upstream
  `_sanitize_prompt` (`local_connection.py:219-229`). Applied to text parts
  only, never the plain-string path.
- **`save_dir`** — default to a `antigravity_*` temp dir when unset.
- **127.0.0.1 fallback** on connect, with the harness stderr in the error
  message (`local_connection.py:1086-1100`).
- **`DebugConfig`** — `enable_server_side_tracing` + logging level (0.1.9).

---

## After Phase A

Re-run `scripts/probe_harness.py` against the 0.1.9 wheel and then drive a real
turn end to end. That is the point to cut `0.1.15-rc` and to update the "still
not connectable" wording in `docs/upstream-parity.md` §2 and the PR body.

Phases B–E are specified in `docs/remaining-work.md` §6 with per-batch
"done when" criteria. Two ordering rules there are load-bearing, not advisory:
**C2 cannot ship without A2** (proven, reverted), and **E5 must be last in
Phase E** — emitting `enabled_hooks` before the router exists converts a silent
no-op into a mid-turn deadlock.
