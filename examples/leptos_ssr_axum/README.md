# Antigravity SDK Chat Interface (Leptos + Spin WASI)

This directory contains the `leptos_ssr_axum` project, which implements a complete, modern web-based chat interface for the Antigravity Agent. It is built with the [Leptos](https://leptos.dev) framework (v0.8) and compiled to run as a WebAssembly guest component on the [Spin](https://github.com/spinframework/spin) WASI runtime.

## Architecture & Integration Pattern

Because standard WASI runtimes (like Spin) restrict components from opening raw TCP or WebSocket connections directly (supporting only `wasi:http` outbound requests), the guest cannot connect directly to the host-side `localharness` server. 

To bridge this limitation, this application utilizes **Sidecar Mode**:
1. **Chat UI (WASI)**: Runs in the Spin WASI guest sandbox. It handles Server-Side Rendering (SSR), serves the client hydrate WASM, and makes outbound HTTP requests to the sidecar server.
2. **Agent Server (Native Sidecar)**: Runs as a native host-side companion process (`agent_server`), wrapping the full Antigravity Rust SDK and managing WebSockets with `localharness`.

```
Browser ──[HTTP/SSE]──→ Spin Component (WASI Guest)
                              │
                              └─ HTTP/SSE Requests ──→ agent_server (Native Sidecar)
                                                           ├─ Agent::chat()
                                                           ├─ WebSocket → localharness
                                                           └─ Gemini API
```

---

## Prerequisites

Ensure you have the following installed on your system:
- **Rust** (with `wasm32-wasip2` and `wasm32-unknown-unknown` targets added).
- **Spin CLI**: Install Spin by following the [Spin Installation Guide](https://developer.fermyon.com/spin/v2/install).
- **cargo-leptos**: Install cargo-leptos using cargo:
  ```sh
  cargo install --locked cargo-leptos
  ```
- **Localharness**: Ensure `localharness` is installed on your host.

---

## Configuration

The application is configured via Spin variables defined in `spin.toml`.

### Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `agent_server_url` | `http://127.0.0.1:8080` | URL of the running `agent_server` sidecar. |
| `gemini_api_key` | `""` | Optional Gemini API key if communicating directly with Gemini APIs from WASI. |

### Outbound Hosts

To make outbound HTTP requests to the sidecar, the component must list the sidecar host in `allowed_outbound_hosts` inside `spin.toml`:
```toml
allowed_outbound_hosts = ["https://generativelanguage.googleapis.com", "http://127.0.0.1:8080"]
```

---

## Building and Running

### 1. Start the Sidecar Server
First, make sure the `agent_server` sidecar is running in a separate terminal:
```sh
cd examples/agent_server
export GEMINI_API_KEY="your-api-key"
cargo run
```

### 2. Build and Start Spin
Now, build the Leptos assets and run the Spin application:
```sh
cd examples/leptos_ssr_axum
spin build --up
```

This will:
1. Run `cargo leptos build --release` to compile client-side hydrate WASM assets and output-style CSS.
2. Compile the server-side logic to the `wasm32-wasip2` target.
3. Start the Spin local web server.

Open `http://localhost:3000` (or the port outputted by Spin) in your web browser.

---

## Key UI/UX Features

- **Interactive Question Hooks**: Dynamically displays multi-choice options or text input forms within the chat timeline when the agent raises questions.
- **Confirmation Intercepts**: Interactive approve/deny dialogs pop up when the agent attempts to run commands, edit files, or execute tools controlled by safety policies.
- **Real-Time Step Logs**: Watch the agent execute its plan step-by-step with collapsible micro-logs showing tool execution details, standard output, and tool statuses.
- **Subagent Nesting & Trajectory Tracking**: Supports nested subagent executions. The UI tracks trajectory IDs and renders subagent execution timelines nested inline within the parent chat timeline.
- **Client-Side Tool Step logs**: Displays live progress and results/errors for custom client-side tools using synthetic step logging.
- **Glassmorphic Theme**: A modern dark-mode design with sleek transitions, subtle hover micro-animations, and a responsive layout.
