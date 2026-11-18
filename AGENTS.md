# Working on the Ranged Pumps mod

A Horizon native mod in the **SDK layout** (mod API 0.2, launcher ≥ 0.2.70,
Horizon's mod-API merge #16941). Read
`../_shared/AGENT_MOD_STRUCTURE.md` §0 first: it supersedes the older
`.hnmod` sections and describes exactly this tree.

> **Build slots (v2):** every build/test/e2e goes through the campaign runner
> (`../tools/build-slot.sh` from the campaign root). It **blocks in the kernel**
> until a slot is free — never poll, never background a build. A timeout exits
> **75** and is a scheduling signal, not a verdict on the code. Read
> `../_shared/build-slots-usage.md` before running anything.

- `src/runtime.rs` is the pure logic and its tests (`cargo test` alone). Keep it
  free of `mod_api` types: the scan, the still-source rule, the tank and the
  state line live there.
- `src/lib.rs` is the glue: the one `impl Hooks` (registration, commands) and
  the pump's `BlockPart`/`BlockEntityPart`. Never guess an API name —
  `cargo horizon-mod hooks <text>` searches the pinned surface offline, and
  `.horizon/sdk/api/` is the reference.
- `horizon-mod.toml` is the only identity and the only manifest; the pins in
  `[native]` must match the SDK in `.horizon/sdk/sdk.json`.
- The SDK is fetched per clone into `.horizon/sdk/` and never packed.
- Validate with `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, `cargo test`, then `cargo horizon-mod check`, `cargo horizon-mod
  build` and `cargo horizon-mod check <file>.hmod` / `fits <file>.hmod`.

Keep `rangedpumps.hmod` in the port root (never in `target/`), with its size and
SHA-256 recorded in `notes.md`.
