# OathPad Architecture

## 1. Product shape

OathPad is a Solana launchpad where creators lock real assets behind public milestone commitments ("Oaths"). Normal launch flow is launch → raise → promise → hope. OathPad's is promise → lock value → launch → deliver → unlock. The frontend and database can both disappear; the vault must still behave according to the original Oath, because Solana — not Postgres — is authority over whether funds move.

## 2. Two independent layers

```
                    OATHPAD
                       |
         +-------------+-------------+
         |                           |
    LAUNCH LAYER                OATH LAYER
         |                           |
   Meteora DBC                 Oath Core
   (integrated via              (our Anchor
    MIT-licensed SDK,            program)
    never forked)
         |                           |
 bonding curve                  asset vault
 buy / sell                     milestones
 graduation                     approvals
 DAMM v2                        consequences
```

Oath Core never depends on Supabase to decide whether funds move. Postgres is a cache/index of chain state, rebuildable at any time by replaying chain history.

## 3. Monorepo layout

```
oathpad/
├── apps/
│   ├── web/            # Next.js frontend, built from the fun-launch extraction
│   └── indexer/         # persistent worker mirroring chain state into Supabase
├── programs/
│   └── oath-core/       # Anchor program, Anchor 0.31 (matches Meteora DBC/DAMM v2/DLMM pin)
├── packages/
│   ├── oath-sdk/        # typed client for our own Oath Core program
│   ├── launch-adapter/  # LaunchEngine interface + engine implementations
│   ├── meteora-adapter/ # MeteoraLaunchEngine, wraps @meteora-ag/dynamic-bonding-curve-sdk
│   ├── db/               # Supabase schema + typed queries
│   ├── shared/           # cross-package types, constants
│   └── ui/                # OathPad design system components
├── docs/
├── Anchor.toml
├── pnpm-workspace.yaml
└── CLAUDE.md
```

Do not vendor Meteora's repositories wholesale. Extract only what `docs/REUSE-LEDGER.md` says to extract, with attribution.

## 4. Launch engine adapter (corrected against verified SDK API)

The original spec's `LaunchEngine` interface (`createLaunch`, `quoteBuy`, `buildBuy`, `getCurveProgress`, `getGraduationState`) does not match the real SDK — see `docs/UPSTREAM-RESEARCH.md` §2. The interface below uses confirmed method names from `@meteora-ag/dynamic-bonding-curve-sdk` and must be re-verified against that package's `CHANGELOG.md` whenever the pinned version bumps, since it has shipped breaking changes before (e.g. v1.5.12 rejecting `BaseFeeMode.RateLimiter`).

```typescript
// packages/launch-adapter/src/types.ts (B0 implementation, verified against SDK 1.5.13 dist/index.d.ts)
interface LaunchEngine {
  readonly name: string
  createConfigAndPool(params: CreateConfigAndPoolParams): Promise<CreateConfigAndPoolResult>
    // The real SDK returns ONE Transaction holding create_config + initialize_pool, so the
    // result is { transactions: Transaction[], requiredSigners: PublicKey[] (config + baseMint
    // keypairs generated client-side), poolAddress, configAddress, baseMint, quoteMint }.
    // B0 note: the original sketch of { configTx, poolTx } here was wrong.
  getPool(poolAddress: PublicKey): Promise<PoolState | null>
  getPoolByBaseMint(baseMint: PublicKey): Promise<PoolState | null>
  getPoolsByCreator(creator: PublicKey): Promise<PoolState[]>
  getPoolConfig(configAddress: PublicKey): Promise<PoolConfig | null>
  swapQuote(params: SwapQuoteParams): Promise<SwapQuote>        // wraps swapQuote() (exact-in)
  buildSwap(params: SwapParams): Promise<Transaction>           // wraps swap()
  getCurveProgress(poolAddress: PublicKey): Promise<CurveProgress>
    // wraps getPoolQuoteTokenCurveProgress() + getPoolBaseTokenCurveProgress()
  getMigrationState(poolAddress: PublicKey): Promise<MigrationState>
    // derived from getPool() + getPoolConfig(): isMigrated, migrationProgress, quoteReserve vs threshold
}
```

Amounts cross this interface as `bigint` in raw units; only `packages/meteora-adapter` touches bn.js.

`MeteoraLaunchEngine implements LaunchEngine` lives in `packages/meteora-adapter` and is the only place that imports `@meteora-ag/dynamic-bonding-curve-sdk` directly. No React component or API route imports the Meteora SDK directly — everything goes through the `LaunchEngine` interface, so a future Raydium adapter (blocked on GPL-3.0 licensing today, see reuse ledger row 6) can be dropped in without touching UI code.

```
                OATH CORE
                    |
             LaunchBinding
                    |
      +-------------+--------------+
      |             |              |
   Meteora       Raydium        Future
   Adapter        Adapter        Adapter
  (V1, live)     (blocked:       (V2+)
                  GPL-3.0)
```

## 5. Oath Core program accounts

`ProtocolConfig` — authority, treasury, pause flag. `paused` is read by `create_oath`/
`create_launch_binding` but has no write path in the current instruction set (hardcoded `false`
forever); `treasury` is stored but read by no instruction. Neither is a functioning capability
today -- see `docs/security/AUTHORITY-INVENTORY.md` and `docs/THREAT-MODEL.md` §4 for the exact
audit and the corrected admin-power boundary (also: the program's upgrade authority is a
separate surface from `ProtocolConfig.authority`, not the same key). Authority may **not**
withdraw user Oath assets, mark a milestone successful, or change a deadline/reviewer
set/allocation/failure destination after activation.

`LaunchBinding` — one per OathPad launch: creator, token mint, DBC pool address, DBC config address, oath count, status (`DRAFT → OATHS_FUNDED → LIVE → GRADUATED → CLOSED`). Once `LIVE`, economic Oath configuration is immutable.

`Oath` — up to 3 per launch in V1: launch, creator, asset mint, vault, committed amount, milestone count, reviewer set, failure action, status.

`Milestone` — oath, index, allocation amount, deadline, evidence hash (SHA-256 of canonical evidence JSON, stored off-chain in R2/Supabase — only the hash is on-chain), approvals count, required approvals, status. States: `PENDING → ACTIVE → EVIDENCE_SUBMITTED → APPROVED → CLAIMABLE → CLAIMED`, or `EXPIRED/FAILED → CONSEQUENCE_EXECUTED`. No ambiguous state.

`ReviewerSet` — reviewers (Vec<Pubkey>), threshold. Creator cannot be a reviewer, no duplicate reviewers, threshold ≤ reviewer count, immutable after activation.

## 6. V1 scope

- Supported Oath assets: creator token allocation, SOL bond, SPL token bond, USDC bond.
- Failure actions: `Burn`, `FixedRecipient { recipient: Pubkey }`. `HOLDER_DISTRIBUTION` (Merkle-based, patterned on `solana-foundation/rewards`) is V1.1.
- Fee streaming (`FeeStream`) exists as an enum value, feature-flagged off. Phase 2 introduces `programs/oath-fee-router`.
- DBC config: SOL quote token, ordinary SPL Token (`tokenType: 0`, not Token-2022), DAMM v2 migration (`migrationOption: 1`), immutable mint authority post-launch.

## 7. Activation safety

Never display a launch as "Oath Backed" until the vault provably holds the committed assets on-chain. Sequence: prepare metadata → create token/pool → create Oath accounts → fund Oath vault → verify vault balance on-chain (read back, don't trust client-reported success) → activate Oath → mark launch publicly `LIVE`. If this can't fit in one transaction, use a multi-transaction setup flow and keep the launch in `SETUP INCOMPLETE` with no badge until every step is chain-confirmed.

## 8. Database role

Supabase mirrors chain state; it is never the authority for a release decision. `wallet_profiles, projects, launches, oaths, milestones, reviewer_sets, reviewers, evidence_packages, approvals, chain_events, dbc_pools, trades, graduations, sync_cursors, indexer_jobs, audit_log`. Every mirrored chain entity stores `chain_address, slot, tx_signature, block_time, last_finalized_slot`. The indexer runs both a realtime subscription and a periodic reconciliation scan, so a websocket disconnect cannot permanently lose state; every event is idempotent via `signature + instruction index` as a unique constraint.
