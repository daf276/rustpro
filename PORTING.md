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
   first: it overrides some core functions (`Duel.IsMainPhase(player)` also checks whose turn it is), also for the
   library procedures that call them (`Duel.Overlay` raises `EVENT_MOVE`, so the Xyz procedure uses
   `DuelWorkaround::overlay`).
4. Declare `SUPPORT` and, if the default does not fit, `CHECKS` (below).
5. Run `cargo run --profile card -p diff --bin card -- <code> --threads 7` and iterate until it passes (the `card`
   profile is release with incremental builds: a card edit rebuilds in seconds). Several codes may be given at once;
   cargo's own flags (`-j 7`) go before `--`. Then run `cargo test --profile card -p diff -j 7` (all suites must stay clean),
   `cargo clippy --workspace --all-targets -j 7`, `cargo fmt --all` and
   `rustfmt --edition 2024 crates/scripts/src/official/c<code>.rs` (`cargo fmt` does not reach card files, since
   build.rs includes them).
   The card command and the diff suites cache the oracle (a hash of each run's stream in `target/oracle-cache`) and
   replay a cached run in the engine alone; `ORACLE_CACHE=off` disables it, `ORACLE_CACHE=<dir>` moves it.

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
  and level: query `vendor/BabelCDB/cards.cdb` with Python's `sqlite3` (`C:\Users\Attila\AppData\Local\Programs\Python\Python310\python.exe -I` where `python` is the Store stub; `datas.type & 0x11 = 0x11`, minus Pendulums), keeping
  only codes `porting/queue.tsv` marks `no script` (the cdb also has tokens and cards outside the pool). A ported
  Effect Monster can stand in where no Normal Monster of a level exists.
  An effect that needs the card in one zone gets a copy there (`(ID, SZONE)`), or more copies of what it reacts to.
  `SZONE` never places a Field Spell (that is the Field Zone); use 0, or `HAND` for one that must be activated.
  `SZONE` never places a monster either: a Pendulum Monster goes in a Pendulum Zone with `PZONE`. A copied effect resolving counts on the copying card as `ResolvesEffect(aux::stringid(copied_code, n))`. Placement draws random numbers in list order, so
  reordering `SUPPORT` changes every scenario.
  `DECK` cards are often drawn into the opening hand: list 2-3 copies of what an effect needs in the Deck (4 when it
  needs two left there). An `(ID, MZONE)` copy defeats a "only monster you control" condition. A destroyed-trigger
  fires far more often from the opponent's `Raigeki` / `Dark Hole` (`| OPPONENT`) than from your own hand.
  `| OPPONENT` is relative to the owner of the card under test. A card immune while a copy is in the GY never sees
  removal: put the second copy in the Extra Deck instead.
- Race values are the `RACE_*` constants (`RACE_DINOSAUR` is 0x10000, `RACE_BEAST` 0x4000); look them up, do not guess.
  `ResolvesEffect(0)` is also met by any `SUPPORT` card's undescribed effect (Raigeki's).
- `ResolvesEffect` and `Message` count by code or by any card: a `SUPPORT` copy of the card can satisfy them, and
  `Message(MSG_ATTACK)` (`use engine::constants::MSG_ATTACK`) counts every attack, the usual check of a card with only
  continuous effects. A battle-damage trigger of a high-ATK card needs few other attackers in `SUPPORT`, or the duel
  ends first. A destroyed-by-battle trigger needs 2-3 copies and strong `MZONE | OPPONENT` monsters. Check that the counted path is the card's own. An effect a card grants
  another (an Xyz material's effect, an equip or `EFFECT_TYPE_GRANT` effect) chains as that other card, so check it
  another way (`MaterialFrom`, `CardHint`).
  `CardHint` counts by the hinted card, so a hint on another card never counts (a BE_MATERIAL effect hinting the
  summoned monster is one). `SZONE` often sets a Trap face-down:
  a cost needing several face-up Traps needs five or more copies.
- `Report` is only for a path no `SUPPORT` can reach (say why in one comment line), never for a reachable one that
  is merely rare. An effect needing an empty hand is one: the opening hand is always drawn.
- A continuous effect with an operation forms no chain link, so `ResolvesEffect` never counts it: check what it does
  (`Message(MSG_DAMAGE)` and the like), as `Report` if other cards do the same. `aux.SpElimFilter(c, true)` with a nil
  third argument accepts only Graveyard monsters.
- The card under test is placed besides its own `SUPPORT` copies: a `SetUniqueOnField` card gets none there.
- A Link Monster `SUPPORT` places never sits in an Extra Monster Zone, so effects on the zones it points to need a
  real Link Summon.
- `(code, DECK)` or `(code, HAND)` for an Extra Deck monster and `(code, EXTRA)` for a Main Deck monster are silently
  dropped. An
  exact-count `aux.SelectUnselectGroup` can stall the random player past the decision limit ("no MSG_WIN within 5000
  decisions"); the Extra Deck size changes the odds (Gordian Slicer pads it to 17).
  An effect that needs an attack on a specific face-up
  Attack Position monster is rarely reached by random play; it may be `Report`. Print cdb names with `python -I -X utf8`.
- A monster `SUPPORT` places in `MZONE` was not summoned from the Extra Deck: a condition on that needs the support
  to really summon it (the Extra Deck monster with Polymerization, or a Link with its materials).
- `SpecialSummoned` counts the card under test being summoned; what a card summons is `SpecialSummonedCode`.
- `Replaced` counts the card under test's own replaced move (a destruction replaced, or a `REASON_REPLACE` move such as
  a cost-replace banish): an effect that replaces another card's destruction is
  checked from that card's file (with the replacing card in its `SUPPORT`). A card replacing its own destruction without
  moving is not counted: check its prompt with `Message(MSG_SELECT_EFFECTYN)` (c71100270).
- A battle-timed effect (`EVENT_PRE_DAMAGE_CALCULATE` and the like) needs several copies and materials-proof support,
  since random play uses monsters as Link/Xyz materials and activates hand Spells early; Polymerization stays in hand
  for a reveal or discard cost. A card whose only target is an unported archetype member is `Report`.
- `mt.material` is `Fusion::material(duel, code)`, recorded by the `Fusion.AddProcMix*` ports.
- An Xyz Summon in `SUPPORT`: Xyz Import (14602126) in hand with Dragon Rebellion (16195942) and Sangan (26202165) on
  the opponent's side (see c1621413).
- A rare Synchro/Xyz/Link path in the first 13 scenarios usually needs three Extra Deck copies and materials of each
  level. Two of each material on the field (Ash Blossom, Level 3, with Shapesnatch, Level 5, for Level 8) works; one pair
  gave 1 Synchro Summon in 64. An attack-announce trigger needs an opponent's monster to attack into (Sangan). There are no Normal Monsters above Level 8: use ported Extra Deck monsters as higher-level materials.
  A Spell that Special Summons another card is checked by `SpecialSummonedCode(code)`.
  A trigger on its own destruction needs a destroyer that can hit the card (Raigeki only hits the opponent; Dark Hole
  also clears what the trigger needs, so a single-target destroyer is best). An effect without `SetDescription` is
  `ResolvesEffect(0)`, and a `SetDescription` the Lua does not have shows up as a `MSG_SELECT_IDLECMD` or
  `MSG_SELECT_CHAIN` diff. `EFFECT_EXTRA_FUSION_MATERIAL` (a card letting the player use extra Fusion materials, such as
  Lunalight Masquerade) is unported: skip such cards. A Link whose effect needs its materials to qualify gets only qualifying materials. `cargo check -p scripts -j 7` finds compile errors first. Never kill a
  build midway (`timeout` on `cargo run`): it corrupts the incremental cache (hundreds of `LNK2019`); delete
  `target/card/incremental/scripts-*` and `diff-*` to recover.
  Scenario Xyz materials come only from the fixed set: a card needing a specific material gets it attached through a
  ported `SUPPORT` card (Xyz Import, c1621413). An effect reacting to the opponent targeting a monster gets a targeting
  card `| OPPONENT` in `SUPPORT` (Mage Power, c49161188). A card an effect summons or adds by name is ported first even
  when the batch does not list it.

## Mirror API

- `Duel.X(...)` is `duel.x(...)`; `c:X(...)`, `e:X(...)`, `g:X(...)` are methods on `CardId`, `EffectId`,
  `CardSet` taking the duel first: `c.is_type(duel, TYPE_SYNCHRO)`. `Effect.CreateEffect(c)` is
  `duel.create_effect(c)`, `e:Clone()` is `e.clone_effect(duel)`, `#g` is `g.len()`. Names are the Lua names in
  snake case.
- Optional trailing arguments: the plain name takes the arguments most calls pass; `_full` takes all of them (add it where a mirror lacks it). Call
  `_full` exactly when the Lua call passes one of the optional ones, with the Lua default for any it leaves out (`Card.IsCanBeXyzMaterial(c, tp)` defaults to `REASON_XYZ | REASON_MATERIAL`; one-argument `e:SetHintTiming(x)` sets both timings; a missed one shows as
  `spe_count` in `MSG_SELECT_CHAIN`)
  (`c.is_type_full(duel, TYPE_TUNER, Some(sc), sumtype, tp)`). Lua nil is `None` for cards and groups and
  `PLAYER_NONE` for a player.
  A nil count is the C++ default, not 1: `Duel.IsPlayerCanDraw(tp)` checks a count of 0. Some Lua helpers shift their
  arguments by type: `Duel.CheckReleaseGroupCost(tp,f,1,true,...)` passes `true` as `use_hand`, so it is min 1, max 1.
  Port the Lua's own argument slips as the core reads them (`scriptlib.h` `lua_get`): a number in a card or group slot
  is nil, so `GetMatchingGroup(Card.IsAbleToRemove,tp,0,LOCATION_ONFIELD,1,handler,tp,POS_FACEDOWN)` checks player
  1 with the default position and reason `tp` (c40080312). In `Duel.SendtoGrave(c,REASON_RULE,nil,PLAYER_NONE)` the
  `PLAYER_NONE` is the reason player: `send_to_grave_full(c, REASON_RULE, PLAYER_NONE, PLAYER_NONE)`.
- Functions that can prompt or yield return `Script<T>`; propagate with `?`. A target whose Lua takes no `chkc`
  ignores it (no early return on `chkc.is_some()`). An argument that reads the duel is computed before a call that
  takes `&mut Duel` (`let turn = duel.get_turn_count(); e1.set_label(duel, turn)`). A target or cost returns `Ok(chk0)` for
  chk 0 and `Ok(false)` after doing its work, where the Lua returns nothing.
- `aux.SelectUnselectGroup` is `aux::select_unselect_group(duel, g, minc, maxc, &|duel, sg, mg, c| (res, stop),
  seltp, hintmsg)`, a nil `rescon` is `&|_, _, _, _| (true, false)`; `Duel.SelectTarget` returns `Option<CardSet>`; a card calling `Fusion.SummonEffTG/OP` itself builds
  the closure first with `Fusion::new_summon_eff_closure(duel, e, op)`.
- `Cost::and` cannot stop on a cost that returns false when run (Lua's `Cost.AND` does): write such a composite
  cost out (`DragonRuler::self_discard_to_grave_cost`). `EffectUtility::has_self_discard_cost` lists the costs it knows.
- A value that resets its own effect (`e:Reset()` inside the value) is `Value::ReasonMut`; messages a check writes
  through `check_mut` stay in the trace, where the core writes them.
- In the engine, `unported!(duel, x)` takes what the core would have used, an iterator or `Option`:
  `(cond).then_some(effect)` for a condition; in card files it takes a bool. Trap Monsters are ported except turning one
  face-down.
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
  A key is unique per card (`ID << 16 | slot`); a Lua global shared by many scripts gets one `pub const` key in
  the first card that uses it (`_replace_count` is `c36623431::REPLACE_COUNT`).
- `Duel.IsPlayerCanSpecialSummonMonster(p, code, setcode, type, atk, def, lv, race, att)` is
  `is_player_can_special_summon_monster(p, code, MonsterStats::new(...))`, with the struct's fields for nil arguments.
  It takes `&mut Duel` (the core's temp card takes the stats), so a condition calls it through `check_mut`.
  So does any other `&mut Duel` mirror a condition or filter needs (`can_chain_attack`, see c70405001). A path where the
  oracle's Lua errors (a nil index, an undefined function) is `unported!`; an erroring `chkc` check is `false`.
- `s.listed_names` is `pub const LISTED_NAMES: &[u32]`, `mt.fit_monster` is `pub const FIT_MONSTER: &[u32]`,
  `s.material_setcode` is `pub const MATERIAL_SETCODE: &[u16]` (read by `ListsArchetypeAsMaterial`),
  `s.dark_calling` is `pub const DARK_CALLING: bool = true` (read through `official::dark_calling(code)`),
  `s.xyz_number` is `pub const XYZ_NUMBER: u32` (`official::xyz_number(code)`), and a Synchro, Xyz or Link
  Monster's `s.material` is `pub const MATERIAL: &[u32]` (`official::material(code)`).
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
- LF line endings. On Windows a Python script's `open(path, "w")` writes CRLF: pass `newline=""` or edit with the
  edit tools.
