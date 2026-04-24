# 🏨 Hostel Key Deposit — Soroban Smart Contract

> Trustless hostel key deposits on Stellar.  
> Guests lock a deposit on check-in. If the room is left clean, the full amount is automatically refunded.

## Project Description

Hostel Key Deposit is a Soroban smart contract on the Stellar network that replaces cash-deposit workflows with on-chain escrow. The deposit is held by the contract itself — not by the hostel — and released trustlessly based on a room inspection at checkout.

## What It Does

1. **Guest checks in** → calls `checkin(room_id, amount)`, locking tokens into escrow
2. **Room is inspected at checkout:**
   - Clean → Admin calls `checkout_clean` → deposit **returned in full** to guest
   - Damaged → Admin calls `checkout_forfeit` → deposit **sent to hostel wallet**
3. Every action emits an on-chain event for auditability and front-end updates

## Features

- 🔒 **Non-custodial escrow** — funds sit in the contract, not the hostel's wallet
- ✅ **Instant refund on clean checkout** — one admin transaction, guest gets paid back immediately
- 🚫 **Forfeit on damage** — admin redirects deposit to hostel wallet if room is dirty/damaged
- 🏷️ **Room-scoped deposits** — keyed to `(room_id, guest)`, supports unlimited rooms simultaneously
- 🔐 **Stellar auth model** — guest signs check-in, admin signs checkout, no impersonation possible
- 📡 **On-chain events** — `CHECKIN`, `REFUNDED`, `FORFEIT` events published for every state change
- 🪙 **Any SAC token** — works with USDC, native XLM, or any Stellar Asset Contract token
- 🧪 **Full test suite** — covers happy path, forfeit path, and double-refund guard

## Quick Start

\```bash
rustup target add wasm32-unknown-unknown
cargo test
cargo build --target wasm32-unknown-unknown --release
\```

## Deploy to Testnet

\```bash
stellar keys generate --global admin --network testnet --fund

stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/hostel_key_deposit.wasm \
  --source admin \
  --network testnet

stellar contract invoke \
  --id <CONTRACT_ID> --source admin --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --token <TOKEN_ADDRESS>
\```

## License

MIT
Wallet address:- GBLAH7DB57GFI3XVKWLBOCW5RM5AVYMGMOKOZB7YYAGF6XRFRVINWGB7
Contract address:- CBAQQHBSXJYVSPQEDUXKFW2B4X2UXFFXH76PO64VMZHANSXI2NX3QR6D
https://stellar.expert/explorer/testnet/contract/CBAQQHBSXJYVSPQEDUXKFW2B4X2UXFFXH76PO64VMZHANSXI2NX3QR6D
<img width="1600" height="861" alt="image" src="https://github.com/user-attachments/assets/e90d3e8c-1a5f-45cd-be14-e28e0fc0647c" />

