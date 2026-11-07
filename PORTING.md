# Porting a card

Port `vendor/CardScripts/official/c<code>.lua` to `crates/scripts/src/official/c<code>.rs` so that the engine matches
the oracle (ocgcore running the Lua) message for message. The file is all there is to add: `crates/scripts/build.rs`
registers every `c<code>.rs` and reads its `LISTED_NAMES`, `FIT_MONSTER`, `SUPPORT` and `CHECKS`. The order of work is
`porting/queue.tsv` (TCG Advanced pool, most viewed first; `ported` is `no script` for cards that need no port).
`vendor/` is read-only.

## Workflow

1. Read the Lua and every library function it calls (`utility.lua`, `cards_specific_functions.lua`, `proc_*.lua`;
   grep all of `vendor/CardScripts/*.lua` before taking a method for missing: `chain.lua` and others define some, and
   a Lua error is not reported as such, only as a diff). `grep -rn "unported!" crates/engine/src` lists the mechanisms
   still missing; one a card reaches is ported like any other core function.
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
   be given at once; cargo's own flags (`-j 7`) go before `--`. Then run `cargo test --release -p diff -j 7` (all suites must stay clean),
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
messages from there (`MSG_<type> [bytes]`; `--context N` shows N of each, default 5). Other failing scenarios are listed
by index. Each check lists the first scenarios that satisfy it; `--trace K` writes what scenario K places and its whole
message stream (up to the divergence, for a failing one), to see why an effect never resolves. A code without a file says `not ported`. The exit code is 1 on any failure. A scenario depends only on
(code, index), so a failure reproduces exactly; ports of other cards never change it.

## Test metadata

Each scenario places the card for a random player in a random zone its type can hold, its `SUPPORT` cards, and up to
three cards from a fixed set (the Elfnote deck and the classic cards), then random-plays Elfnote mirror decks.

- `pub const SUPPORT: &[(u32, u16)]`: (code, location) of cards the effects need (costs, targets, materials, a second
  copy), with locations from `crate::testing` (`HAND`, `DECK`, `MZONE`, `SZONE`, `GRAVE`, `REMOVED`, `EXTRA`, or 0
  for a random zone), `| OPPONENT` for the other player. Only ported cards and Normal Monsters (which have no
  script); one comment line naming them.
  An unported card in SUPPORT runs its Lua in the oracle but nothing in the engine, so the diff fails: port a
  needed support card first (it then has its own checks).
  Pendulum Normal Monsters have a script (`Pendulum.AddProcedure`), so they need a port too.
  Before choosing SUPPORT, read the Lua's `listed_names`, material codes and searched names: port those cards
  first (an unported card in the Deck shows up as an unexplained `MSG_SELECT_CHAIN` mismatch).
- `pub const CHECKS: &[Check]` (`crate::testing::Check`), each needing a count above 0 in both the first 13 and all
  64 scenarios. Without `CHECKS` the card must resolve some effect (`Check::Resolves`). Others:
  `ResolvesEffect(aux::stringid(ID, n))`, `SpecialSummoned`, `SpecialSummonedCode(code)` (a token the card creates),
  `RitualMaterial`, `MaterialFrom(location)`, `CardHint(description)`, `Message(MSG_*)` (from `engine::constants`; the
  message must be about the card or about no card), `TributeSummoned(n)`, `TributeSet(n)`, `Replaced`, and
  `Report(&check)`, which only writes the count out. Check each effect that needs a specific path; use another check
  than `Resolves` only for a card that has no effect a scenario can resolve, and say why in one comment line.
- If a check fails at 13 but passes at 64, improve `SUPPORT` (place what the effect needs where it can be used).
  A second copy in the Extra Deck makes a Synchro/Xyz/Link Summon of the card far likelier. Normal Monsters by race
  and level: query `vendor/BabelCDB/cards.cdb` (`datas.type & 0x11 = 0x11`, minus Pendulums), keeping
  only codes `porting/queue.tsv` marks `no script` (the cdb also has tokens and cards outside the pool). A ported
  Effect Monster can stand in where no Normal Monster of a level exists.
  An effect that needs the card in one zone gets a copy there (`(ID, SZONE)`), or more copies of what it reacts to.
  `SZONE` never places a Field Spell (that is the Field Zone); use 0. Placement draws random numbers in list order, so
  reordering `SUPPORT` changes every scenario.
  `DECK` cards are often drawn into the opening hand: list 2-3 copies of what an effect needs in the Deck.
  `| OPPONENT` is relative to the owner of the card under test. A card immune while a copy is in the GY never sees
  removal: put the second copy in the Extra Deck instead.
- `ResolvesEffect` and `Message` count by code or by any card: a `SUPPORT` copy of the card can satisfy them, and
  `Message(MSG_ATTACK)` (`use engine::constants::MSG_ATTACK`) counts every attack, the usual check of a card with only
  continuous effects. A battle-damage trigger of a high-ATK card needs few other attackers in `SUPPORT`, or the duel
  ends first. Check that the counted path is the card's own. An effect a card grants
  another (an Xyz material's effect) chains as that other card, so check it another way (`MaterialFrom`, `CardHint`).
  `CardHint` counts by the hinted card, so a hint on another card never counts. `SZONE` often sets a Trap face-down:
  a cost needing several face-up Traps needs five or more copies.
- `Report` is only for a path no `SUPPORT` can reach (say why in one comment line), never for a reachable one that
  is merely rare. An effect needing an empty hand is one: the opening hand is always drawn.
- `SpecialSummoned` counts the card under test being summoned; what a card summons is `SpecialSummonedCode`.
- `mt.material` is `Fusion::material(duel, code)`, recorded by the `Fusion.AddProcMix*` ports.
- A rare Synchro/Xyz/Link path in the first 13 scenarios usually needs three Extra Deck copies and materials of each
  level. There are no Normal Monsters above Level 8: use ported Extra Deck monsters as higher-level materials.
  A Spell that Special Summons another card is checked by `SpecialSummonedCode(code)`.
  A trigger on its own destruction needs a destroyer that can hit the card (Raigeki only hits the opponent; Dark Hole
  also clears what the trigger needs, so a single-target destroyer is best). An effect without `SetDescription` is
  `ResolvesEffect(0)`. A Link whose effect needs its materials to qualify gets only qualifying materials. Test several cards per card command run: each rebuild takes over a minute.

## Mirror API

- `Duel.X(...)` is `duel.x(...)`; `c:X(...)`, `e:X(...)`, `g:X(...)` are methods on `CardId`, `EffectId`,
  `CardSet` taking the duel first: `c.is_type(duel, TYPE_SYNCHRO)`. `Effect.CreateEffect(c)` is
  `duel.create_effect(c)`, `e:Clone()` is `e.clone_effect(duel)`, `#g` is `g.len()`. Names are the Lua names in
  snake case.
- Optional trailing arguments: the plain name takes the arguments most calls pass; `_full` takes all of them (add it where a mirror lacks it). Call
  `_full` exactly when the Lua call passes one of the optional ones, with the Lua default for any it leaves out
  (`c.is_type_full(duel, TYPE_TUNER, Some(sc), sumtype, tp)`). Lua nil is `None` for cards and groups and
  `PLAYER_NONE` for a player.
  A nil count is the C++ default, not 1: `Duel.IsPlayerCanDraw(tp)` checks a count of 0. Some Lua helpers shift their
  arguments by type: `Duel.CheckReleaseGroupCost(tp,f,1,true,...)` passes `true` as `use_hand`, so it is min 1, max 1.
- Functions that can prompt or yield return `Script<T>`; propagate with `?`. A target whose Lua takes no `chkc`
  ignores it (no early return on `chkc.is_some()`). An argument that reads the duel is computed before a call that
  takes `&mut Duel` (`let turn = duel.get_turn_count(); e1.set_label(duel, turn)`). A target or cost returns `Ok(chk0)` for
  chk 0 and `Ok(false)` after doing its work, where the Lua returns nothing.
- `aux.SelectUnselectGroup` is `aux::select_unselect_group(duel, g, minc, maxc, &|duel, sg, mg, c| (res, stop),
  seltp, hintmsg)`; `Duel.SelectTarget` returns `Option<CardSet>`; a card calling `Fusion.SummonEffTG/OP` itself builds
  the closure first with `Fusion::new_summon_eff_closure(duel, e, op)`.
- Filters are `Fn(CardId, &Duel) -> bool`, so `Card.IsX` methods pass directly; extra filter arguments become
  closure captures.
- A Lua closure an effect keeps (procedure parameters, delayed operations) is a per-card type implementing the
  library's trait (`SynchroProcedure`, `XyzProcedure`, `FusionMix`, `RitualParams`, `EquipProcedure`,
  `aux::DelayedOperation`, ...); nil parameters are `Option` consts or the trait's defaults. Values a closure
  captures go into the effect's labels or label object. Functions shared by several codes use const generics.
  A captured card travels as a one-card group in the label object.
- Lua state kept across calls (`s.x = ...`, tables) is per-duel script state (`get_script_state`/
  `set_script_state`); never a Rust `static`. `pairs(t)` over a table keyed by card codes, where the order shows (a
  filter or message built from it), visits `lua::pairs(codes in insertion order)`.
- `Duel.IsPlayerCanSpecialSummonMonster(p, code, setcode, type, atk, def, lv, race, att)` is
  `is_player_can_special_summon_monster(p, code, MonsterStats::new(...))`, with the struct's fields for nil arguments.
  It takes `&mut Duel` (the core's temp card takes the stats), so a condition calls it through `check_mut`.
  So does any other `&mut Duel` mirror a condition or filter needs (`can_chain_attack`, see c70405001). A path where the
  oracle's Lua errors (a nil index, an undefined function) is `unported!`; an erroring `chkc` check is `false`.
- `s.listed_names` is `pub const LISTED_NAMES: &[u32]`, `mt.fit_monster` is `pub const FIT_MONSTER: &[u32]`,
  `s.material_setcode` is `pub const MATERIAL_SETCODE: &[u16]` (read by `ListsArchetypeAsMaterial`),
  `s.dark_calling` is `pub const DARK_CALLING: bool = true` (read through `official::dark_calling(code)`).
  `aux.Stringid(id, n)` is `aux::stringid(ID, n)`. A helper another card's file already has is reused through
  `super::c<code>::name`.
- Every name of `constant.lua`, `card_counter_constants.lua` and `archetype_setcode_constants.lua` is in the prelude
  with its Lua name and the type the API takes (`WIN_REASON_NUMBER_iC1000` is upper-cased); never add constants.
- Cards use `set_operation`; `set_promptless_operation` is for library code that provably cannot prompt.
- Effect function shapes are the variants of `Value`, `Target`, `Condition`, `Cost` and `Operation` in
  `crates/engine/src/effect.rs`; reuse one before adding a variant for an unusual signature.
- `e:SetHintTiming(x)` with one argument sets both timings: `e.set_hint_timing(duel, x, x)`.
- A core function a library replaces keeps a `_core` suffix in the engine; the scripts-crate trait takes the Lua name.
- An internal engine function whose name a new Lua mirror needs is renamed with a `field_` prefix (`field_raise_event`).
- A new `Duel` field must also go into the `copy_from` destructure in `script.rs` (snapshot restore).

## Fidelity

- Match EDOPro exactly, misrulings and Lua bugs included (an undefined global is nil, an upvalue the Lua reassigns
  stays reassigned). A mirrored bug gets a one-line comment.
- Port line by line: the same checks, selections, hints, messages and random draws in the same order.
- A boolean argument the Lua passes as nil is false; the core's default (`lua_get<bool, true>`) applies only to an
  absent argument, so a library wrapper forwarding a nil parameter passes false.
- Lua nil for a numeric argument is 0 (`Duel.Remove(c, nil, r)` is position 0, not `POS_FACEUP`). The core
  rounds Lua numbers it reads as integers (`atk/2` becomes `round(atk/2)`). `Duel.IsChainNegatable` is always true
  in the core; the real check is `field_is_chain_negatable`, reached by `NegateActivation`.
- An optimisation must never skip or add an effect lookup the Lua makes. A conditional effect gets a new id each
  time it becomes available and `filter_effect` orders effects by id, so a skipped or extra lookup renumbers effects
  later; random play rarely shows it. Skip only work that looks nothing up.
- A Lua check that registers and resets effects inside a condition, material filter or value runs through
  `duel.check_mut(|duel| ...)` (see its uses in `proc_synchro.rs` and `proc_link.rs`), only when the Lua would mutate.
  `AssumeProperty` exists for race only.
- A condition that writes state the Lua reads later (its effect's label or label object, `s.x`, a flag effect) is
  set with `e.set_condition_mut(duel, f)`, `f` taking `&mut Duel`, and writes as an operation does. This holds for
  the conditions of activated and continuous effects (`EFFECT_TYPE_ACTIONS`), which the engine reaches with
  `&mut Duel`. A writing condition of another effect (a field or single effect, a summon procedure) or a value, target
  or filter that writes where its shape takes `&Duel` is not supported yet; stop and report such a card.

## Code style

- Rust 2024. No `unwrap`/`expect`/`panic`/indexing outside tests (clippy denies them), never `#[allow(...)]`;
  restructure instead. Errors via `thiserror` where a function can fail for reasons other than a prompt.
- Comments are rare: the card name, a path left out and why, a mirrored oracle bug, a non-obvious `SUPPORT`.
  One line each. No commented-out code.
- Minimal code: no copy-pasted blocks; share a helper rather than repeat it.
