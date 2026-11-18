# Ranged Pumps — a Horizon port

**Ranged Pumps** is a Minecraft mod. Ranged Pumps is a simple mod that adds a pump that pumps liquids in a range. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [refinedmods/rangedpumps](https://github.com/refinedmods/rangedpumps) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Ranged Pumps ([source](https://github.com/refinedmods/rangedpumps)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `rangedpumps.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- **Scan.** `range` starts at `-1` (the single column directly **below** the
- **Drain.** Only `minecraft:water`/`minecraft:lava` with `level=0` (a still
- **Tank.** One **32,000 mB** tank, one fluid at a time, declared through
- **Output.** At the end of each of the pump's ticks, the neighbouring
- **Replacement.** The drained source becomes air and then `minecraft:stone`
- **Readout.** Right click sends the state (scanning position/range, full,
- **Range setting.** The ring radius is per block, saved as `configuredRange`

## Not yet ported

- **No energy (FE) gating.** The SDK has energy capabilities
- **No redstone pause.** A part could read `has_neighbor_signal`, but the
- **Range is per block** (`configuredRange`, default 64, `/rangedpumps range`)
- **No config file** for `speed`, tank capacity or the replacement block; the
- **No custom renderer**; the 1.3.0 cube model and textures are carried.
- **No pickaxe tag** — the 1.3.0 jar omits it; `requires_tool` is false.
- **No in-game re-proof at this build** (trap 10): the tank, the drain, the
- **The readout is `use_without_item`** (empty hand). That is upstream's own

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `rangedpumps.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Ranged Pumps was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/refinedmods/rangedpumps

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run rangedpumps` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/rangedpumps/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
