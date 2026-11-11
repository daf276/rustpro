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

# All tests (the per-card suites run a sample of the ported cards; DIFF_ALL_CARDS=1 runs every card)
cargo test --profile card --workspace

# Python module (unwinds on panic, so a panic reaches Python as an exception)
maturin develop --profile python -m crates/py/Cargo.toml
python -m pytest crates/py/tests

# Throughput benchmark of the full stack
cargo run --profile sim -p bench --bin stack

# The same with PGO (binaries in target/pgo-use/sim/); the Python module: python tools/pgo.py maturin develop -m crates/py/Cargo.toml
python tools/pgo.py -p bench --bin stack --bin throughput
```

The `card` profile is a release build with incremental compilation, so a card edit rebuilds in seconds. The card command and the diff suites cache a hash of the oracle's output per scenario, so a scenario whose setup is unchanged only replays the engine (`ORACLE_CACHE=off` disables it).

## Status

2,160 card files are ported, covering all of the 2,000 most viewed cards of the TCG Advanced pool (by ygoprodeck weekly views; 68 of them are Normal Monsters without a script). The ported and script-less cards make up 71% of the pool's views. `porting/queue.tsv` lists the pool in porting order and what is ported.

## Accuracy

Every result is checked against the real ocgcore, never against expectations written by hand: both engines get the same seed, decks and answers, and their message streams must match byte for byte, misrulings included.

- **Per card, before it is merged:** 64 random scenarios, each placing the card with support cards picked to reach its effects and playing the duel out with random legal answers in both engines. The card's checks must also show that each of its effects actually resolved, in the first 13 scenarios and in all 64. That is about 140,000 diffed duels over the ported cards.
- **Default suite** (`cargo test`, about 40 tests):
  - 1,000 duels each of Normal Monster decks, a classic effect deck and the Elfnote mirror;
  - 13 scenarios for about 200 cards, rotated each run, plus every card ported since the last full run;
  - gamestate round trips: saved positions and replayed records continue message for message like the original;
  - observations checked against what the core marks public, hidden cards sampled evenly, and duels playing the same whether observed or not;
  - the action encoding and tensors at every prompt of the sampled duels.
- **Full suite** (`DIFF_ALL_CARDS=1 cargo test --profile card --workspace -- --include-ignored`): 5,000 duels per deck and every card at 64 scenarios.

## Benchmarks

Elfnote mirror matches (a YCS-winning deck with Synchro, Xyz and Link play) under uniformly random legal actions, setup excluded, `sim` profile, Ryzen 7 5700X (8 cores), on an otherwise idle machine:

| | 1 thread | 7 threads |
|---|---|---|
| Actions applied per second | 223k | 1.11M |
| Decisions (prompts with more than one legal action) per second | 104k | 516k |
| Batch runner (64 duels in lockstep), actions per second | 153k-166k | 616k-678k |
| Clone at a decision point, median | 3.7 µs | 5.5 µs |
| Observe, median | 3.2 µs | 4.1 µs |
| Encode the legal actions, median | 0.4 µs | 0.6 µs |
| Determinize (resample hidden cards), median | 19.3 µs | 26.8 µs |

The table is from before the performance pass of 2026-11-11 and is re-measured when it ends. Measured against it side by side (both binaries at once on two cores, median of six rounds, 1 thread), master runs 1.24x on the Elfnote mirror (about 276k actions/s) and 1.58x on Normal Monster duels (about 1.11M); built with PGO, 1.56x (about 347k) and 2.22x (about 1.57M). The `sim` profile aborts on panic and the benches use mimalloc (`--features system-allocator` for the system heap); `-C target-cpu=x86-64-v3` adds about 2% on AVX2 machines. For comparison, ocgcore with Lua runs about 10.8k decisions per second per thread on random Advanced decks.

## License

AGPL-3.0-or-later (see `LICENSE`). ocgcore and the CardScripts are AGPL-3.0, and this port is a derivative work of both.
