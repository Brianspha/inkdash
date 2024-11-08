<template>
    <div class="game-hud">
        <div v-if="store.gameActive" class="game-header">
            <h1>Ink-Dash</h1>
            <div class="sub-heading">DASH to the END!</div>
        </div>

        <div v-if="store.gameActive" class="stats-bar">
            <div class="level">
                <span>Distance</span>
                <div class="level-value">{{ store.distance }}</div>
            </div>

            <div class="score">
                <div class="level">
                    <span>Score</span>
                    <div class="level-value">{{ store.score }}</div>
                </div>
            </div>
            <div class="score">
                <div class="level">
                    <span>Address</span>
                    <div class="level-value">{{ updateAddress() }}</div>
                </div>
            </div>
        </div>

        <!-- Game Over Menu -->
        <div v-if="!store.gameActive" class="menu-overlay">
            <div class="game-over-menu">
                <h2>Game Over</h2>
                <p class="level-value">Your Score: {{ store.score }}</p>
                <p class="level-value">Your Distance: {{ store.distance }}</p>
                <button @click="restart">Restart</button>
                <button @click="continueGame">Continue</button>
            </div>
        </div>
        <!-- Start Menu -->
        <div v-if="store.showMenu" class="menu-overlay">
            <div class="game-over-menu">
                <h2 class="level-value">Welcome</h2>
                <button class="level-value" @click="play">Play</button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useGameStore } from '@/stores/gamestore'
import { onMounted, inject, ref } from 'vue'
import Onboard from '@subwallet-connect/core';
import injectedModule from '@subwallet-connect/injected-wallets';
import subwalletModule from '@subwallet-connect/subwallet';
import subwalletPolkadotModule from '@subwallet-connect/subwallet-polkadot';
import type { EIP1193Provider, SubstrateProvider } from "@subwallet-connect/common";
import { ethers } from 'ethers';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { ContractPromise } from '@polkadot/api-contract';

const MAINNET_RPC_URL = 'https://mainnet.infura.io/v3/<INFURA_KEY>'
const ws = 'wss://rpc.polkadot.io'
const injected = injectedModule()
const subwalletWallet = subwalletModule()
const subwalletPolkadotWalet = subwalletPolkadotModule()
const progress: any = inject("progress");
const store = useGameStore()
const onboard = Onboard({
    wallets: [injected, subwalletWallet, subwalletPolkadotWalet],
    chains: [
        {
            id: '0x1',
            token: 'ETH',
            label: 'Ethereum Mainnet',
            rpcUrl: MAINNET_RPC_URL
        }
    ],
    chainsPolkadot: [
        {
            id: '0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3',
            namespace: 'substrate',
            token: 'DOT',
            label: 'Polkadot',
            rpcUrl: `polkadot.api.subscan.io`,
            decimal: 10
        }
    ]
})
const provider = ref<WsProvider | null>(null)

// Create the API and wait until ready
const api = ref<ApiPromise | null>(null)
//const contract = new ContractPromise(api, require("../../../contracts/"), "");

const submitScore = () => {
    console.log("submitting score", store.score)
}
const play = () => {
    store.showMenu = false
    connectWallet()
}
const continueGame = () => {
    console.log("continue")
}
const restart = () => {
    store.restart = true
    console.log("restart", store.restart)
}
const connectWallet = async () => {
    // Initialise the provider to connect to the local node

    progress.start();
    provider.value = new WsProvider('ws://127.0.0.1:62160');
    api.value = await ApiPromise.create({ provider: provider.value });
    // Retrieve the chain & node information via rpc calls
    const [chain, nodeName, nodeVersion] = await Promise.all([
        api.value.rpc.system.chain(),
        api.value.rpc.system.name(),
        api.value.rpc.system.version()
    ]);

    console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`);
    const wallets = await onboard.connectWallet()
    const wallet = wallets[0]
    console.log(wallet)
    if (wallet?.type === 'substrate') {
        console.log("connected to polkadot")
        store.address = wallet.accounts[0].address
    }
    store.canDash = true
    progress.finish();
}
const updateAddress = (): string => {
    return store.address.length > 0 ? store.address.substring(0, 3) + "..." + store.address.substring(store.address.length - 3, store.address.length) : "Connect Wallet"
}
onMounted(() => {
    console.log("mounted: ", store.showMenu)
    store.gameActive = false
    store.canDash = false
})
</script>
<style scoped>
.main-container {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
}

.game-container {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
}

.game-hud {
    position: fixed;
    top: 10%;
    right: 35%;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 50;
}

.game-header {
    text-align: center;
    margin-bottom: 1rem;
    padding-top: 1rem;
}

.game-header h1 {
    font-family: 'Georgia', serif;
    font-weight: 400;
    font-size: 2.5rem;
    letter-spacing: 0.1em;
    color: black;
    margin: 0;
}

.sub-heading {
    font-size: 1rem;
    letter-spacing: 0.2em;
    color: black;
}

.stats-bar {
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: var(--bg-light, #f8e5d1);
    padding: 1rem;
    border-radius: 30px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    gap: 2rem;
    margin: 0 auto;
    width: fit-content;
    pointer-events: all;
}

.menu-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: rgba(0, 0, 0, 0.5);
    pointer-events: all;
}

.game-over-menu {
    background: white;
    padding: 2rem;
    border-radius: 20px;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
    width: 80%;
    max-width: 400px;
    text-align: center;
}

.game-over-menu h2 {
    font-size: 2rem;
    margin: 0 0 1rem;
    color: var(--primary-color, #c09b79);
}

.game-over-menu button {
    background-color: var(--text-dark, #6e5339);
    color: white;
    border: none;
    border-radius: 10px;
    padding: 10px 20px;
    font-size: 1rem;
    cursor: pointer;
    transition: background-color 0.3s ease;
    margin: 0.5rem;
}

.game-over-menu button:hover {
    background-color: var(--secondary-color, #b17f6a);
}

.level {
    color: #000;
}

.level-value {
    color: #000;
}

/* Mobile layout */
@media (max-width: 768px) {
    .game-header h1 {
        font-size: 1.5rem;
    }

    .sub-heading {
        font-size: 0.8rem;
    }

    .stats-bar {
        max-width: 90%;
        padding: 0.5rem;
    }

    .game-over-menu {
        width: 90%;
        padding: 1.5rem;
    }
}
</style>