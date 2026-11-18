# Ranged Pumps — upstream findings and slice boundary

Ported from **Ranged Pumps 1.3.0** (`refinedmods/rangedpumps`, **MIT**),
Minecraft Java 1.21 NeoForge. **LOCAL-ONLY** until the maintainer approves
publishing (ARCHITECTURE.md §8).

## Provenance

| | Value |
|---|---|
| ATM10 modlist entry | `rangedpumps-1.3.0.jar`, modId `rangedpumps`, `modrinthHash` (SHA-1) `b2430e501d0c03d0464ec351e3e839cb2a56a06a`, `curseForgeHash` (fingerprint) `623574109` |
| Jar fetched | `https://edge.forgecdn.net/files/5508/029/rangedpumps-1.3.0.jar` (CurseForge file 5508029, project 247496) |
| Jar size / SHA-1 | `25,483 B` / `b2430e501d0c03d0464ec351e3e839cb2a56a06a` (matches the ATM10 entry) |
| Modrinth 1.3.0 file | `0e843f98…c0f3` — a different build of the same version; the pack's jar is the CurseForge build, so the Modrinth SHA-1 lookup returns 404 and the CurseForge file is the source of truth |
| `META-INF/neoforge.mods.toml` | `license = "MIT"`, version `1.3.0`, authors `Refined Mods` |
| Upstream source | <https://github.com/refinedmods/rangedpumps>, commit `13da16f526c5064f9f3d1d671982e6744dec470c` (2026-07-13, `docs: curseforge link fix`), cloned `--depth 1` to `ports/_shared/upstream/rangedpumps` |
| Jar content | `assets/rangedpumps/{blockstates,models,textures,lang}`, `data/rangedpumps/{recipe,loot_table}`, no pickaxe tag |

## Files read

`PumpBlockEntity.java`, `PumpState.java`, `PumpBlock.java`, `ServerConfig.java`,
`RangedPumps.java`; the jar's blockstate/model/item/lang/recipe/loot entries.

## Behaviour extracted

### The scan

- `range` is a **server config**, default **64**, clamp `0..=1024`.
- `speed` is a **server config**, default **8** ticks per move.
- The scan state is three saved fields: `range` (starts `-1`), `columnIndex`,
  `currentPos` (a packed long).
- Range `-1` is the single column directly **below** the pump. Each subsequent
  range `r` is the **perimeter** of a `(2r+3) × (2r+3)` square (the 8, 16, 24…
  block ring at horizontal distance `r+1`) — the interior is *not* scanned.
- Within a column the pump goes down one block per move until the dimension's
  minimum Y, then starts the next column at the pump's `y - 1`.
- At every move it inspects the block: only a `LiquidBlock` whose `LEVEL` is 0
  (a still source) is drained. Flowing water/lava (`level` 1–7) and waterlogged
  non-liquid blocks are ignored.

### The tank and the drain

- Tank capacity **32 buckets = 32,000 mB**, one fluid at a time.
- A drain removes exactly **1,000 mB** and only succeeds if a whole bucket fits;
  otherwise the source is left in place (the scan has already moved on).
- The tank refuses external insertion (`PumpTank.insert` returns 0) and allows
  external extraction (the capability's `extract` is inherited).
- `native_block_entity_fluids` (API 54) is the pin shape: one 32,000 mB tank,
  all faces reach it, `inputs = []`, `outputs = [0]`.

### The output

- Each tick `fillNeighbors` moves `tank_amount / neighbour_count` mB into every
  adjacent block that exposes a fluid handler; a vanilla cauldron is such a
  handler (whole thirds of a bucket).

### The source replacement

- `replaceLiquidWithBlock` default **true**, `blockIdToReplaceLiquidsWith`
  default **`minecraft:stone`**. After a drain the source becomes air and then
  stone.

### The states and the readout

- State order: `DONE` (range > configured), `REDSTONE` (neighbour signal),
  `ENERGY` (0 FE), `FULL` (`amount > capacity - 1000`), else `WORKING`.
- Only `WORKING` drains.
- Right click with no item sends two chat lines: the state (`Scanning at x, y, z
  (range: r of max)`, `Internal tank is full.`, `Done.`, …) and the tank
  (`<n> mB of <fluid> - <fe> / <fe> FE`).
- `BlockEntity.setChanged()` marks the chunk dirty on every scan move and
  drain.

### Energy (gap)

- `energyCapacity` 32,000 FE, `energyUsagePerPerMove` 0, `energyUsagePerDrain`
  100, `useEnergy` true by default; external extraction is refused.
- This slice does **not** implement FE (see Gaps): the pump works without
  energy, as if `useEnergy = false`.

## Slice 1 boundary (this job)

Ported: the pump block, per-block range setting, the scan, still-source drain,
the 32-bucket internal tank exposed through `block_entity_fluids`, output into
adjacent fluid handlers, source replacement with stone, the right-click
readout, `/rangedpumps info|range`, and the carried item/recipe/loot/lang/
models/textures.

## Gaps / deviations (slice 1)

1. **No energy (FE) gating.** API 142 exists at the pin, but there is no FE
   source in this campaign and the slice brief allows energy to be a gap. The
   pump always runs, equivalent to many packs' `useEnergy = false`.
2. **No redstone pause.** Reading a neighbour signal needs a `LevelAccess`/
   `Ticks` pair the block-entity tick does not carry; the pump keeps running
   under redstone.
3. **Range is per block** (saved `configuredRange`, default 64, `/rangedpumps
   range`) instead of a single server config file. This is the "range setting"
   the slice asks for and makes the in-game check precise.
4. **No config file.** `speed`, tank capacity and the replacement block are the
   upstream defaults as constants (8 ticks, 32,000 mB, stone); the replace
   block is not configurable.
5. **No custom renderer**; the pump keeps upstream's cube model and textures.
6. **No pickaxe tag.** The ATM10 jar omits `data/minecraft/tags/block/mineable/
   pickaxe.json` (the repo HEAD carries it); the carried content follows the
   jar. `requires_tool` is false, as upstream's plain `Block` is.
7. **Output push runs after the level tick**, walking the level's mod block
   entities, because a block entity is out of the world during its own tick.
8. **Ticks are saved** in the block entity (`ticks`); upstream resets its
   counter on load.
