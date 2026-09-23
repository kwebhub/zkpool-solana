//! Validates that `circuits/withdrawal/spec.json` is internally consistent
//! and matches its consumers (.nr, Rust, TypeScript).
//!
//! This is a skeleton — stage 2.1.1. It only:
//!   - finds the project root,
//!   - loads and parses spec.json,
//!   - prints a summary.
//!
//! Actual validation rules are added in later sub-stages.

use anyhow::{Context, Result};
use std::path::PathBuf;

fn main() -> Result<()> {
    let project_root = find_project_root()?;
    println!("📁 Project root: {}", project_root.display());

    let spec_path = project_root.join("circuits/withdrawal/spec.json");
    println!("📄 Spec: {}", spec_path.display());

    let raw = std::fs::read_to_string(&spec_path)
        .with_context(|| format!("Failed to read {}", spec_path.display()))?;

    let spec: serde_json::Value =
        serde_json::from_str(&raw).context("Failed to parse spec.json as JSON")?;

    print_summary(&spec)?;

    println!("✅ Spec loaded successfully (validation rules not yet implemented).");
    Ok(())
}

/// Finds the project root (directory containing `circuits/` and `onchain/`).
fn find_project_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir().context("Failed to get current dir")?;

    loop {
        if dir.join("circuits").is_dir() && dir.join("onchain").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            anyhow::bail!(
                "Could not find project root (directory with `circuits/` and `onchain/`)"
            );
        }
    }
}

/// Prints a short summary of what the spec contains.
fn print_summary(spec: &serde_json::Value) -> Result<()> {
    let version = spec
        .get("version")
        .and_then(|v| v.as_u64())
        .context("spec.json: missing `version`")?;

    let circuit = spec
        .get("circuit")
        .context("spec.json: missing `circuit`")?;

    let name = circuit
        .get("name")
        .and_then(|v| v.as_str())
        .context("spec.json: missing `circuit.name`")?;

    let tree_depth = circuit
        .get("tree_depth")
        .and_then(|v| v.as_u64())
        .context("spec.json: missing `circuit.tree_depth`")?;

    let nr_public_inputs = circuit
        .get("nr_public_inputs")
        .and_then(|v| v.as_u64())
        .context("spec.json: missing `circuit.nr_public_inputs`")?;

    let public_inputs = spec
        .get("public_inputs")
        .and_then(|v| v.as_array())
        .context("spec.json: missing `public_inputs` array")?;

    let private_inputs = spec
        .get("private_inputs")
        .and_then(|v| v.as_array())
        .context("spec.json: missing `private_inputs` array")?;

    let witness_total = spec
        .get("witness_layout")
        .and_then(|v| v.get("total_bytes"))
        .and_then(|v| v.as_u64())
        .context("spec.json: missing `witness_layout.total_bytes`")?;

    println!();
    println!("=== Spec summary ===");
    println!("  version:            {}", version);
    println!("  circuit name:       {}", name);
    println!("  tree depth:         {}", tree_depth);
    println!("  nr public inputs:   {}", nr_public_inputs);
    println!("  public inputs:      {} entries", public_inputs.len());
    println!("  private inputs:     {} entries", private_inputs.len());
    println!("  witness total:      {} bytes", witness_total);
    println!();

    println!("  Public inputs (in order):");
    for (i, pi) in public_inputs.iter().enumerate() {
        let name = pi
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("<unnamed>");
        let ty = pi
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("<untyped>");
        let bytes = pi.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);
        println!("    [{}] {} : {} ({} bytes)", i, name, ty, bytes);
    }
    println!();

    println!("  Private inputs:");
    for (i, pi) in private_inputs.iter().enumerate() {
        let name = pi
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("<unnamed>");
        let ty = pi
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("<untyped>");
        println!("    [{}] {} : {}", i, name, ty);
    }
    println!();

    Ok(())
}
