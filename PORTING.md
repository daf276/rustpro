# Porting a card

Port `vendor/CardScripts/official/c<code>.lua` to `crates/scripts/src/official/c<code>.rs` so that the engine matches
the oracle (ocgcore running the Lua) message for message. The file is all there is to add: `crates/scripts/build.rs`
registers every `c<code>.rs` and reads its `LISTED_NAMES`, `FIT_MONSTER`, `SUPPORT` and `CHECKS`. The order of work is
`porting/queue.tsv` (TCG Advanced pool, most viewed first; `ported` is `no script` for cards that need no port).
`vendor/` is read-only.

## Workflow

1. Read the Lua and every library function it calls (`utility.lua`, `cards_specific_functions.lua`, `proc_*.lua`).
   Read two or three ported cards with similar effects (`grep -l EVENT_SPSUMMON_SUCCESS crates/scripts/src/official`).
2. Write the file: `// <card name>`, `use crate::prelude::*;`, `const ID: u32 = <code>;`, then
   `pub fn initial_effect(duel: &mut Duel, c: CardId)` and the script's functions in the Lua's order and names.
3. A library function the card needs that is not ported yet is ported now, from the Lua, into the matching module
   (`utility.rs` for `aux`/`Cost` and the `CardUtility`/`DuelUtility`/`EffectUtility` traits, `proc_*.rs` for the
   procedures). A missing core function goes into the engine's mirror (`libduel.rs`, `libcard.rs`, `libeffect.rs`,
   `libgroup.rs`), ported from ygopro-core's `lib*.cpp` and the processor code it reaches. Port only the paths ported
   cards can reach; leave the rest as `unported!(duel, <condition the Lua would take that path on>)`.
   The scripts crate's `unported!` takes a bool, the engine's an iterator of effects. Check `proc_workaround.lua`
   first: it overrides some core functions (`Duel.IsMainPhase(player)` also checks whose turn it is).
4. Declare `SUPPORT` and, if the default does not fit, `CHECKS` (below).
5. Run `cargo run --release -p diff --bin card -- <code> --threads 7` and iterate until it passes. Several codes may
   be given at once. Then run `cargo test --release -p diff -j 7` (all suites must stay clean),
   `cargo clippy --workspace --all-targets -j 7`, `cargo fmt --all` and
   `rustfmt --edition 2024 crates/scripts/src/official/c<code>.rs` (`cargo fmt` does not reach card files, since
   build.rs includes them).

Never weaken a check (the card's own or another card's) or the harness to get a pass. If a card cannot be ported
without engine work out of scope, stop and report what is missing.

## The card command

Per card: `pass`/`FAIL` and how many scenarios diffed clean; each check with its count over the first 13 scenarios
(the default test) and over all of them (default 64, `--scenarios K`); the resolved chain links per effect (`#n` is
`aux.Stringid(code, n)`); and the first failing scenario: either the unported call site reached, or the first
differing message, with its index in the stream, the last messages both engines wrote alike, then each engine's
messages from there (`MSG_<type> [bytes]`). Other failing scenarios are listed by index. A code without a file says
`not ported`. The exit code is 1 on any failure. A scenario depends only on (code, index), so a failure reproduces
exactly; ports of other cards never change it.

## Test metadata

Each scenario places the card for a random player in a random zone its type can hold, its `SUPPORT` cards, and up to
three cards from a fixed set (the Elfnote deck and the classic cards), then random-plays Elfnote mirror decks.

- `pub const SUPPORT: &[(u32, u16)]`: (code, location) of cards the effects need (costs, targets, materials, a second
  copy), with locations from `crate::testing` (`HAND`, `DECK`, `MZONE`, `SZONE`, `GRAVE`, `REMOVED`, `EXTRA`, or 0
  for a random zone), `| OPPONENT` for the other player. Only ported cards and Normal Monsters (which have no
  script); one comment line naming them.
  An unported card in SUPPORT runs its Lua in the oracle but nothing in the engine, so the diff fails: port a
  needed support card first (it then has its own checks).
- `pub const CHECKS: &[Check]` (`crate::testing::Check`), each needing a count above 0 in both the first 13 and all
  64 scenarios. Without `CHECKS` the card must resolve some effect (`Check::Resolves`). Others:
  `ResolvesEffect(aux::stringid(ID, n))`, `SpecialSummoned`, `RitualMaterial`, `MaterialFrom(location)`,
  `CardHint(description)`, `Message(MSG_*)` (from `engine::constants`; the message must be about the card),
  `TributeSummoned(n)`, `TributeSet(n)`, `Replaced`, and `Report(&check)`, which only writes the count out.
  Check each effect that needs a specific path; use another check than `Resolves` only for a card that has no
  effect a scenario can resolve, and say why in one comment line.
- If a check fails at 13 but passes at 64, improve `SUPPORT` (place what the effect needs where it can be used).

## Mirror API

- `Duel.X(...)` is `duel.x(...)`; `c:X(...)`, `e:X(...)`, `g:X(...)` are methods on `CardId`, `EffectId`,
  `CardSet` taking the duel first: `c.is_type(duel, TYPE_SYNCHRO)`. `Effect.CreateEffect(c)` is
  `duel.create_effect(c)`, `e:Clone()` is `e.clone_effect(duel)`, `#g` is `g.len()`. Names are the Lua names in
  snake case.
- Optional trailing arguments: the plain name takes the arguments most calls pass; `_full` takes all of them. Call
  `_full` exactly when the Lua call passes one of the optional ones, with the Lua default for any it leaves out
  (`c.is_type_full(duel, TYPE_TUNER, Some(sc), sumtype, tp)`). Lua nil is `None` for cards and groups and
  `PLAYER_NONE` for a player.
- Functions that can prompt or yield return `Script<T>`; propagate with `?`. A target or cost returns `Ok(chk0)` for
  chk 0 and `Ok(false)` after doing its work, where the Lua returns nothing.
- Filters are `Fn(CardId, &Duel) -> bool`, so `Card.IsX` methods pass directly; extra filter arguments become
  closure captures.
- A Lua closure an effect keeps (procedure parameters, delayed operations) is a per-card type implementing the
  library's trait (`SynchroProcedure`, `XyzProcedure`, `FusionMix`, `RitualParams`, `EquipProcedure`,
  `aux::DelayedOperation`, ...); nil parameters are `Option` consts or the trait's defaults. Values a closure
  captures go into the effect's labels or label object. Functions shared by several codes use const generics.
- Lua state kept across calls (`s.x = ...`, tables) is per-duel script state (`get_script_state`/
  `set_script_state`); never a Rust `static`.
- `s.listed_names` is `pub const LISTED_NAMES: &[u32]`, `mt.fit_monster` is `pub const FIT_MONSTER: &[u32]`.
  `aux.Stringid(id, n)` is `aux::stringid(ID, n)`. A helper another card's file already has is reused through
  `super::c<code>::name`.
- Every name of `constant.lua`, `card_counter_constants.lua` and `archetype_setcode_constants.lua` is in the prelude
  with its Lua name and the type the API takes (`WIN_REASON_NUMBER_iC1000` is upper-cased); never add constants.
- Cards use `set_operation`; `set_promptless_operation` is for library code that provably cannot prompt.
- `e:SetHintTiming(x)` with one argument sets both timings: `e.set_hint_timing(duel, x, x)`.
- A core function a library replaces keeps a `_core` suffix in the engine; the scripts-crate trait takes the Lua name.
- An internal engine function whose name a new Lua mirror needs is renamed with a `field_` prefix (`field_raise_event`).

## Fidelity

- Match EDOPro exactly, misrulings and Lua bugs included (an undefined global is nil, an upvalue the Lua reassigns
  stays reassigned). A mirrored bug gets a one-line comment.
- Port line by line: the same checks, selections, hints, messages and random draws in the same order.
- An optimisation must never skip or add an effect lookup the Lua makes. A conditional effect gets a new id each
  time it becomes available and `filter_effect` orders effects by id, so a skipped or extra lookup renumbers effects
  later; random play rarely shows it. Skip only work that looks nothing up.
- A Lua check that registers and resets effects inside a condition, material filter or value runs through
  `duel.check_mut(|duel| ...)` (see its uses in `proc_synchro.rs` and `proc_link.rs`), only when the Lua would mutate.
  `AssumeProperty` exists for race only. A check that leaves state behind (e.g. a condition setting its effect's
  label) is not supported yet; stop and report such a card.

## Code style

- Rust 2024. No `unwrap`/`expect`/`panic`/indexing outside tests (clippy denies them), never `#[allow(...)]`;
  restructure instead. Errors via `thiserror` where a function can fail for reasons other than a prompt.
- Comments are rare: the card name, a path left out and why, a mirrored oracle bug, a non-obvious `SUPPORT`.
  One line each. No commented-out code.
- Minimal code: no copy-pasted blocks; share a helper rather than repeat it.
