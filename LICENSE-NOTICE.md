# Licence notice — Ranged Pumps

- **Upstream:** Ranged Pumps 1.3.0 for Minecraft 1.21.1 / NeoForge
  (`refinedmods/rangedpumps`, <https://github.com/refinedmods/rangedpumps>).
  The pack jar is `rangedpumps-1.3.0.jar` (SHA-1
  `b2430e501d0c03d0464ec351e3e839cb2a56a06a`, CurseForge file `5508029`); its
  `META-INF/neoforge.mods.toml` declares `license = "MIT"`, which the source
  map and `ports/_shared/licence-audit.csv` also record. The jar is not in
  `ports/_shared/licence-cache/` (it was read at import), so the recorded
  SHA-1 and the repository's `LICENSE` are the evidence.
- **Licence:** MIT. `LICENSE` carries both copyright lines: upstream
  (`Copyright (c) 2016-2026 Refined Mods`) and the Horizon port
  (`Copyright (c) 2026 AdLf0208`).
- **Carried material:** the mod's own assets and data only — the pump block's
  blockstate, block and item models, three textures, `en_us` lang, and the
  block definition, creative tab, shaped recipe and loot table under
  `assets/rangedpumps/` and `data/rangedpumps/`. No Java code, no compiled
  classes, no Mojang material.
- **Rust:** a clean-room rewrite against Horizon's documented mod API
  (`horizon-mod-api` at the pinned SDK); `src/runtime.rs` is this campaign's
  pure logic and `src/lib.rs` is the game glue.
- **Rank:** 0 (MIT, permissive). Still **local-only** until the maintainer
  approves publishing (ARCHITECTURE.md §8); MIT itself allows derivatives and
  redistribution with this notice.
