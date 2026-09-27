<template lang="pug">
.deposit-form
  h2 Deposit SOL

  .field
    label(for="amount") Amount (SOL)
    input#amount(
      v-model="amountSol"
      type="text"
      placeholder="0.01"
      :disabled="deposit.loading.value"
    )
    p.hint
      | Minimum: {{ minSol }} SOL

  button.primary(
    :disabled="!canDeposit"
    @click="onDeposit"
  )
    | {{ deposit.loading.value ? "Depositing…" : "Deposit" }}

  p.error(v-if="deposit.error.value") {{ deposit.error.value }}

  .result(v-if="deposit.result.value")
    h3 ✅ Deposit submitted
    dl
      dt Signature
      dd
        a(
          :href="explorerUrl"
          target="_blank"
          rel="noopener"
        ) {{ shortSig }}
      dt Commitment
      dd.mono {{ deposit.result.value.note.commitment }}
      dt New root
      dd.mono {{ deposit.result.value.newRoot }}

    .note
      h4 ⚠️ Save this note
      p
        | The following secrets are required to withdraw later.
        strong  Loss of this note means loss of funds.
      dl
        dt nullifier
        dd.mono {{ deposit.result.value.note.nullifier }}
        dt secret
        dd.mono {{ deposit.result.value.note.secret }}
        dt note_secret
        dd.mono {{ deposit.result.value.note.noteSecret }}
        dt amount (hex)
        dd.mono {{ deposit.result.value.note.amount }}

      button.secondary(@click="copyNote") Copy note as JSON
      p.copied(v-if="copied") Copied!
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { useDeposit } from "../composables/useDeposit";
import { useWalletStore } from "../stores/wallet";
import { LAMPORTS_PER_SOL, MIN_DEPOSIT_AMOUNT } from "../constants";

const wallet = useWalletStore();
const deposit = useDeposit();

const amountSol = ref("0.01");
const copied = ref(false);

const minSol = computed(() => (Number(MIN_DEPOSIT_AMOUNT) / Number(LAMPORTS_PER_SOL)).toString());

const canDeposit = computed(() => {
  if (!wallet.connected) return false;
  if (deposit.loading.value) return false;
  const n = Number(amountSol.value);
  if (!Number.isFinite(n) || n <= 0) return false;
  const lamports = BigInt(Math.floor(n * Number(LAMPORTS_PER_SOL)));
  return lamports >= MIN_DEPOSIT_AMOUNT;
});

const explorerUrl = computed(() => {
  const sig = deposit.result.value?.signature;
  if (!sig) return "#";
  return `https://explorer.solana.com/tx/${sig}?cluster=devnet`;
});

const shortSig = computed(() => {
  const sig = deposit.result.value?.signature;
  if (!sig) return "";
  return `${sig.slice(0, 8)}…${sig.slice(-8)}`;
});

async function onDeposit() {
  copied.value = false;
  const lamports = BigInt(Math.floor(Number(amountSol.value) * Number(LAMPORTS_PER_SOL)));
  await deposit.deposit(lamports);
}

async function copyNote() {
  const r = deposit.result.value;
  if (!r) return;
  const json = JSON.stringify(
    {
      nullifier: r.note.nullifier,
      secret: r.note.secret,
      note_secret: r.note.noteSecret,
      amount: r.note.amount,
      commitment: r.note.commitment,
      nullifier_hash: r.note.nullifierHash,
      tx_signature: r.signature,
      pool_pda: "B89Yhoecj9AKJEDXT49DfjbTJoqjovmcKgYmqdzQwBYf",
    },
    null,
    2,
  );
  try {
    await navigator.clipboard.writeText(json);
    copied.value = true;
    setTimeout(() => (copied.value = false), 2000);
  } catch {
    // Clipboard not available; ignore.
  }
}
</script>

<style lang="scss" scoped>
.deposit-form {
  max-width: 600px;

  h2 {
    margin-top: 0;
  }

  .field {
    margin-bottom: 1rem;

    label {
      display: block;
      font-weight: 600;
      margin-bottom: 0.25rem;
    }

    input {
      width: 100%;
      padding: 0.5rem;
      border: 1px solid #ccc;
      border-radius: 4px;
      font-size: 1rem;
      font-family: ui-monospace, monospace;

      &:disabled {
        opacity: 0.6;
      }
    }

    .hint {
      color: #666;
      font-size: 0.85rem;
      margin: 0.25rem 0 0;
    }
  }

  button.primary {
    padding: 0.6rem 1.2rem;
    border: 1px solid #333;
    background: #333;
    color: #fff;
    border-radius: 4px;
    font-size: 1rem;
    cursor: pointer;

    &:hover:not(:disabled) {
      background: #000;
    }

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  }

  button.secondary {
    margin-top: 0.5rem;
    padding: 0.4rem 0.8rem;
    border: 1px solid #666;
    background: #fff;
    border-radius: 4px;
    cursor: pointer;
  }

  .error {
    color: #c00;
    margin-top: 0.75rem;
  }

  .result {
    margin-top: 1.5rem;
    padding: 1rem;
    background: #f8f8f8;
    border-radius: 6px;
    border: 1px solid #ddd;

    h3 {
      margin-top: 0;
      color: #060;
    }

    dl {
      margin: 0.5rem 0;

      dt {
        font-weight: 600;
        font-size: 0.85rem;
        color: #555;
        margin-top: 0.5rem;
      }

      dd {
        margin: 0;
        word-break: break-all;

        &.mono {
          font-family: ui-monospace, monospace;
          font-size: 0.8rem;
          background: #eee;
          padding: 0.25rem 0.4rem;
          border-radius: 3px;
        }
      }
    }

    a {
      color: #06c;
    }

    .note {
      margin-top: 1.5rem;
      padding: 1rem;
      background: #fff8e0;
      border: 1px solid #e0c060;
      border-radius: 4px;

      h4 {
        margin-top: 0;
      }

      strong {
        color: #c00;
      }
    }

    .copied {
      color: #060;
      font-weight: 600;
      margin: 0.25rem 0 0;
    }
  }
}
</style>
