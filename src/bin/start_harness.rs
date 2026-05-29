#[cfg(not(target_arch = "wasm32"))]
use antigravity_sdk_rust::proto::localharness::{InputConfig, OutputConfig};
#[cfg(not(target_arch = "wasm32"))]
use prost::Message;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{Read, Write};
#[cfg(not(target_arch = "wasm32"))]
use std::process::{Command, Stdio};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting localharness...");

    // Determine binary path
    let binary_path = if let Ok(path) = std::env::var("ANTIGRAVITY_HARNESS_PATH") {
        path
    } else {
        let p = std::path::Path::new("bin/localharness");
        if p.exists() {
            p.to_string_lossy().into_owned()
        } else {
            return Err("localharness binary not found. Run 'just install' first or set ANTIGRAVITY_HARNESS_PATH.".into());
        }
    };

    println!("Using harness binary at: {}", binary_path);

    // Spawn the child process
    let mut child = Command::new(&binary_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;
    let mut stdout = child.stdout.take().ok_or("Failed to open stdout")?;

    // Create InputConfig
    let input_config = InputConfig {
        storage_directory: Some("target/harness_store".to_string()),
        port: Some(8000),
        bind_address: Some("127.0.0.1".to_string()),
        client_info: None,
    };

    // Serialize InputConfig
    let mut input_bytes = Vec::new();
    input_config.encode(&mut input_bytes)?;

    // Write 4-byte length prefix (little-endian)
    let len = input_bytes.len() as u32;
    stdin.write_all(&len.to_le_bytes())?;
    stdin.write_all(&input_bytes)?;
    stdin.flush()?;

    // Read 4-byte length prefix from stdout
    let mut len_bytes = [0u8; 4];
    stdout.read_exact(&mut len_bytes)?;
    let resp_len = u32::from_le_bytes(len_bytes) as usize;

    // Read OutputConfig bytes
    let mut resp_bytes = vec![0u8; resp_len];
    stdout.read_exact(&mut resp_bytes)?;

    // Decode OutputConfig
    let output_config = OutputConfig::decode(&resp_bytes[..])?;
    let port = output_config.port.ok_or("Harness output missing port")?;
    let api_key = output_config
        .api_key
        .ok_or("Harness output missing api_key")?;

    println!("\n==========================================");
    println!("Localharness is running successfully!");
    println!("WebSocket Port: {}", port);
    println!("Harness API Key: {}", api_key);
    println!("==========================================\n");

    // Write to a .env file so the Spin application can read it automatically
    let mut file = std::fs::File::create(".env")?;
    writeln!(file, "ANTIGRAVITY_API_KEY={}", api_key)?;
    writeln!(file, "ANTIGRAVITY_HARNESS_PORT={}", port)?;
    println!("Configuration written to .env file.");

    // Keep running and monitor process
    let status = child.wait()?;
    println!("Harness exited with status: {}", status);

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
