#[cfg(not(target_arch = "wasm32"))]
use antigravity_sdk_rust::proto::localharness::{InputConfig, OutputConfig};
#[cfg(not(target_arch = "wasm32"))]
use futures_util::{SinkExt, StreamExt};
#[cfg(not(target_arch = "wasm32"))]
use prost::Message;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(not(target_arch = "wasm32"))]
use tokio::net::TcpListener;
#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::accept_async;
#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::tungstenite::Message as WsMessage;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Read InputConfig (length prefix + payload) from stdin
    let mut stdin = tokio::io::stdin();
    let mut len_bytes = [0u8; 4];
    if stdin.read_exact(&mut len_bytes).await.is_err() {
        std::process::exit(1);
    }
    let length = u32::from_le_bytes(len_bytes) as usize;
    let mut input_buf = vec![0u8; length];
    stdin.read_exact(&mut input_buf).await?;

    let input_config = InputConfig::decode(&input_buf[..]).ok();
    let client_info = input_config.as_ref().and_then(|c| c.client_info.as_ref());
    let client_lang = client_info
        .and_then(|ci| ci.language.as_deref())
        .unwrap_or("unknown");
    let client_ver = client_info
        .and_then(|ci| ci.version.as_deref())
        .unwrap_or("unknown");

    // 2. Bind TCP listener to random port on localhost
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();

    // 3. Write OutputConfig to stdout
    let output_config = OutputConfig {
        port: Some(i32::from(port)),
        api_key: Some("mock_api_key".to_string()),
    };
    let mut output_buf = Vec::new();
    output_config.encode(&mut output_buf)?;

    let mut stdout = tokio::io::stdout();
    let size = output_buf.len() as u32;
    stdout.write_all(&size.to_le_bytes()).await?;
    stdout.write_all(&output_buf).await?;
    stdout.flush().await?;

    // Exit on stdin EOF, which is how `disconnect()` asks the harness to shut
    // down (the real one monitors stdin for the same reason). Without this the
    // mock outlived every test by the client's full 3-minute process-wait
    // timeout — the integration suite took six minutes, almost all of it spent
    // waiting for a process that was never going to exit on its own.
    tokio::spawn(async move {
        let mut sink = Vec::new();
        let _ = stdin.read_to_end(&mut sink).await;
        std::process::exit(0);
    });

    // 4. Accept a TCP connection and upgrade to WebSocket
    let (stream, _) = listener.accept().await?;
    let ws_stream = accept_async(stream).await?;

    // 5. Handle the WebSocket conversation
    handle_ws_connection(ws_stream, client_lang, client_ver).await
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_lines)] // one branch per scripted scenario; splitting
// them would scatter the wire format across helpers for no gain
async fn handle_ws_connection(
    mut ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    client_lang: &str,
    client_ver: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read client config message (InitializeConversationEvent / HarnessConfig)
    if let Some(msg_res) = ws_stream.next().await {
        let _ = msg_res?;
    }

    // Answer the handshake. Since 0.1.4 this is the harness's mandatory first
    // frame, and upstream's client blocks on it before doing anything else
    // (local_connection.py:1162-1176). The SDK does not read it yet — that is
    // WP-6 — but the mock must send it, or it will keep certifying a handshake
    // no real harness performs.
    let init_response = serde_json::json!({
        "initializeConversationResponse": {
            "cascadeId": "test_traj",
            "history": []
        },
        "seqNum": "1",
        "timestampMicros": "1"
    });
    ws_stream
        .send(WsMessage::Text(init_response.to_string()))
        .await?;

    // Read client user prompt message
    let mut prompt = String::new();
    if let Some(msg_res) = ws_stream.next().await {
        let msg = msg_res?;
        if let WsMessage::Text(text) = msg {
            prompt = text;
        }
    }

    // Send trajectoryStateUpdate (RUNNING) to signal the turn is active
    let traj_running = serde_json::json!({
        "trajectoryStateUpdate": {
            "trajectoryId": "test_traj",
            "state": "STATE_RUNNING"
        }
    });
    ws_stream
        .send(WsMessage::Text(traj_running.to_string()))
        .await?;

    if let Some(path) = prompt
        .split("trigger_tool_confirmation:")
        .nth(1)
        .map(|rest| rest.trim_end_matches(['"', '}', ' ']).to_string())
    {
        // Drive a real pre-tool gate: a VIEW_FILE the harness wants confirmed.
        // The SDK answers with ToolConfirmation{accepted}, which is the policy
        // layer's decision observed from the outside — the only way to prove
        // the enforcer is actually registered and consulted at runtime.
        //
        // STATE_WAITING_FOR_USER matters: the client only treats a confirmation
        // request as new while the step is in that state.
        let step_confirm = serde_json::json!({
            "stepUpdate": {
                "stepIndex": 1,
                "cascadeId": "test_traj",
                "trajectoryId": "test_traj",
                "text": "Requesting confirmation",
                "state": "STATE_WAITING_FOR_USER",
                "source": "SOURCE_MODEL",
                "target": "TARGET_USER",
                "viewFile": { "filePath": path },
                "toolConfirmationRequest": {}
            }
        });
        ws_stream
            .send(WsMessage::Text(step_confirm.to_string()))
            .await?;

        // Wait for the client's decision and report it back in the turn's text,
        // so a test can assert on it without reaching into the SDK.
        let mut accepted = "none".to_string();
        while let Some(msg_res) = ws_stream.next().await {
            let WsMessage::Text(text) = msg_res? else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
                continue;
            };
            if let Some(confirmation) = value.get("toolConfirmation") {
                accepted = confirmation
                    .get("accepted")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
                    .to_string();
                break;
            }
        }

        let step_done = serde_json::json!({
            "stepUpdate": {
                "stepIndex": 2,
                "cascadeId": "test_traj",
                "trajectoryId": "test_traj",
                "text": format!("accepted={accepted}"),
                "textDelta": format!("accepted={accepted}"),
                "state": "STATE_DONE",
                "source": "SOURCE_MODEL",
                "target": "TARGET_USER",
                "finish": { "outputString": "\"done\"" }
            }
        });
        ws_stream
            .send(WsMessage::Text(step_done.to_string()))
            .await?;
    } else if prompt.contains("trigger_crash") {
        // Die mid-turn the way a real crash does: something on stderr, then the
        // socket drops with no idle transition. The sleep gives the client's
        // stderr reader time to see the line before the socket closes; the two
        // arrive on different channels and are not ordered against each other.
        eprintln!("panic: mock harness exploded");
        tokio::time::sleep(Duration::from_millis(200)).await;
        std::process::exit(101);
    } else if prompt.contains("trigger_cancel") {
        // Emit one step, then stall until the client halts. The real harness
        // answers a halt with an ordinary STATE_FULLY_IDLE — it does *not* send
        // STATE_CANCELLED — which is exactly the case the client-side flag
        // exists to disambiguate. Waiting for the frame rather than sleeping
        // keeps the test deterministic.
        let step1 = serde_json::json!({
            "stepUpdate": {
                "stepIndex": 1,
                "cascadeId": "test_traj",
                "trajectoryId": "test_traj",
                "text": "Working...",
                "state": "STATE_ACTIVE",
                "source": "SOURCE_MODEL",
                "target": "TARGET_USER"
            }
        });
        ws_stream.send(WsMessage::Text(step1.to_string())).await?;

        while let Some(msg_res) = ws_stream.next().await {
            match msg_res? {
                WsMessage::Text(text) if text.contains("haltRequest") => break,
                WsMessage::Close(_) => break,
                _ => {}
            }
        }
    } else if prompt.contains("trigger_terminal_error") {
        let step_terminal = serde_json::json!({
            "stepUpdate": {
                "stepIndex": 1,
                "cascadeId": "test_traj",
                "trajectoryId": "test_traj",
                "text": "Terminal error triggered",
                "state": "STATE_ERROR",
                "source": "SOURCE_MODEL",
                "target": "TARGET_USER",
                "errorMessage": "Terminal error triggered by prompt"
            }
        });
        ws_stream
            .send(WsMessage::Text(step_terminal.to_string()))
            .await?;
    } else {
        // Send the step updates including client language/version info in output
        let step1 = serde_json::json!({
            "stepUpdate": {
                "stepIndex": 1,
                "cascadeId": "test_traj",
                "trajectoryId": "test_traj",
                "text": format!("Client info language: {}, version: {}", client_lang, client_ver),
                "textDelta": format!("Client info language: {}, version: {}", client_lang, client_ver),
                "state": "STATE_ACTIVE",
                "source": "SOURCE_MODEL",
                "target": "TARGET_USER"
            }
        });
        ws_stream.send(WsMessage::Text(step1.to_string())).await?;

        tokio::time::sleep(Duration::from_millis(100)).await;

        let step2 = serde_json::json!({
            "stepUpdate": {
                "stepIndex": 2,
                "cascadeId": "test_traj",
                "trajectoryId": "test_traj",
                "text": "Hello from mock harness!How can I help you today?",
                "textDelta": "How can I help you today?",
                "state": "STATE_DONE",
                "source": "SOURCE_MODEL",
                "target": "TARGET_USER",
                "finish": {
                    "outputString": "\"done\""
                }
            }
        });
        ws_stream.send(WsMessage::Text(step2.to_string())).await?;
    }

    // Send trajectoryStateUpdate (IDLE) to signal the turn is complete
    let traj_idle = serde_json::json!({
        "trajectoryStateUpdate": {
            "trajectoryId": "test_traj",
            // Renamed from STATE_IDLE upstream in 0.1.9. The numeric value is
            // still 2, but protojson matches on the value NAME, so the old
            // spelling is dropped as an unknown variant and the turn never ends.
            "state": "STATE_FULLY_IDLE"
        }
    });
    ws_stream
        .send(WsMessage::Text(traj_idle.to_string()))
        .await?;

    // Keep reading until client disconnects or we get terminated
    while let Some(msg_res) = ws_stream.next().await {
        if msg_res.is_err() {
            break;
        }
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
