//! Integration tests against the deployed testnet contract in `deployments/testnet.json`.
//!
//! These tests invoke the live contract via the Stellar CLI (read-only view calls).
//! Run locally with:
//!   cargo test testnet_integration -- --ignored --nocapture
//!
//! Override the contract ID with `SOROSTREAM_TESTNET_CONTRACT` or populate
//! `deployments/testnet.json`.


extern crate std;

use std::eprintln;
use std::path::PathBuf;
use std::process::Command;
use std::string::ToString;

const DEPLOYMENT_MANIFEST: &str = "../../deployments/testnet.json";

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEPLOYMENT_MANIFEST)
}

fn load_stream_contract_id() -> Option<std::string::String> {
    if let Ok(id) = std::env::var("SOROSTREAM_TESTNET_CONTRACT") {
        let id = id.trim().to_string();
        if !id.is_empty() {
            return Some(id);
        }
    }

    let raw = std::fs::read_to_string(manifest_path()).ok()?;
    let manifest: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let id = manifest
        .get("StreamContract")?
        .as_str()?
        .trim()
        .to_string();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

fn stellar_available() -> bool {
    Command::new("stellar")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn source_account() -> std::string::String {
    match std::env::var("SOROSTREAM_TESTNET_SOURCE") {
        Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
        _ => panic!(
            "Set SOROSTREAM_TESTNET_SOURCE to a funded Stellar CLI identity for testnet invokes"
        ),
    }
}

fn invoke_read_only(contract_id: &str, method: &str, extra_args: &[&str]) -> std::io::Result<std::process::Output> {
    let mut cmd = Command::new("stellar");
    cmd.args([
        "contract",
        "invoke",
        "--id",
        contract_id,
        "--source-account",
        &source_account(),
        "--network",
        "testnet",
        "--",
        method,
    ]);
    cmd.args(extra_args);
    cmd.output()
}

fn require_testnet_contract() -> std::string::String {
    let contract_id = load_stream_contract_id().unwrap_or_else(|| {
        eprintln!(
            "Skipping testnet integration: set StreamContract in deployments/testnet.json \
             or export SOROSTREAM_TESTNET_CONTRACT"
        );
        panic!("testnet contract id not configured");
    });

    assert!(
        stellar_available(),
        "stellar CLI is required for testnet integration tests"
    );

    contract_id
}

fn assert_invoke_success(output: &std::process::Output, method: &str) {
    assert!(
        output.status.success(),
        "stellar contract invoke {method} failed\nstdout: {}\nstderr: {}",
        std::string::String::from_utf8_lossy(&output.stdout),
        std::string::String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn testnet_manifest_loads() {
    let raw = std::fs::read_to_string(manifest_path())
        .expect("deployments/testnet.json should exist and be readable");
    let manifest: serde_json::Value =
        serde_json::from_str(&raw).expect("deployments/testnet.json should be valid JSON");
    assert!(
        manifest.get("StreamContract").is_some(),
        "deployments/testnet.json must contain StreamContract"
    );
}

#[test]
#[ignore = "requires deployed testnet contract; run with: cargo test testnet_integration -- --ignored"]
fn testnet_integration_contract_is_initialized() {
    let contract_id = require_testnet_contract();

    let output = invoke_read_only(&contract_id, "get_admin", &[])
        .expect("failed to run stellar CLI");
    assert_invoke_success(&output, "get_admin");
}

#[test]
#[ignore = "requires deployed testnet contract; run with: cargo test testnet_integration -- --ignored"]
fn testnet_integration_pause_state_readable() {
    let contract_id = require_testnet_contract();

    let output = invoke_read_only(&contract_id, "is_paused", &[])
        .expect("failed to run stellar CLI");
    assert_invoke_success(&output, "is_paused");
}

#[test]
#[ignore = "requires deployed testnet contract; run with: cargo test testnet_integration -- --ignored"]
fn testnet_integration_stats_readable() {
    let contract_id = require_testnet_contract();

    let output = invoke_read_only(&contract_id, "get_stats", &[])
        .expect("failed to run stellar CLI");
    assert_invoke_success(&output, "get_stats");
}

#[test]
#[ignore = "requires deployed testnet contract; run with: cargo test testnet_integration -- --ignored"]
fn testnet_integration_protocol_fee_readable() {
    let contract_id = require_testnet_contract();

    let output = invoke_read_only(&contract_id, "get_protocol_fee_info", &[])
        .expect("failed to run stellar CLI");
    assert_invoke_success(&output, "get_protocol_fee_info");
}

#[test]
#[ignore = "requires deployed testnet contract; run with: cargo test testnet_integration -- --ignored"]
fn testnet_integration_stream_ids_paginated() {
    let contract_id = require_testnet_contract();

    let output = invoke_read_only(&contract_id, "get_all_stream_ids", &["--start", "0", "--limit", "5"])
        .expect("failed to run stellar CLI");
    assert_invoke_success(&output, "get_all_stream_ids");
}
