//! pool-init — initialize the zkpool on-chain pool on Solana devnet.
//!
//! Builds the `pool` instruction by hand (8-byte discriminator + 3 accounts
//! + system program) and sends it via raw JSON-RPC. No `solana-client`
//! dependency — avoids version conflicts with the modular `solana-*` crates.
//!
//! Usage:
//!   pool-init [--rpc <url>] [--keypair <path>] [--program <pubkey>]
//!
//! Defaults:
//!   rpc      = https://api.devnet.solana.com
//!   keypair  = ~/.config/solana/id.json
//!   program  = 8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use solana_hash::Hash;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

// ---- Constants from the on-chain program ----

/// `pool` instruction discriminator (from `PROJECT_CONTEXT.md` §6).
const POOL_DISCRIMINATOR: [u8; 8] = [134, 215, 119, 168, 28, 199, 193, 127];

/// PDA seed for `PoolState`.
const POOL_SEED: &[u8] = b"pool";

/// PDA seed for `vault`.
const VAULT_SEED: &[u8] = b"vault";

/// Default program ID.
const DEFAULT_PROGRAM: &str = "8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm";

/// Default RPC URL.
const DEFAULT_RPC: &str = "https://api.devnet.solana.com";

/// System program ID.
const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";

#[derive(Debug)]
struct Args {
    rpc: String,
    keypair: PathBuf,
    program: Pubkey,
}

impl Args {
    fn from_env_and_argv() -> Result<Self> {
        let mut rpc = DEFAULT_RPC.to_string();
        let mut keypair = dirs_home()
            .context("cannot determine home directory")?
            .join(".config/solana/id.json");
        let mut program = Pubkey::from_str(DEFAULT_PROGRAM)?;

        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--rpc" => rpc = args.next().context("--rpc requires a value")?,
                "--keypair" => {
                    keypair = PathBuf::from(args.next().context("--keypair requires a value")?)
                }
                "--program" => {
                    program = Pubkey::from_str(&args.next().context("--program requires a value")?)?
                }
                "-h" | "--help" => {
                    println!(
                        "Usage: pool-init [--rpc <url>] [--keypair <path>] [--program <pubkey>]"
                    );
                    std::process::exit(0);
                }
                other => bail!("unknown argument: {}", other),
            }
        }

        Ok(Self {
            rpc,
            keypair,
            program,
        })
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_env_and_argv()?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    println!("RPC:      {}", args.rpc);
    println!("Keypair:  {}", args.keypair.display());
    println!("Program:  {}", args.program);

    // Load keypair.
    let keypair = load_keypair(&args.keypair)?;
    let authority = keypair.pubkey();
    println!("Authority: {}", authority);

    // Derive PDAs.
    let (pool_pda, pool_bump) =
        Pubkey::derive_program_address(&[POOL_SEED], &args.program).context("derive pool PDA")?;
    let (vault_pda, vault_bump) =
        Pubkey::derive_program_address(&[VAULT_SEED, pool_pda.as_ref()], &args.program)
            .context("derive vault PDA")?;
    println!("Pool PDA:  {} (bump {})", pool_pda, pool_bump);
    println!("Vault PDA: {} (bump {})", vault_pda, vault_bump);

    // Check if pool already exists.
    if account_exists(&client, &args.rpc, &pool_pda).await? {
        println!("\n⚠️  Pool PDA already exists — nothing to do.");
        print_pool_state(&client, &args.rpc, &pool_pda).await?;
        return Ok(());
    }

    // Build instruction.
    let system_program = Pubkey::from_str(SYSTEM_PROGRAM)?;
    let ix = Instruction {
        program_id: args.program,
        accounts: vec![
            AccountMeta::new(authority, true),  // authority (signer, writable)
            AccountMeta::new(pool_pda, false),  // pool (writable)
            AccountMeta::new(vault_pda, false), // vault (writable)
            AccountMeta::new_readonly(system_program, false), // system_program
        ],
        data: POOL_DISCRIMINATOR.to_vec(),
    };

    // Fetch recent blockhash.
    let blockhash = get_latest_blockhash(&client, &args.rpc).await?;
    println!("\nBlockhash: {}", blockhash);

    // Build + sign transaction.
    let mut tx = Transaction::new_with_payer(&[ix], Some(&authority));
    tx.sign(&[&keypair], blockhash);
    let tx_bytes = bincode::serialize(&tx).context("serialize transaction")?;
    println!("Sending transaction ({} bytes)...", tx_bytes.len());

    // Send.
    let sig = send_transaction(&client, &args.rpc, &tx_bytes).await?;
    println!("Signature: {}", sig);

    // Confirm.
    println!("Waiting for confirmation...");
    let confirmed = wait_for_confirmation(&client, &args.rpc, &sig).await?;
    if !confirmed {
        bail!("transaction not confirmed within timeout");
    }
    println!("✅ Confirmed");

    // Verify pool state.
    print_pool_state(&client, &args.rpc, &pool_pda).await?;

    Ok(())
}

fn load_keypair(path: &PathBuf) -> Result<Keypair> {
    let data =
        std::fs::read(path).with_context(|| format!("reading keypair from {}", path.display()))?;
    let bytes: Vec<u8> = serde_json::from_slice(&data)
        .with_context(|| format!("parsing keypair JSON from {}", path.display()))?;
    Keypair::try_from(bytes.as_slice()).context("invalid keypair bytes")
}

async fn account_exists(client: &reqwest::Client, rpc: &str, pubkey: &Pubkey) -> Result<bool> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [pubkey.to_string(), {"encoding": "base64"}],
    });
    let resp: Value = client.post(rpc).json(&body).send().await?.json().await?;
    let value = resp.pointer("/result/value");
    Ok(matches!(value, Some(v) if !v.is_null()))
}

async fn get_latest_blockhash(client: &reqwest::Client, rpc: &str) -> Result<Hash> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getLatestBlockhash",
        "params": [{"commitment": "confirmed"}],
    });
    let resp: Value = client.post(rpc).json(&body).send().await?.json().await?;
    let bh_str = resp
        .pointer("/result/value/blockhash")
        .and_then(|v| v.as_str())
        .context("getLatestBlockhash response missing blockhash")?;
    Ok(Hash::from_str(bh_str)?)
}

async fn send_transaction(client: &reqwest::Client, rpc: &str, tx_bytes: &[u8]) -> Result<String> {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(tx_bytes);
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sendTransaction",
        "params": [b64, {"encoding": "base64", "skipPreflight": false}],
    });
    let resp: Value = client.post(rpc).json(&body).send().await?.json().await?;
    if let Some(err) = resp.get("error") {
        bail!("sendTransaction RPC error: {}", err);
    }
    resp.pointer("/result")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .context("sendTransaction response missing signature")
}

async fn wait_for_confirmation(client: &reqwest::Client, rpc: &str, sig: &str) -> Result<bool> {
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSignatureStatuses",
            "params": [[sig], {"searchTransactionHistory": true}],
        });
        let resp: Value = client.post(rpc).json(&body).send().await?.json().await?;
        let status = resp.pointer("/result/value/0");
        if let Some(s) = status {
            if !s.is_null() {
                let confirmation = s.get("confirmationStatus").and_then(|v| v.as_str());
                if matches!(confirmation, Some("confirmed") | Some("finalized")) {
                    return Ok(true);
                }
                if let Some(err) = s.get("err") {
                    if !err.is_null() {
                        bail!("transaction failed: {}", err);
                    }
                }
            }
        }
    }
    Ok(false)
}

async fn print_pool_state(client: &reqwest::Client, rpc: &str, pubkey: &Pubkey) -> Result<()> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [pubkey.to_string(), {"encoding": "base64"}],
    });
    let resp: Value = client.post(rpc).json(&body).send().await?.json().await?;
    let data_b64 = resp
        .pointer("/result/value/data/0")
        .and_then(|v| v.as_str());
    match data_b64 {
        Some(b64) => {
            use base64::Engine;
            let data = base64::engine::general_purpose::STANDARD.decode(b64)?;
            println!("\n✅ Pool account exists ({} bytes)", data.len());
            // Layout: 8 (discriminator) + 32 (authority) + 8 (next_leaf_index)
            //         + 8 (total_deposits) + 1 (current_root_index)
            //         + ROOT_HISTORY_SIZE * 32 (roots)
            if data.len() >= 8 + 32 + 8 + 8 + 1 {
                let authority = Pubkey::try_from(&data[8..40]).context("authority")?;
                let next_leaf_index = u64::from_le_bytes(data[40..48].try_into().unwrap());
                let total_deposits = u64::from_le_bytes(data[48..56].try_into().unwrap());
                let current_root_index = data[56];
                println!("   authority:          {}", authority);
                println!("   next_leaf_index:    {}", next_leaf_index);
                println!("   total_deposits:     {}", total_deposits);
                println!("   current_root_index: {}", current_root_index);
            }
        }
        None => {
            println!("\n⚠️  Pool account does not exist");
        }
    }
    Ok(())
}
