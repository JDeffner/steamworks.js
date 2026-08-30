# Agent guidance

All agent and contributor guidance for this repository lives in [CLAUDE.md](./CLAUDE.md). Despite the filename, everything in it is tool-agnostic — it applies to any coding agent or human working here.

Read it before changing code. The short version, if you read nothing else:

- This is a drop-in fork of `ceifa/steamworks.js`; never break an existing export's name, signature, or behavior. Additive changes only.
- `client.d.ts` and `index.d.ts` are generated (by the napi build and `npm run types`) — edit Rust or JSDoc, never the `.d.ts` files.
- Dev loop: `npm ci`, then `npm run build:debug`. Verify with `cargo fmt --all --check`, `cargo clippy`, and the smoke test in `test/typescript` (`npm i && npm run compile`).
- Steam ids and handles are u64: `BigInt` across the JS boundary, never `number`.
- `wiki/` is the source of truth for the GitHub wiki (synced by a workflow) — update the relevant page when you change public API; never edit the GitHub wiki directly.
- Never publish, tag, or open a PR unless explicitly asked.
