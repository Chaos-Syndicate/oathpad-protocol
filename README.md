# OathPad Protocol

OathPad is a Solana launchpad where creators lock real assets behind public milestone commitments ("Oaths"). Normal launch flow is launch → raise → promise → hope. OathPad's is promise → lock value → launch → deliver → unlock.

Live product: [oathpad.app](https://oathpad.app)

## What's in this repository

This is a curated, public mirror of the on-chain program source from OathPad's private monorepo — not the full repository. It contains:

- `programs/oath-core/` — the Anchor program that holds Oath vaults, milestones, and consequences on-chain. Solana, not a database, is authority over whether funds move.
- `idl/oath_core.json` — the program's Anchor IDL.
- `docs/ARCHITECTURE.md` — product and account-model overview.

Frontend code, SDKs, internal build/process reports, and security-review material are not published here.

## Status

OathPad is currently deployed and under active testing on Solana **devnet** only. No mainnet deployment has occurred.

## License

MIT — see [LICENSE](./LICENSE).
