# MCP

Model Context Protocol integration for the Antigravity Rust SDK.

## Overview

MCP (Model Context Protocol) allows the agent to connect to external tool servers. The SDK configures MCP servers which are managed by the underlying localharness runtime — the SDK passes configuration, and the harness handles client sessions, tool discovery, and execution.

## McpServerConfig

Three transport variants are supported:

### Stdio

Launch a local MCP server as a subprocess:

```rust,no_run
use antigravity_sdk_rust::types::McpServerConfig;

let server = McpServerConfig::Stdio {
    name: "my_server".to_string(),
    command: "npx".to_string(),
    args: vec!["my-mcp-server".to_string()],
    enabled_tools: None,
    disabled_tools: None,
};
```

Use Stdio for local MCP servers distributed as npm packages, Python packages, or standalone binaries.

### SSE (Server-Sent Events)

Connect to a remote MCP server via SSE:

```rust,no_run
use antigravity_sdk_rust::types::McpServerConfig;

let server = McpServerConfig::Sse {
    name: "remote_server".to_string(),
    url: "https://my-server.example.com/sse".to_string(),
    headers: None,
    enabled_tools: None,
    disabled_tools: None,
};
```

### HTTP

Connect via standard HTTP with configurable timeouts:

```rust,no_run
use antigravity_sdk_rust::types::McpServerConfig;
use std::collections::HashMap;

let server = McpServerConfig::Http {
    name: "http_server".to_string(),
    url: "https://my-server.example.com/mcp".to_string(),
    headers: Some(HashMap::from([
        ("Authorization".to_string(), "Bearer my-token".to_string()),
    ])),
    timeout: 30.0,           // Connection timeout (seconds)
    sse_read_timeout: 300.0,  // SSE read timeout (seconds)
    terminate_on_close: true, // Terminate channel on close
    enabled_tools: None,
    disabled_tools: None,
};
```

## Tool Filtering

Each transport variant supports fine-grained tool control:

- **`enabled_tools`**: Allowlist — only these tools are exposed to the model
- **`disabled_tools`**: Denylist — these tools are hidden from the model

These are mutually exclusive. When both are `None`, all tools from the server are available.

```rust,no_run
use antigravity_sdk_rust::types::McpServerConfig;

// Only expose specific tools
let server = McpServerConfig::Stdio {
    name: "fs_server".to_string(),
    command: "npx".to_string(),
    args: vec!["@anthropic/mcp-fs-server".to_string()],
    enabled_tools: Some(vec!["read_file".to_string(), "list_dir".to_string()]),
    disabled_tools: None,
};

// Or disable specific tools
let server = McpServerConfig::Stdio {
    name: "fs_server".to_string(),
    command: "npx".to_string(),
    args: vec!["@anthropic/mcp-fs-server".to_string()],
    enabled_tools: None,
    disabled_tools: Some(vec!["delete_file".to_string()]),
};
```

## Agent Builder Integration

### Single Server

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::types::McpServerConfig;

let agent = Agent::builder()
    .mcp_server(McpServerConfig::Stdio {
        name: "my_server".to_string(),
        command: "npx".to_string(),
        args: vec!["my-mcp-server".to_string()],
        enabled_tools: None,
        disabled_tools: None,
    })
    .allow_all()
    .build();
```

### Multiple Servers

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::types::McpServerConfig;

let agent = Agent::builder()
    .mcp_servers(vec![
        McpServerConfig::Stdio {
            name: "fs".to_string(),
            command: "npx".to_string(),
            args: vec!["@anthropic/mcp-fs-server".to_string()],
            enabled_tools: None,
            disabled_tools: None,
        },
        McpServerConfig::Sse {
            name: "api".to_string(),
            url: "https://api.example.com/mcp/sse".to_string(),
            headers: None,
            enabled_tools: None,
            disabled_tools: None,
        },
    ])
    .allow_all()
    .build();
```

### Chaining

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::types::McpServerConfig;

let agent = Agent::builder()
    .mcp_server(McpServerConfig::Stdio {
        name: "server_a".to_string(),
        command: "npx".to_string(),
        args: vec!["server-a".to_string()],
        enabled_tools: None,
        disabled_tools: None,
    })
    .mcp_server(McpServerConfig::Stdio {
        name: "server_b".to_string(),
        command: "npx".to_string(),
        args: vec!["server-b".to_string()],
        enabled_tools: None,
        disabled_tools: None,
    })
    .allow_all()
    .build();
```

## Policy Integration

MCP tools are named `{server_name}_{tool_name}` in the policy system. The SDK provides helpers:

```rust,no_run
use antigravity_sdk_rust::policy;
use antigravity_sdk_rust::types::McpServerConfig;

let server = McpServerConfig::Stdio {
    name: "fs".to_string(),
    command: "npx".to_string(),
    args: vec!["mcp-fs-server".to_string()],
    enabled_tools: None,
    disabled_tools: None,
};

// Allow all tools from this server
let policies = policy::allow_mcp(&server, None);

// Allow only specific tools
let policies = policy::allow_mcp(&server, Some(&["read_file", "list_dir"]));

// Deny specific tools
let policies = policy::deny_mcp(&server, Some(&["delete_file"]));

// Require user confirmation for all tools
let policies = policy::ask_user_mcp(&server, None);
```

### Combined with Agent Builder

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::policy;
use antigravity_sdk_rust::types::McpServerConfig;

let fs_server = McpServerConfig::Stdio {
    name: "fs".to_string(),
    command: "npx".to_string(),
    args: vec!["mcp-fs-server".to_string()],
    enabled_tools: None,
    disabled_tools: None,
};

let agent = Agent::builder()
    .mcp_server(fs_server.clone())
    .policies(vec![
        policy::deny_all(),                              // Block everything by default
        policy::allow("VIEW_FILE"),                       // Allow built-in VIEW_FILE
        policy::allow_mcp(&fs_server, Some(&["read"])),  // Allow MCP fs.read
    ])
    .build();
```

## Architecture

```
┌─────────────────────────────────────────────┐
│  Your Rust Application                       │
│                                              │
│  Agent::builder()                            │
│    .mcp_server(McpServerConfig::Stdio {...}) │
│    .build()                                  │
│    .start().await                            │
└──────────────────┬──────────────────────────┘
                   │ Config passed via WebSocket
                   ▼
┌──────────────────────────────────────────────┐
│  localharness (subprocess)                    │
│                                              │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │ MCP      │  │ MCP      │  │ MCP       │  │
│  │ Client 1 │  │ Client 2 │  │ Client N  │  │
│  └────┬─────┘  └────┬─────┘  └─────┬─────┘  │
│       │              │              │        │
│       ▼              ▼              ▼        │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │ External │  │ External │  │ External  │  │
│  │ MCP      │  │ MCP      │  │ MCP       │  │
│  │ Server   │  │ Server   │  │ Server    │  │
│  └──────────┘  └──────────┘  └───────────┘  │
└──────────────────────────────────────────────┘
```

The SDK configures MCP servers; the localharness manages their lifecycle (connecting, tool discovery, execution, disconnection).

## Common MCP Servers

| Package | Name | Transport | Description |
|---------|------|-----------|-------------|
| `@anthropic/mcp-fs-server` | `fs` | Stdio | File system access |
| `@anthropic/mcp-memory` | `memory` | Stdio | Persistent memory |
| `@anthropic/mcp-github` | `github` | Stdio | GitHub API |
| `@anthropic/mcp-slack` | `slack` | Stdio | Slack messaging |

## McpServerConfig API

```rust,no_run
impl McpServerConfig {
    /// Returns the unique name identifier of this MCP server.
    pub fn name(&self) -> &str;
}
```

The name is used for:
- Tool call routing (tools are prefixed with the server name)
- Policy matching (`{server_name}/{tool_name}` or `{server_name}/*`)
- Logging and diagnostics

## Python SDK Comparison

| Python | Rust |
|--------|------|
| `McpStdioServer(name, command, args)` | `McpServerConfig::Stdio { name, command, args, ... }` |
| `McpSseServer(name, url, headers)` | `McpServerConfig::Sse { name, url, headers, ... }` |
| N/A | `McpServerConfig::Http { ... }` (Rust-only) |
| `LocalAgentConfig(mcp_servers=[...])` | `Agent::builder().mcp_servers(vec![...])` |
| `McpBridge` (runtime client) | Handled by localharness (not in SDK) |
