# Ranged Pumps — port notes

Ported from **Ranged Pumps 1.3.0** (`refinedmods/rangedpumps`, **MIT**),
Minecraft Java 1.21 NeoForge, ATM10 jar `rangedpumps-1.3.0.jar` (SHA-1
`b2430e501d0c03d0464ec351e3e839cb2a56a06a`, CurseForge file `5508029`).

**LOCAL-ONLY** until the maintainer approves publishing (ARCHITECTURE.md §8).
MIT allows derivatives; the approval is still required.

**Class + tier:** `content` / **`machines`** (`ports/ledger.csv`;
provenance `ports/_shared/source-map/atm10-sources.csv` row `rangedpumps`).
Approach per Horizon's `docs/modpacks.md`: a block entity with a fluid
capability, a server tick, a content block + item + models/textures, and a
right-click/readout — **not** a data-only carry. Licence **MIT → rank 0**; it
is a core mechanics mod with no required parent (`HORIZON_PUMP` depends on
nothing in the pack).

**Status:** slice 1 implemented and verified in-game — the pump block, its
scan, still-source drain, the 32-bucket tank exposed through
`block_entity_fluids`, output to adjacent fluid handlers, source replacement,
the readout and `/rangedpumps info|range`.

**Format (2026-11-18):** **migrated to the SDK layout** — one flat
`rangedpumps.hmod` (`horizon-mod.toml` + `Cargo.toml` + `src/` + `assets/` +
`data/`), mod API **0.2**, SDK nightly
`0b14f8acfd97fa2c750e42795249dcc981178ca4`, mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, version **0.2.0**. The retired
`.hnmod` (last built 19,338 B `0c622b40…`) and its tree are gone; the mapping,
the traps and the current gaps are in
[`## Migration to the SDK format`](#migration-to-the-sdk-format) below. The
sections after it describe the retired `.hnmod` slices and are history.

## Behaviour (slice 1)

The pure arithmetic is `runtime.rs`; the world work is `src/lib.rs` (the retired
`.hnmod` kept it in `mixins/rangedpumps/support.rs`).

- **Scan.** `range` starts at `-1` (the single column directly **below** the
  pump); each later range is the **perimeter** of a `(2r+3)²` square, i.e. the
  8, 16, 24… block ring at horizontal distance `r+1`. Within a column the pump
  goes down one block per **8 ticks** (upstream `speed`) to the dimension's
  minimum Y, then starts the next column. The interior of a ring is not
  scanned (upstream behaviour).
- **Drain.** Only `minecraft:water`/`minecraft:lava` with `level=0` (a still
  source) is drained; flowing water (`level` 1–7) and waterlogged non-liquid
  blocks are skipped. A drain moves exactly **1,000 mB** and only if a whole
  bucket fits; otherwise the source stays and the scan moves on.
- **Tank.** One **32,000 mB** tank, one fluid at a time, declared through
  `Registrar::block_fluids` (the retired slice declared it through
  `native_block_entity_fluids`, API 54) as `inputs = []`, `outputs = [0]`
  (upstream's tank refuses insertion and allows extraction). Contents live in
  the block entity's saved `fluids`.
- **Output.** At the end of each of the pump's ticks, the neighbouring
  `horizon:fluids` capabilities are filled with `insert_any`
  (`tank_amount / neighbour_count` mB each; the retired slice did it after the
  level tick with `native_move_fluids`, API 55); a vanilla cauldron takes the
  bucket.
- **Replacement.** The drained source becomes air and then `minecraft:stone`
  (upstream `replaceLiquidWithBlock` default).
- **Readout.** Right click sends the state (scanning position/range, full,
  done) and the tank line; `/rangedpumps info <x> <y> <z>` prints the same.
- **Range setting.** The ring radius is per block, saved as `configuredRange`
  (default 64, clamp 0..=1024), set by `/rangedpumps range <x> <y> <z> <range>`
  (op 2) which restarts the scan.

## Hooks (retired `.hnmod` plan, kept for history)

The live mapping to the SDK surface is in
[`## Migration to the SDK format`](#migration-to-the-sdk-format) above.

| Hook (API) | Helper | What it does |
|---|---|---|
| `block_entity` (15) | `block_entity.rs` | Create the pump's block entity. |
| `block_entity_ticks` (15) | `ticks.rs` | The pump ticks on the server. |
| `block_entity_fluids` (54) | `fluids.rs` | One 32,000 mB tank, all faces, output only. |
| `block_entity_tick` (17) | `tick.rs` | Advance the scan and drain a source every 8 ticks. |
| `right_click_block` (21) | `right_click.rs` | Report state + tank, claim the click. |
| `level_tick_post` (23) | `level_tick_post.rs` | Push the tank into adjacent fluid handlers. |
| `register_commands` (8) | `register.rs` | `/rangedpumps info|range`. |
| `perform_command` (8) | `perform.rs` | Command bodies. |

## Commands

- `/rangedpumps info <x> <y> <z>` — state, scan position, range, tank fluid and
  amount (the e2e reads this).
- `/rangedpumps range <x> <y> <z> <range>` — op 2; sets the per-pump ring
  radius and restarts the scan.

## Migration to the SDK format

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70) retired `.hnmod`; the
registry closed PR **#607** with *"republish this as a native `.hmod` with the
SDK from the next Nightly"*. This slice does exactly that: **`runtime.rs` and
its tests are byte-identical**, only the glue, the manifest and the packaging
are new.

### Files created and deleted

| Action | Path |
|---|---|
| created | `horizon-mod.toml` (one manifest, `api = "0.2"`, `[native]` pins) |
| created | `Cargo.toml` (only `horizon-mod-api` from `.horizon/sdk/`) |
| created | `src/lib.rs` — the one `impl Hooks` and the pump's two Parts |
| created | `LICENSE-NOTICE.md` (packed beside `LICENSE`) |
| moved | `content/assets/rangedpumps/` → `assets/rangedpumps/`; `content/data/rangedpumps/` → `data/rangedpumps/` (byte-identical, same files) |
| carried | `mixins/rangedpumps/runtime.rs` → `src/runtime.rs`, **byte-identical**: both `af81428e7a5d01002e974cc42d5f346f7a41828cc549aaf63450d5b88b556f90` |
| deleted | `mixins/rangedpumps/` (11 files: `horizon-mixin.toml`, `runtime.rs`, `support.rs`, 8 helpers, `data/rangedpumps-content.hmod`), `content/` (its `horizon-mod.toml`, `target/`, built `.hmod`), `rangedpumps.hnmod`, `compatibility.json`, `rust-toolchain.toml`, the old `Cargo.lock`, `tools/` (the mixin-era ladder), `target/` |
| rebuilt | `rangedpumps.hmod` at the port root (the deliverable) |
| rewritten | `README.md`, `AGENTS.md`, `.github/workflows/ci.yml` for this layout |

The retired tree's last state (for reading, not for building) is kept at
`ports/_shared/evidence/rangedpumps/retired-hnmod/`.

### Hook → SDK mapping

| Old `.hnmod` | Old helper | New SDK surface |
|---|---|---|
| `block_entity` (15) | `block_entity.rs` | `Registrar::block_entity_type("pump").blocks(&["rangedpumps:pump"]).build()` — the game creates the entity; no `(id, NBT)` factory |
| `block_entity_ticks` (15) | `ticks.rs` | `BlockEntityPart::block_entity_ticks` → `event.ticks = true` |
| `block_entity_tick` (17) | `tick.rs` | `BlockEntityPart::block_entity_tick` (`event.pos`, `event.entity` = a `ModBlockEntityView`) |
| `block_entity_fluids` (54), `ModFluidTanks` | `fluids.rs` | `Registrar::block_fluids(&BlockKey, ModFluidTanks)` — the game's **default fluid provider** answers `horizon:fluids` and keeps the contents in the entity's `fluids` compound |
| `level_tick_post` (23) + `native_fluid_tanks` + `native_move_fluids` (55) | `level_tick_post.rs` | **no level sweep**: the push runs at the end of the pump's own tick via `LevelRef::capability(host, &FLUIDS, neighbour, Some(face.opposite()))` and `ResourceHandler::insert_any` inside `host.transaction(..)` |
| `right_click_block` (21) | `right_click.rs` | `BlockPart::use_without_item` on the pump block (the block Part is expanded from the block-entity declaration) |
| `register_commands` (8) | `register.rs` | `Hooks::register_commands` + `Command::new` / `Command::op(.., 2)` with the same usage strings |
| `perform_command` (8) | `perform.rs` | `Hooks::run_mod_command` with a parsed `CommandInvocation` (`literals`, `argument("pos") -> BlockPos`, `argument("range") -> Integer`) |
| `content_root` HEAD injection (AE2 pattern, `embedded_content.rs`) | `embedded_content.rs` | **not needed** — one `.hmod` carries content and code; nothing is embedded |
| `game.native_block(..)` / `native_block_property(..)` | — | `LevelRef::block(host, pos)` + `BlockKey::id(host)` / `LevelRef::block_property(host, pos, "level")` |
| `game.native_set_block(dim, pos, state, flags)` | — | `LevelRef::set_block(host, pos, state, BlockUpdateFlags::NEIGHBORS \| CLIENTS)` |
| `game.world.bottom_y()` | — | `LevelRef::height(host)` → `(min_y, height)` |
| `game.world.block_entities` (direct map access) | — | the tick's `event.entity.nbt`; commands read `BlockEntityRef::data(host)` and write `replace_data(host, ..)` |
| `game.native_feedback(Text)` | — | `host.commands().feedback(Text)` (honours silence/track-output/the swapped-in player) and `PlayerRef::tell(host, Text)` for the readout |
| `&NativeServerPlayer` (removed type) | — | `CommandSource` for commands; `event.click.player` (uuid) for the readout |

**Which host/capability replaced the old fluid calls** (the risky area): the
tank declaration moved from the `block_entity_fluids` hook to
`Registrar::block_fluids`, i.e. the **game's default fluid provider** reached
through the `horizon:fluids` `CapabilityKey<dyn ResourceHandler<FluidResource>>`
— the same capability a cauldron answers. `native_fluid_tanks` is
`LevelRef::capability(host, &FLUIDS, pos, Some(side))` and the answer is a
`ResourceHandler<FluidResource>` (`size`/`resource`/`amount`/`capacity`/
`insert`/`extract`/`insert_any`). `native_move_fluids` is
`ResourceHandler::insert_any` inside `host.transaction(..)` (the generic
`move_resources` helper exists for two capabilities, but the pump's own tank is
read from its compound, so each neighbour is filled with its share directly).
The storage shape did **not** change: the game still saves declared tanks in the
entity's `fluids` list as `{Fluid, Amount}` (verified in the game source at
nightly `9f42064`, `crates/horizon-client/src/game/machines/native_fluids.rs`
`read_tanks`/`write_tanks`), so `read_tank`/`write_tank` are unchanged.

### Manifest mapping

`content/horizon-mod.toml` (id/name/version **1.3.0**/api/side/capabilities) +
`mixins/rangedpumps/horizon-mixin.toml` (`[mod]`, `[compatibility]`, two
`[[module]]`, nine `[[injection]]`) + `horizon-native.json` + `compatibility.json`
→ **one** `horizon-mod.toml`: `id = "rangedpumps"`, `version = "0.2.0"` (the
port's version, not upstream's), `api = "0.2"`, `side = "both"`,
`capabilities = ["registries"]`, and `[native]` pinned to
`game = "26.3"`, `horizon = "0b14f8acfd97fa2c750e42795249dcc981178ca4"`,
`mixins = "e59da9094f0c2fdda6c627fe9eff22dec6a906f0"`.

### Build and check commands (all in one lease each)

```sh
cd ports/rangedpumps && cargo horizon-mod check            # fetches .horizon/sdk/, compiles, prints API usage
ports/tools/build-slot.sh run --job sdk-rp --port ports/rangedpumps --step test --  bash -c 'cd ports/rangedpumps && cargo test'
ports/tools/build-slot.sh run --job sdk-rp --port ports/rangedpumps --step check -- bash -c 'cd ports/rangedpumps && cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings'
ports/tools/build-slot.sh run --job sdk-rp --port ports/rangedpumps --step build -- bash -c 'cd ports/rangedpumps && cargo horizon-mod build && cp target/horizon-mod/rangedpumps.hmod .'
cargo horizon-mod check rangedpumps.hmod
cargo horizon-mod fits rangedpumps.hmod
```

### Traps (measured this slice)

1. **The runner forwards compile steps to the far box by default.** The first
   `cargo test` went to `deepseek@192.168.1.115` and died with
   `cd: /home/deepseek/atm10/rangedpumps: No such file or directory` (that copy
   still has the retired tree). Use `BUILD_LOCAL=1 …` until the far copy is
   re-synced.
2. **`cargo horizon-mod` must run inside the package.** From the campaign root
   it answers `error: horizon-mod.toml: No such file or directory (os error 2)`.
3. **The packer refuses the retired tree**: `error: unexpected package path:
   mixins/rangedpumps/runtime.rs` until `mixins/`, `content/`,
   `compatibility.json` and the `.hnmod` were deleted.
4. **The SDK is fetched per clone.** The first `cargo test` failed with
   `failed to load source for dependency horizon-mod-api … No such file or
   directory`; one `cargo horizon-mod` command fetches `.horizon/sdk/` (749
   surface items) and must run before any bare `cargo`.
5. **`JavaString` is not exported** by `mod_api` (`Compound` and `Tag` are), so
   a compound key cannot be named. `src/lib.rs` takes it from `Tag::String`'s
   own payload (`key_tag`/`tag`/`put`/`drop_tag`). Gap 10 below.
6. **`ModFluidTanks` is `#[non_exhaustive]`**: `Default::default()` then assign
   the public fields; a struct literal does not compile.
7. Small type facts that cost a round each: `BlockKey::id(host)` returns
   `String` (not `Option<String>`); `LevelRef::set_block` returns
   `Result<bool, LevelWriteError>`; `CommandInvocation::argument` returns
   `&CommandArgumentValue` (deref an `Integer` before `clamp`).
8. **A `BlockEntityTickEvent` carries no level.** The authoritative level is
   found with `LevelRef::in_place(host)` over `host.levels().list_levels()`
   (`in_place_level` in `src/lib.rs`).
9. Clippy (1.97+) denies `manual_is_multiple_of`: `ticks % SPEED_TICKS == 0`
   became `ticks.is_multiple_of(SPEED_TICKS)`.
10. **The e2e cannot run at this build.** It needs a 26.3 game binary compiled
    against the *same* `0b14f8a` SDK (`cargo horizon-mod run --game …` or
    `horizon-launcher native-build`); this machine has no `0b14f8a` checkout and
    no `horizon-launcher`, and the shared binaries are the `1d63e0a` /
    `41f822b` pin pair. The ladder therefore stops at `check` + `fits`; gap 9.

### Gaps (current, superseding the retired-format list below)

1. **No energy (FE) gating.** The SDK has energy capabilities
   (`block_energy`, `ENERGY`), but this campaign has no FE source and the source
   config leaves `useEnergy = false`; the pump runs ungated.
2. **No redstone pause.** A part could read `has_neighbor_signal`, but the
   upstream config's `useRedstone` is off by default; not implemented.
3. **Range is per block** (`configuredRange`, default 64, `/rangedpumps range`)
   instead of a server config file — unchanged from the retired slice.
4. **No config file** for `speed`, tank capacity or the replacement block; the
   upstream defaults (8 ticks, 32,000 mB, stone) are constants in `runtime.rs`.
5. **No custom renderer**; the 1.3.0 cube model and textures are carried.
6. **No pickaxe tag** — the 1.3.0 jar omits it; `requires_tool` is false.
7. **No in-game re-proof at this build** (trap 10): the tank, the drain, the
   output and the readout are the same code as the verified `.hnmod` slice with
   the calls named above, but nothing at `0b14f8a` has driven a pump yet.
8. **The readout is `use_without_item`** (empty hand). That is upstream's own
   method and more faithful than the old global `right_click_block`, but a
   right-click with an item in hand no longer prints the state.
9. **`JavaString` gap (API):** a mod cannot name its compound's key type, so
   every key read goes through `Tag::String`; worth an upstream note ("export
   `JavaString`, or give `Compound` `&str`-keyed `get`/`remove`").
10. **`ModFluidTanks` has no constructor** for a `#[non_exhaustive]` type; a
    mod must `Default::default()` and assign fields.

### Changelog for the resubmission (0.2.0)

> Supersedes #607, closed because Horizon retired `.hnmod` (#17992). Changes:
> one flat SDK package (`horizon-mod.toml`, `api = "0.2"`, `[native]` pinned to
> `0b14f8a` / mixins `e59da90`), the content moved in beside the code
> (`assets/rangedpumps/`, `data/rangedpumps/`) so no content is embedded and no
> `.hnmod` exists; the eight injection helpers became one `impl
> mod_api::Hooks` plus the pump's `BlockPart`/`BlockEntityPart` (typed
> hosts/events: `block_entity_type` + `block_fluids` + `BlockEntityPart::tick`,
> the readout as `use_without_item`, commands as `register_commands` +
> `run_mod_command`, the tank push through the `horizon:fluids` capability and
> `insert_any`); `runtime.rs` and its 13 tests are unchanged.

### Verification (SDK, 2026-11-18)

Every step ran through `ports/tools/build-slot.sh` (`BUILD_LOCAL=1`,
`CARGO_INCREMENTAL=0`); logs in `ports/_shared/evidence/rangedpumps/`.

| Step | Command | Result |
|---|---|---|
| runtime | `cargo test` | **13 passed**, 0 failed (`sdk-test.log`) |
| fmt | `cargo fmt --all -- --check` | clean (`sdk-fmt.log`) |
| clippy | `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| sdk check | `cargo horizon-mod check` | exit 0, **290 API items**, made for `0b14f8a` (`sdk-check.log`) |
| package | `cargo horizon-mod build` | **28,570 B**, SHA-256 `8199584f111bf95a9e844ad2f663a32a1c9af58521dcf09fc68ccb28cfa4bad9`, **21 entries**, reproducible (`sdk-package.txt`) |
| package check | `cargo horizon-mod check rangedpumps.hmod` | `native-mod`, format `native-1/0b14f8a…/e59da90…`, for 26.3 (`sdk-build-check-fits.log`) |
| fits | `cargo horizon-mod fits rangedpumps.hmod` | `0b14f8a  made for` (`sdk-build-check-fits.log`) |
| runtime identity | `cmp src/runtime.rs <retired copy>` | identical, `af81428e…` (`sdk-package.txt`) |
| e2e | — | **not run** (gap 7 / trap 10) |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

## Upstream source analysis

- Repository: <https://github.com/refinedmods/rangedpumps> (MIT), commit
  `13da16f526c5064f9f3d1d671982e6744dec470c` (2026-07-13), shallow clone in
  `ports/_shared/upstream/rangedpumps`.
- Files read: `blockentity/PumpBlockEntity.java`, `blockentity/PumpState.java`,
  `block/PumpBlock.java`, `config/ServerConfig.java`, `RangedPumps.java`; the
  jar's `assets/rangedpumps/*` and `data/rangedpumps/*`.
- Behaviours extracted: the three-field scan (`range` starting -1,
  `columnIndex`, packed `currentPos`), the perimeter-only ring, the 8-tick
  move, `LiquidBlock` + `LEVEL == 0` as the only drain target, one whole bucket
  per drain, 32,000 mB tank, output share `amount / handlers`, air then stone
  replacement, state order `DONE/REDSTONE/ENERGY/FULL/WORKING`, and the two-line
  right-click readout.
- Where the rewrite differs: see Gaps (energy, redstone, config file, per-block
  range, output after the level tick, saved tick counter).

## Slice boundary and gaps (retired `.hnmod` slice)

The live gap list is in [`## Migration to the SDK format`](#migration-to-the-sdk-format);
the items below are the retired slice's, and #7 (output after the level tick) is
withdrawn there.

Implemented: the pump block, per-block range setting, the scan, still-source
drain, the internal tank and its fluid capability, adjacent output, stone
replacement, the readout, commands, and the carried item/recipe/loot/lang/
models/textures.

1. **No energy (FE) gating.** The pin has API 142, but this campaign has no FE
   source and the slice brief permits energy as a gap; the pump runs as if
   `useEnergy = false`.
2. **No redstone pause.** A neighbour-signal read needs `LevelAccess`/`Ticks`,
   which the block-entity tick does not carry.
3. **Range is per block** (saved, default 64, `/rangedpumps range`) instead of
   a server config file; this is the slice's range setting.
4. **No config file** for `speed`, tank capacity or the replacement block;
   the upstream defaults (8 ticks, 32,000 mB, stone) are constants.
5. **No custom renderer**; upstream's cube model and textures are carried.
6. **No pickaxe tag** — the 1.3.0 jar omits it (repo HEAD has one); the
   carried content follows the jar. `requires_tool` is false as upstream.
7. **Output runs after the level tick** (the block entity is out of the world
   during its own tick), walking the level's mod block entities.
8. **Ticks are saved** (`ticks` in the block entity); upstream resets its
   counter on load.
9. **`tools/test_template.py`** is the template's one-injection fixture
   (`break_block`), not a test of this bundle; the real ladder is the evidence.

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

## Verification (retired `.hnmod` slices — history)

The SDK-format ladder is in [`## Migration to the SDK format`](#migration-to-the-sdk-format).

Pin **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`**, mixins
**`5a64af938ec86b00d53d16a408aabc0038fb3e57`** (schema 6). Every build/test
ran through `ports/tools/build-slot.sh` with `CARGO_INCREMENTAL=0`; the in-game
check used the pin-matched shared `horizon-ctl`
(`ports/_shared/bin/horizon-1d63e0a5/horizon-ctl`) and the port-built fast game.

### Ladder

| Step | Command (via the slot runner) | Result |
|---|---|---|
| runtime RED | `cargo test --offline --manifest-path ports/rangedpumps/Cargo.toml` | failed to compile (missing API) — `evidence/runtime-red.log` |
| runtime GREEN | `cargo test --locked --offline` | **11 passed**, 0 failed |
| fmt + clippy | `cargo fmt --check`; `cargo clippy --all-targets --locked --offline -- -D warnings` | pass, 0 warnings |
| fetch | `cargo fetch --locked` on the staged lock | exit 0 (once, online) |
| stage | `python3 tools/horizon.py stage --horizon …/pins/horizon-1d63e0a` | exit 0, revision `1d63e0a…` |
| check | `python3 tools/horizon.py check …` | `cargo_exit: 0`, **0 warnings** (`evidence/horizon-check.log`) |
| test | `python3 tools/horizon.py test …` | **11 passed**, 1240 filtered (`evidence/horizon-test.log`) |
| build | `python3 tools/horizon.py build …` | `cargo_exit: 0` (fast) |
| e2e | `build-slot.sh e2e -- … -- tools/e2e.sh` | exit 0, `PASS`, `EXIT:0` (`evidence/e2e.log`) |
| selftest | `SELFTEST=1 … tools/e2e.sh` | **exit 1** with the flipped assertion (`evidence/e2e-selftest.log`) |
| `.hmod` | `cargo-horizon-mod build/check` | **5,068 B**, SHA-256 `9e09290e…f8f2c`, 13 entries, content only |
| load check | `tools/hmod-load.sh` | `PASS` + `EXIT:0`; negative: `Unknown block type 'rangedpumps:pump'` |

### In-game scenario (quoted from `e2e.log`)

```
== A: a still water source below is drained into the tank ==
  source 0,63,0 -> minecraft:stone
  info(0,64,0): Ranged Pump: working, range 0 of 0, scan 0, 56, 0, minecraft:water 1000 / 32000 mB
== B: the tank is pushed into the adjacent cauldron (output) ==
  cauldron 9,64,0 -> minecraft:water_cauldron
  info(8,64,0): Ranged Pump: working, range 0 of 0, scan 8, 52, 0, empty 0 / 32000 mB
== C: no fluid below leaves the tank empty (control) ==
  info(16,64,0): Ranged Pump: working, range 0 of 0, scan 16, 52, 0, empty 0 / 32000 mB
== D: a source outside the range-0 ring is not drained (control) ==
  info(24,64,0): Ranged Pump: working, range 0 of 0, scan 24, 51, 0, empty 0 / 32000 mB
  info(24,64,0): Ranged Pump: working, range 0 of 0, scan 24, 48, 0, empty 0 / 32000 mB
== E: a waterlogged stair is not a source (control) ==
  info(32,64,0): Ranged Pump: working, range 0 of 0, scan 32, 47, 0, empty 0 / 32000 mB
  stair 32,63,0 still minecraft:oak_stairs
== range: a larger range re-scans from the block below ==
  info(16,64,0): Ranged Pump: working, range 0 of 2, scan 16, 63, 0, empty 0 / 32000 mB
PASS: …
EXIT:0
```

What the controls prove: the **same** enclosed water source is drained when it
is the pump's first scan target (A) but **not** when it is two blocks away,
outside the range-0 eight-neighbour ring (D); a pump over stone (C) and one
over a waterlogged stair (E) stay empty. The cauldron beside B receives the
bucket, so the tank is genuinely emptied through the fluid capability rather
than only read. `SELFTEST=1` flips A to "empty" and fails, so the scenario is
not vacuous.

### Fail-capable evidence

```
(selftest: asserting A's tank is empty)
  info(0,64,0): Ranged Pump: working, range 0 of 0, scan 0, 56, 0, minecraft:water 1000 / 32000 mB
FAIL: expected /rangedpumps info to say 'empty', got 'Ranged Pump: working, range 0 of 0, scan 0, 56, 0, minecraft:water 1000 / 32000 mB'
```

### Content package and load check

`content/rangedpumps.hmod` **5,068 B**, SHA-256
`9e09290ea97e37f70f40cd391240a553e825b9130cd02f47d02f7fa0086f8f2c`,
**13 entries**, `horizon-mod check`: mod, API 0.2, both, content only.
`tools/hmod-load.sh` placed and gave `rangedpumps:pump` on the shared unmodded
pin game and the same id is `Unknown block type` without `--dev-mod`.

## Content embedded in the bundle; re-verified in game (2026-11-10)

**Beat: one file per project.** The registry takes one `file` per version, so a
port that ships a `.hmod` *and* a `.hnmod` cannot be listed faithfully. This port
now follows the AE2 pattern proved on `charginggadgets` (PR #526): the built
content mod travels **inside** the bundle at
`mixins/rangedpumps/data/rangedpumps-content.hmod`, and `mixins/rangedpumps/embedded_content.rs`
compiles it in with `horizon_client::native_data!` and injects it at
`content_root` (`target = "horizon_client::game::mods::content_root"`,
`kind = "HEAD"`, `cancellable = true`). The helper writes the file to
`.horizon-cache/rangedpumps-embedded-content.hmod` once, appends it to the dev-mod list
and returns the merged root; the recursive call stops at its own
`dev.contains(&file)` guard, so the vanilla body runs once. The `.hmod` stays in
`content/` as the build input; `tools/package.py` refuses a stale copy in `data/`.

**Proved with no `--dev-mod`:** the e2e launches the game *without* passing the
content folder, so every content id and every behaviour it asserts comes from the
bundle. `EMBEDDED=1` is the flag; the launch line passes an empty dev-mod list.

| Artifact | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `content/rangedpumps.hmod` (input) | 5,068 B | `9e09290ea97e37f70f40cd391240a553e825b9130cd02f47d02f7fa0086f8f2c` | 13 |
| `rangedpumps.hnmod` (deliverable, embeds it) | 16,641 B | `3b5ef70b409830a57b32969e5f36bc6d1344746124a82bf413f7d6273a91f405` | 14 |

E2E (one lease each, `CARGO_INCREMENTAL=0`, shared `1d63e0a5` `horizon-ctl`, the
port's own fast game): default `PASS` + `EXIT:0`; `SELFTEST=1` `FAIL` + `EXIT:1`.

`python3 ports/tools/check-e2e-logs.py rangedpumps --strict` → every log carries its
own `PASS`/`FAIL` + `EXIT:` pair. `python3 ports/tools/upstream_ready.py rangedpumps` →
**READY (exit 0)**.

## Review repairs (PR #607, 2026-11-11)

Review: `ports/_shared/horizon-mods/reviews/pr-607-rangedpumps.md` (CHANGES_REQUESTED, 5 items).

| Reviewer's item | This job |
|---|---|
| 1. relative link, truncated paragraphs, "Not yet ported" listing implemented features | **rewritten** from this file: the implemented scan/drain/tank/output list, then the real gaps (no FE gating, no redstone pause, no screen) |
| 2. artifact not staged | **orchestrator** (no write access) |
| 3. **scan positions were consumed when the tank could not take the source** | **fixed** (see below) |
| 4. **a pump on the dimension's bottom block decremented below `min_y` forever** | **fixed** (see below) |
| 5. upstream MIT notice missing from repository notice and both archives | **fixed**: `LICENSE` now carries `Copyright (c) 2016-2026 Refined Mods`, the packager ships it in both archives |

### the scan only commits a position it settled

`support.rs` advanced the scan and wrote it back even when `insert_bucket` failed, so every
source behind a full tank was skipped for good. The tick now advances a **copy**: a source
the tank cannot take leaves the scan where it was (retried next tick), a non-source moves
it on, and a finished scan stays finished. The incompatible-fluid case is the same path —
`insert_bucket` refuses and the block is left alone.

### a column that starts below the floor is skipped

`Scan::advance` ended a column only on `pos[1] == min_y`, so `origin.y - 1 < min_y` (a pump
on the bottom block) decremented forever. It now ends at or below the bound and skips a
column whose top is already below `min_y`.

Tests: `a_pump_on_the_bottom_block_stops_instead_of_descending_forever` and
`a_column_ends_at_the_floor_and_the_next_one_starts` (the first hangs the old code).

Verified: `python3 tools/horizon.py test` in a build slot — staged compile `cargo_exit: 0`,
**13/13** embedded tests pass, including
`a_pump_on_the_bottom_block_stops_instead_of_descending_forever` and
`a_column_ends_at_the_floor_and_the_next_one_starts`
(`ports/_shared/evidence/rangedpumps/review-repairs-test.log`).

Verified after the change: `tools/e2e.sh` — **PASS + `EXIT:0`**: a still source below is
drained and becomes stone, the tank's bucket reaches an adjacent cauldron, and the no-fluid,
out-of-range and waterlogged-stair controls stay empty, with the range command re-scanning
(`ports/_shared/evidence/rangedpumps/e2e.log`).

Artifact: `rangedpumps.hnmod` **19,338 B** `0c622b403b4d9d77bafddc0be30485c6cc9b7f2bb833d7922bb3a3fb23e035a3`
(content `rangedpumps.hmod`).
