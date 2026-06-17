# meditation_minutes

## Project Title
meditation_minutes

## Project Description
Mental-health and mindfulness practice is hard to keep honest: paper logs get lost, mobile apps sit on a single device, and streak counts disappear the moment a user switches phones. `meditation_minutes` is a small Soroban smart contract that lets a user log each meditation session (minutes, technique, date) directly to the Stellar blockchain, so the practice history becomes a permanent, portable, verifiable record owned by the user's wallet — not by a corporation. Every session is signed by the user via `require_auth`, accumulates a lifetime-minute total, and unlocks on-chain achievement badges at well-known milestones.

## Project Vision
Our long-term goal is to make "I meditated" something the world can cryptographically verify, the same way a diploma or a marathon finish time can be verified. We imagine a future in which mental-health practitioners, employers running wellness programs, insurers offering mindfulness incentives, and online meditation teachers can all read a user's on-chain practice history (with the user's consent) and reward consistency — without any centralized app storing the data. `meditation_minutes` is the first, deliberately tiny, step in that direction: a contract that proves the concept of self-sovereign mindfulness logging on Stellar.

## Key Features
- **Self-sovereign session log** — every entry is signed by the user's own Stellar address via `require_auth`; no admin or third party can write to another user's history.
- **Per-session metadata** — each call to `log_session` records the duration in minutes, a `Symbol` technique tag (e.g. `mindfulness`, `breath`, `loving_k`, `mantra`, `yoga`), and the ledger timestamp, so the history is rich enough to be useful and not just a counter.
- **Lifetime minutes counter** — `lifetime_minutes(user)` returns the running total for any address in a single read, with no off-chain aggregation needed.
- **Milestone achievement badges** — once a user crosses a chosen threshold (e.g. 100, 500, 1 000, 10 000 lifetime minutes), they can self-claim a permanent badge via `claim_badge`; `claim_badge` is idempotent, so a duplicate claim is a safe no-op that returns `false`.
- **Public badge verification** — `has_badge(user, threshold)` lets any third party (a teacher, an employer wellness portal, a friend) read on-chain whether a user has earned a given milestone, with no authentication required for the read.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** healthcare dApp — see `contracts/meditation_minutes/src/lib.rs` for the full meditation_minutes business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** CBWZ55A6EKI7LMWOETMPIBUSPCCMEKP63LREEGZ2P3VM4JFTLHS3WIES
- **Explorer template:** https://stellar.expert/explorer/testnet/tx/5a59347ab340a232fb41d0d3ded6384e2e82c021ab5ae7d293a6a826098beb1c
- **Screenshot of deployed contract on Stellar Expert:**
![screenshot](https://ibb.co/LX3X4Rzn)


## Future Scope
- **Per-session history getter** — a `get_session(user, index) -> (u32, Symbol, u64)` view that returns the full record of a specific past session, paginated for users with long histories.
- **Friend / teacher attestation** — let a second user co-sign a session (`log_session_with_witness`) so group meditation, retreats, or guided sessions can be verified by a teacher.
- **Wellness-reward token hook** — emit an event from `claim_badge` that an off-chain indexer (or another contract) can listen for, to optionally mint a soulbound "wellness" token or trigger a USDC micropayment from an employer wellness budget.
- **Streak tracking** — a `current_streak(user) -> u32` view that uses the ledger timestamp to compute consecutive-day streaks, with a separate `streak_badge` claim flow.
- **Time-bounded challenges** — admin-set challenges such as "200 minutes in any 14-day window", with badges awarded automatically by the contract at the end of the window.
- **Native-asset tip jar** — a small opt-in XLM tip flow that lets a user thank a teacher on-chain after a verified session, while keeping the core practice log free of any monetary activity by default.
- **Frontend dApp** — a Freighter-connected web UI that visualises lifetime minutes, session history, and earned badges, built on top of this same contract.

## Project Structure
```text
.
├── contracts
│   └── meditation_minutes
│       ├── src
│       │   └── lib.rs
│       ├── Cargo.toml
│       └── Makefile
├── Cargo.toml
└── README.md
```

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `meditation_minutes` (healthcare)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
