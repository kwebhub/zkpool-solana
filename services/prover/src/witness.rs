//! Serialization of withdrawal witness inputs into `Prover.toml`.
//!
//! `nargo execute -p <name>` reads `<circuit_dir>/<name>.toml`. All hex
//! field values MUST be `0x`-prefixed (verified: bare hex is rejected by
//! the nargo TOML deserializer for `amount` and `merkle_proof` elements).

use anyhow::{bail, Result};

/// Merkle tree depth — must match the circuit.
pub const TREE_DEPTH: usize = 20;

/// Withdrawal witness inputs, as received over HTTP.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct WitnessInputs {
    // Public
    pub root: String,
    pub nullifier_hash: String,
    pub recipient: String,
    pub recipient_binding: String,
    pub amount: String,
    // Private
    pub nullifier: String,
    pub secret: String,
    pub note_secret: String,
    pub merkle_proof: Vec<String>,
    pub is_even: Vec<bool>,
}

impl WitnessInputs {
    /// Validate shape and length. Called before serialization.
    pub fn validate(&self) -> Result<()> {
        if self.merkle_proof.len() != TREE_DEPTH {
            bail!(
                "merkle_proof must have {} elements, got {}",
                TREE_DEPTH,
                self.merkle_proof.len()
            );
        }
        if self.is_even.len() != TREE_DEPTH {
            bail!(
                "is_even must have {} elements, got {}",
                TREE_DEPTH,
                self.is_even.len()
            );
        }
        for (name, v) in [
            ("root", &self.root),
            ("nullifier_hash", &self.nullifier_hash),
            ("recipient", &self.recipient),
            ("recipient_binding", &self.recipient_binding),
            ("amount", &self.amount),
            ("nullifier", &self.nullifier),
            ("secret", &self.secret),
            ("note_secret", &self.note_secret),
        ] {
            if v.is_empty() {
                bail!("{} must not be empty", name);
            }
        }
        Ok(())
    }

    /// Render as TOML text for `nargo execute`.
    ///
    /// All hex values are `0x`-prefixed, because nargo's TOML deserializer
    /// rejects bare hex for `amount` and `merkle_proof[i]`.
    pub fn to_toml(&self) -> Result<String> {
        self.validate()?;

        let mut s = String::new();
        s.push_str(&format!("root = \"{}\"\n", with_0x(&self.root)));
        s.push_str(&format!(
            "nullifier_hash = \"{}\"\n",
            with_0x(&self.nullifier_hash)
        ));
        s.push_str(&format!("recipient = \"{}\"\n", with_0x(&self.recipient)));
        s.push_str(&format!(
            "recipient_binding = \"{}\"\n",
            with_0x(&self.recipient_binding)
        ));
        s.push_str(&format!("amount = \"{}\"\n", with_0x(&self.amount)));
        s.push_str(&format!("nullifier = \"{}\"\n", with_0x(&self.nullifier)));
        s.push_str(&format!("secret = \"{}\"\n", with_0x(&self.secret)));
        s.push_str(&format!(
            "note_secret = \"{}\"\n",
            with_0x(&self.note_secret)
        ));

        s.push_str("merkle_proof = [");
        for (i, v) in self.merkle_proof.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(&format!("\"{}\"", with_0x(v)));
        }
        s.push_str("]\n");

        s.push_str("is_even = [");
        for (i, v) in self.is_even.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(if *v { "true" } else { "false" });
        }
        s.push_str("]\n");

        Ok(s)
    }
}

/// Ensure a hex string has the `0x` prefix.
fn with_0x(s: &str) -> String {
    if s.starts_with("0x") || s.starts_with("0X") {
        s.to_string()
    } else {
        format!("0x{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> WitnessInputs {
        WitnessInputs {
            root: "1e8508c3c11def8bfecd33c4bf97ce7fd065fea15eafe55c47e35bd056f1c6b2".to_string(),
            nullifier_hash: "1412cc9d862599e6869a1881c8562062b98537456d9035819288219b5cd3e6e4"
                .to_string(),
            recipient: "062afbde1181c71c".to_string(),
            recipient_binding: "200cfdb247b0436ba0483327abcf8f83ed65f75a0db8d9ebbb611561d7d3b2e1"
                .to_string(),
            amount: "0f4240".to_string(),
            nullifier: "018abef7846071c7".to_string(),
            secret: "03157def08c0e38e".to_string(),
            note_secret: "04a03ce68d215555".to_string(),
            merkle_proof: std::iter::once("07b5bad595e238e3".to_string())
                .chain(std::iter::repeat("00".to_string()).take(19))
                .collect(),
            is_even: vec![true; TREE_DEPTH],
        }
    }

    #[test]
    fn test_validate_ok() {
        sample().validate().expect("valid");
    }

    #[test]
    fn test_validate_wrong_merkle_len() {
        let mut w = sample();
        w.merkle_proof.pop();
        assert!(w.validate().is_err());
    }

    #[test]
    fn test_validate_wrong_is_even_len() {
        let mut w = sample();
        w.is_even.pop();
        assert!(w.validate().is_err());
    }

    #[test]
    fn test_to_toml_has_0x_prefix() {
        let t = sample().to_toml().expect("toml");
        assert!(t.contains("root = \"0x1e85"));
        assert!(t.contains("amount = \"0x0f4240\""));
        assert!(t.contains("merkle_proof = [\"0x07b5bad595e238e3\""));
    }

    #[test]
    fn test_to_toml_idempotent_0x() {
        let mut w = sample();
        w.root = "0x1e8508c3c11def8bfecd33c4bf97ce7fd065fea15eafe55c47e35bd056f1c6b2".to_string();
        let t = w.to_toml().expect("toml");
        // Must not become 0x0x...
        assert!(t.contains("root = \"0x1e85"));
        assert!(!t.contains("0x0x"));
    }

    #[test]
    fn test_to_toml_lists_lengths() {
        let t = sample().to_toml().expect("toml");
        let proof_line = t.lines().find(|l| l.starts_with("merkle_proof")).unwrap();
        // 20 hex strings -> 20 occurrences of "0x
        assert_eq!(proof_line.matches("\"0x").count(), TREE_DEPTH);
        let is_even_line = t.lines().find(|l| l.starts_with("is_even")).unwrap();
        assert_eq!(is_even_line.matches("true").count(), TREE_DEPTH);
    }
}
