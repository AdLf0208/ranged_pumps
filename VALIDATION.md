# Validation

## SDK format (2026-11-18) — current

SDK nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`**, mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, mod API 0.2, version 0.2.0.
Every step through `ports/tools/build-slot.sh` with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; logs in `ports/_shared/evidence/rangedpumps/`.

| Check | Result |
|---|---|
| `cargo test` | **13 passed**, 0 failed |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings |
| `cargo horizon-mod check` | exit 0 — 290 API items, "Made for 0b14f8a" |
| `cargo horizon-mod build` | `rangedpumps.hmod` **28,570 B**, SHA-256 `8199584f…a4bad9`, 21 entries, reproducible on rebuild |
| `cargo horizon-mod check rangedpumps.hmod` | `native-mod`, `native-1/0b14f8a…/e59da90…`, for 26.3 |
| `cargo horizon-mod fits rangedpumps.hmod` | `0b14f8a  made for` |
| `src/runtime.rs` identity | `cmp` against the retired `.hnmod` copy: identical |
| e2e | **not run** — no 26.3 game binary for `0b14f8a` on this machine (see notes.md, gap 7) |

The retired-format ladder below is history: it validated the `.hnmod` that PR
#607 carried, not the SDK package.

## `.hnmod` ladder (retired format — history)

Validated on Linux with Rust 1.97.0 against the exact revisions in
`compatibility.json`:
Horizon `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`, horizon-mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57` (schema 6). Every build/test step
ran through `ports/tools/build-slot.sh` with `CARGO_INCREMENTAL=0`; the
in-game check used the pin-matched shared `horizon-ctl`.

| Check | Result |
|---|---|
| `cargo fmt --check` | Pass |
| `cargo clippy --all-targets --locked --offline -- -D warnings` | Pass |
| `cargo test --locked --offline` (standalone) | **11 passed**, 0 failed |
| `tools/horizon.py stage` | exit 0, revision `1d63e0a…` |
| `tools/horizon.py check` | `cargo_exit: 0`, **0 warnings** |
| `tools/horizon.py test` | **11 passed** (`game::rangedpumps::tests`, 1240 filtered) |
| `tools/horizon.py build` (fast) | `cargo_exit: 0` |
| `tools/e2e.sh` | `PASS` + `EXIT:0` (ctl transcript in notes.md) |
| `SELFTEST=1 tools/e2e.sh` | **exit 1**, `FAIL: expected … 'empty'` (fail-capable) |
| `tools/hmod-load.sh` | `PASS` + `EXIT:0`; same ids unknown without the package |

Evidence logs: `ports/_shared/evidence/rangedpumps/`.

`tools/test_template.py` is the template's own one-injection fixture
(`break_block`); it is not a test of this eight-injection bundle and was not
used as evidence. The real bundle is validated by the ladder above.

## Embedded content + re-verification (2026-11-10)

Horizon `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9` (game.rs `64813cf4…`),
horizon-mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57` (schema 6), every step
through `ports/tools/build-slot.sh` with `CARGO_INCREMENTAL=0`.

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --all-targets --locked --offline -- -D warnings` | exit 0, 0 warnings |
| `cargo test --locked --offline` | green |
| `horizon.py stage` / `check` / `test` | exit 0 / `cargo_exit: 0`, 0 warnings / embedded tests green |
| `tools/package.sh` | `content/rangedpumps.hmod` 5,068 B `9e09290ea97e37f7…`, `rangedpumps.hnmod` 16,641 B `3b5ef70b409830a5…` (14 entries, embeds the content) |
| e2e (no `--dev-mod`) | default `PASS` + `EXIT:0`; `SELFTEST=1` `FAIL` + `EXIT:1` |
| `upstream_ready.py rangedpumps` | **READY (exit 0)** |
