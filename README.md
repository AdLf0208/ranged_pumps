# Ranged Pumps for Horizon

A native Horizon port of **Ranged Pumps 1.3.0** (`refinedmods/rangedpumps`,
**MIT**): a pump block that walks a scan through still source blocks in a
configured radius, drains each one into a 32-bucket internal tank, and pushes
the tank into adjacent fluid handlers.

**LOCAL-ONLY** until the maintainer approves publishing (ARCHITECTURE.md §8).

- Format: the **SDK layout** (mod API 0.2, launcher ≥ 0.2.70): one flat
  `rangedpumps.hmod` — `horizon-mod.toml` + `Cargo.toml` + `src/` + `assets/` +
  `data/`.
- SDK: nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`**, mixins
  **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, game 26.3.
- Class/tier: `content` / `machines`; approach per
  `horizon/docs/modpacks.md` (block entity + fluid capability + tick, not a
  data-only carry).
- Behaviour, gaps and verification: [notes.md](notes.md) — including
  `## Migration to the SDK format`; upstream notes: [FINDINGS.md](FINDINGS.md);
  licence: [LICENSE-NOTICE.md](LICENSE-NOTICE.md).

## Layout

```text
horizon-mod.toml                   The only manifest: id, version, api 0.2, [native] pins
Cargo.toml                         The crate the launcher compiles (horizon-mod-api from .horizon/sdk/)
src/lib.rs                         The one #[mod_api::horizon_mod] impl Hooks + the pump Parts
src/runtime.rs                     Pure scan/tank logic and tests (this campaign's rule)
assets/rangedpumps/                blockstate, block/item models, textures, lang
data/rangedpumps/                  block definition, creative tab, recipe, loot table
rangedpumps.hmod                   The built package (port root, never target/)
```

Retired by the 2026-11-17 mod-API merge (#16941) and no longer present:
`mixins/rangedpumps/`, `horizon-mixin.toml`, `horizon-native.json`,
`compatibility.json`, `rust-toolchain.toml`, `content/` and the embedded-content
injection. Their last state is kept under
`ports/_shared/evidence/rangedpumps/retired-hnmod/`.

## Build and verify

```sh
# fetch the pinned SDK into .horizon/sdk/ (once per clone; any `cargo horizon-mod` command does it)
cd ports/rangedpumps && cargo horizon-mod check

ports/tools/build-slot.sh run --job sdk-rp --port ports/rangedpumps --step check -- \
  bash -c 'cd ports/rangedpumps && cargo test && cargo clippy --all-targets -- -D warnings'
ports/tools/build-slot.sh run --job sdk-rp --port ports/rangedpumps --step build -- \
  bash -c 'cd ports/rangedpumps && cargo horizon-mod build && cp target/horizon-mod/rangedpumps.hmod .'
cargo horizon-mod check rangedpumps.hmod && cargo horizon-mod fits rangedpumps.hmod
```

In game: right-click a pump with an empty hand for its state and tank,
`/rangedpumps info <x> <y> <z>` for the same, and
`/rangedpumps range <x> <y> <z> <range>` (op 2) to set its ring radius
(default 64).
