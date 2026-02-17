# Strangemood Revival

**Decentralized Game Store on Solana** — A protocol for buying, selling, and governing digital games with on-chain escrow, license NFTs, and DAO-based marketplace governance.

Originally built by [Strangemood Labs](https://github.com/strangemoodfoundation) in 2022 using Anchor 0.20 / Solana 1.9. This revival ports the protocol to modern Solana (Anchor 0.30 / Solana 2.2) with all 20 instructions working and tested.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Charter (DAO)                      │
│  Governance token mint, contribution rates, URI     │
│  Payment: 10% to treasury | Votes: 10% to treasury │
├─────────────────┬───────────────────────────────────┤
│ Charter Treasury│  Per-token treasury (USDC, wSOL)  │
│                 │  Expansion scalar for vote minting │
├─────────────────┴───────────────────────────────────┤
│                    Listings                          │
│  Game metadata, price, refund policy, availability  │
│  Each listing gets its own token mint (license NFT) │
├─────────────────────────────────────────────────────┤
│                   Purchases                          │
│  Escrow → Receipt → Cash (with revenue splits)      │
│  Refundable: mint+freeze license, cancel returns $  │
│  Non-refundable: immediate cashout                  │
└─────────────────────────────────────────────────────┘
```

## Instructions (20 total)

| Category | Instructions |
|----------|-------------|
| **Charter** | `init_charter`, `set_charter_expansion_rate`, `set_charter_contribution_rate`, `set_charter_authority`, `set_charter_vote_deposit` |
| **Treasury** | `init_charter_treasury`, `set_charter_treasury_expansion_scalar`, `set_charter_treasury_deposit` |
| **Listing** | `init_listing`, `set_listing_price`, `set_listing_uri`, `set_listing_availability`, `set_listing_deposits`, `set_listing_authority`, `set_listing_charter` |
| **Commerce** | `purchase`, `cash`, `cancel`, `consume`, `set_receipt_cashable` |

## How It Works

1. **Charter** — A marketplace DAO defines contribution rates (% of each sale goes to the treasury) and a governance token
2. **Treasury** — Accepts specific payment tokens (USDC, wSOL) and controls vote minting expansion
3. **Listing** — Game devs list games with price, metadata URI, and refund policy
4. **Purchase** — Buyers pay into escrow, receive a frozen license NFT as proof of purchase
5. **Cash** — The cashier (game dev) finalizes the sale: escrow splits between dev (90%) and treasury (10%), governance tokens are minted
6. **Cancel** — For refundable purchases: burns the license NFT, returns escrowed funds

## Quick Start

```bash
# Install dependencies
yarn install

# Build the program
anchor build

# Run tests (7/7 passing)
anchor test
```

## Test Suite

```
  strangemood-revival
    ✔ Initializes a Charter (marketplace DAO)
    ✔ Initializes a Charter Treasury
    ✔ Lists a game on the marketplace
    ✔ Purchases a game
    ✔ Updates listing price
    ✔ Updates listing availability
    ✔ Updates charter contribution rates

  7 passing (1s)
```

## What Changed from Original

| Original (2022) | Revival (2026) |
|-----------------|----------------|
| Anchor 0.20.1 | Anchor 0.30.1 |
| Solana 1.9 | Solana 2.2 |
| `ProgramResult` | `Result<()>` |
| `#[error]` | `#[error_code]` |
| Bare `AccountInfo` | `/// CHECK:` annotations |
| No IDL build | Full IDL with `idl-build` feature |

## Program ID

```
Av997JVrRJXPTrjbMnkPmMzbgwWWsHxuGRjecqVWUMFi
```

## License

Apache-2.0 (same as original Strangemood)

---

*Built for the [Solana Graveyard Hackathon](https://www.colosseum.org) — Gaming Track*
