//! Deposit composable: generate note → preview root → build + send tx.
//!
//! Uses the wallet's `signAndSendTransaction` for a single-call flow:
//! 1. Build kit `Instruction` via Codama.
//! 2. Build kit `TransactionMessage`.
//! 3. Convert to wire format (base64).
//! 4. `provider.signAndSendTransaction(base64)` → signature.
//! 5. Poll RPC until confirmed.

import { ref, shallowRef } from "vue";
import {
  createSolanaRpc,
  createTransactionMessage,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  compileTransaction,
  getBase64EncodedWireTransaction,
} from "@solana/kit";
import { getDepositInstructionAsync } from "../client";
import { getCommitments, postRootPreview } from "../api/client";
import { generateNote, type DepositNote } from "../deposit/generateNote";
import { useWalletStore } from "../stores/wallet";
import { makeNoopSigner } from "../wallet/kitSigner";

const RPC_URL = "https://api.devnet.solana.com";

export interface DepositResult {
  signature: string;
  note: DepositNote;
  newRoot: string;
}

export function useDeposit() {
  const wallet = useWalletStore();
  const loading = ref(false);
  const error = ref<string | null>(null);
  const result = shallowRef<DepositResult | null>(null);

  async function deposit(amountLamports: bigint): Promise<DepositResult | null> {
    loading.value = true;
    error.value = null;
    try {
      const walletAddr = wallet.addr;
      const provider = wallet.provider;
      if (!walletAddr || !provider) {
        throw new Error("wallet not connected");
      }
      if (!provider.signAndSendTransaction) {
        throw new Error("wallet does not support signAndSendTransaction");
      }

      // 1. Generate note + commitment (browser-side noir_js).
      const note = await generateNote(amountLamports);

      // 2. Fetch current commitments + preview new root.
      const { commitments } = await getCommitments();
      const commitmentHexes = commitments.map((c) => c.commitment);
      commitmentHexes.push(note.commitment);
      const { root: newRoot } = await postRootPreview(commitmentHexes);

      // 3. Build the deposit instruction.
      //    Codama requires a TransactionSigner for `depositor`. We supply
      //    a noop signer — real signing happens via the wallet provider
      //    on the serialized wire transaction (step 7).
      const ix = await getDepositInstructionAsync({
        depositor: makeNoopSigner(walletAddr),
        commitment: hexToBytes(note.commitment),
        newRoot: hexToBytes(newRoot),
        amount: amountLamports,
      });

      // 4. Fetch a recent blockhash.
      const rpc = createSolanaRpc(RPC_URL);
      const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

      // 5. Build the transaction message.
      const message = appendTransactionMessageInstruction(
        ix,
        setTransactionMessageLifetimeUsingBlockhash(
          latestBlockhash,
          setTransactionMessageFeePayer(walletAddr, createTransactionMessage({ version: 0 })),
        ),
      );

      // 6. Compile to wire format (base64).
      const compiled = compileTransaction(message);
      const wireBase64 = getBase64EncodedWireTransaction(compiled);

      // 7. Sign + send via wallet.
      const { signature } = await provider.signAndSendTransaction(wireBase64);

      // 8. Poll for confirmation (best-effort; does not block the return).
      void confirmSignature(rpc, signature);

      const out: DepositResult = { signature, note, newRoot };
      result.value = out;
      return out;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      return null;
    } finally {
      loading.value = false;
    }
  }

  return { loading, error, result, deposit };
}

async function confirmSignature(
  rpc: ReturnType<typeof createSolanaRpc>,
  signature: string,
): Promise<void> {
  for (let i = 0; i < 30; i++) {
    await new Promise((r) => setTimeout(r, 1000));
    try {
      const { value } = await rpc.getSignatureStatuses([signature as never]).send();
      const status = value[0];
      if (
        status?.confirmationStatus === "confirmed" ||
        status?.confirmationStatus === "finalized"
      ) {
        return;
      }
      if (status?.err) {
        console.error("deposit failed:", status.err);
        return;
      }
    } catch {
      // Ignore transient errors; keep polling.
    }
  }
}

function hexToBytes(hex: string): Uint8Array {
  if (hex.length % 2 !== 0) throw new Error(`odd hex length: ${hex.length}`);
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < out.length; i++) {
    out[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  }
  return out;
}
