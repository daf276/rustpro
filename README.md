# rustpro

A pure-Rust reimplementation of the EDOPro duel engine ([edo9300/ygopro-core](https://github.com/edo9300/ygopro-core) and the [ProjectIgnis CardScripts](https://github.com/ProjectIgnis/CardScripts)), built for massively parallel Yu-Gi-Oh! simulation: game-tree search, RL training and deck statistics. There is no Lua at runtime.

- **Exact:** the processor is a near line-by-line port of ocgcore, and every card is checked by differential fuzzing against the real ocgcore (linked through FFI as an oracle): same seed, decks and answers in both engines, and the message streams must match byte for byte, misrulings included.
- **Scope:** the current TCG Advanced format, one Master Rule, no Speed/Rush/Goat or skills.
- **Simulation API:** load a gamestate, list the legal actions, apply one, clone cheaply at any decision point. Per-player observations, hidden-card sampling, a fixed action encoding with masks, a rayon batch runner and Python bindings.

Card scripts are ported to Rust by LLM agents in batches, each card gated by the oracle diff. See `PORTING.md` for how a card is ported and `PLAN.md` for the design, the per-step findings and the open gaps.

## Layout

| Crate | What it is |
|---|---|
| `engine` | The duel core: processor, effects, the Lua-mirror API (`Duel`, `Card`, `Effect`, `Group`), observations, gamestate formats |
| `scripts` | The ported card scripts (`src/official/c<code>.rs`) and the shared library (`utility.lua`, `proc_*.lua` ports) |
| `carddata` | Card database reading (BabelCDB) |
| `sim` | Action encoding, observation tensors, the batch runner |
| `py` | The `ygo_sim` Python module (pyo3, maturin) |
| `ocgcore-sys`, `oracle` | The C++ ocgcore built with `cc`, and a safe wrapper with a random player |
| `diff` | The differential harness: per-card scenarios, deck suites, the `card` command |
| `bench` | Throughput and clone benchmarks |

## Setup

The oracle sources and card data are not in the repository. Clone them into `vendor/` at the pinned commits:

| Directory | Repository | Commit |
|---|---|---|
| `vendor/ygopro-core` | https://github.com/edo9300/ygopro-core (with its Lua submodule) | `baa3881d0bae48e9994f915f5c931be53c1ba8ef` |
| `vendor/CardScripts` | https://github.com/ProjectIgnis/CardScripts | `3b3a66f685e4c1d0d82214550e56f54c001cc30c` |
| `vendor/BabelCDB` | https://github.com/ProjectIgnis/BabelCDB | `cf9a274244f2e61dfad12a68ed4517613ac65ba5` |
| `vendor/LFLists` | https://github.com/ProjectIgnis/LFLists | `73b027287c6123832fca165f5d76204362fa1078` |

## Usage

```sh
# Check one or more ported cards against the oracle (64 random scenarios each)
cargo run --profile card -p diff --bin card -- <code>... --threads 8

# All tests (the diff suites run a sample of the ported cards)
cargo test --profile card --workspace

# Python module
maturin develop --release -m crates/py/Cargo.toml
python -m pytest crates/py/tests

# Throughput benchmark of the full stack
cargo run --profile sim -p bench --bin stack
```

The `card` profile is a release build with incremental compilation, so a card edit rebuilds in seconds.

## Status

About 930 card files are ported, weighted toward the most played cards. Single-threaded throughput on an Elfnote mirror is about 260k actions per second (318k with PGO, `tools/pgo.py`).

## License

AGPL-3.0-or-later (see `LICENSE`). ocgcore and the CardScripts are AGPL-3.0, and this port is a derivative work of both.
