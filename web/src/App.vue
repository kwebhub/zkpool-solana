<template lang="pug">
.container
  header.header
    h1 zkpool-solana
    .wallet
      .connected(v-if="wallet.connected")
        span.name {{ wallet.walletName }}
        code.addr {{ shortAddr }}
        button(@click="wallet.disconnect()") Disconnect
      .disconnected(v-else)
        button(
          v-for="w in wallet.available"
          :key="w.name"
          :disabled="wallet.connecting"
          @click="wallet.connect(w)"
        ) Connect {{ w.name }}
        p.muted(v-if="wallet.available.length === 0") No Solana wallet detected. Install Phantom or Solflare.

  main
    p(v-if="!wallet.connected") Connect a wallet to deposit.
    DepositForm(v-else)

  p.error(v-if="wallet.error") {{ wallet.error }}
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useWalletStore } from "./stores/wallet";
import DepositForm from "./components/DepositForm.vue";

const wallet = useWalletStore();

const shortAddr = computed(() => {
  const a = wallet.addr;
  if (!a) return "";
  const s = a.toString();
  return `${s.slice(0, 4)}…${s.slice(-4)}`;
});
</script>

<style lang="scss" scoped>
body {
  font-family: system-ui, sans-serif;
  margin: 2rem;
}

.container {
  max-width: 800px;
  margin: 0 auto;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid #ddd;
}

.wallet {
  display: flex;
  gap: 0.5rem;
  align-items: center;

  .name {
    font-weight: 600;
    margin-right: 0.25rem;
  }

  .addr {
    font-family: ui-monospace, monospace;
    font-size: 0.85rem;
    background: #f4f4f4;
    padding: 0.15rem 0.4rem;
    border-radius: 4px;
    margin-right: 0.5rem;
  }

  button {
    padding: 0.4rem 0.8rem;
    border: 1px solid #333;
    background: #fff;
    border-radius: 4px;
    cursor: pointer;

    &:hover:not(:disabled) {
      background: #333;
      color: #fff;
    }

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  }

  .muted {
    color: #888;
    font-size: 0.9rem;
  }
}

main {
  margin-top: 2rem;
}

.error {
  color: #c00;
}
</style>
