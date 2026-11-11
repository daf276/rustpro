# Refactoring survey

Status: done on 2026-11-11 except E28, see PLAN.md phase 7 for what each step changed. Sections 1.2 and 1.3 (effects
registered inside operations, repeated condition closures) were not done: 1.2 needs each block compared by hand.

Survey of 2026-11-11 at 550ce80: what can be extracted or merged, measured against the global guidelines (as little
code as possible, no copy-paste, existing utilities first, comments rare). Nothing was changed. Line counts are
estimates; locations were read or grepped, and the items marked "checked" I verified myself after the reviewers.

## Summary

| Area | Lines | Could go | Kind of work |
| --- | --- | --- | --- |
| Card files (2,160) | 214,000 | 8,000 - 12,000 | A small set of stock targets, operations and costs; needs a decision (section 1) |
| Engine (`crates/engine`) | 30,000 | about 1,700 | Helpers for repeated sequences, two macros, `Default` derives, dead code |
| Script libraries (`utility.rs`, `proc_*.rs`) | 9,900 | about 260 | Helpers; most repetition mirrors the Lua |
| Harness, tests, tools | 6,300 | about 240 | Shared loading and test helpers, dead FFI items |
| Comments, all non-card code | 1,720 comment lines (3.7%) | 600 - 900 | Delete restating and multi-line comments |

The card files are where the volume is: 23 function shapes account for 1,381 functions and 11,700 lines. Everything
else together is about 2,200 lines of code plus the comments.

The guidelines and PORTING.md pull in opposite directions here. PORTING.md says "port line by line" with "the
script's functions in the Lua's order and names", and the Lua and the C++ repeat themselves heavily; most of what
follows exists because the port copied that repetition faithfully. Each item says whether it leaves the C++ or Lua
shape. Nothing below changes behaviour, message order, random draws or effect-lookup order unless it says so.

## 1. Card files

Measured over `crates/scripts/src/official/c*.rs`: 48,600 lines of `initial_effect`, 105,800 lines of other
functions, 5,400 comment lines.

### 1.1 Stock targets, operations and costs (8,000 - 12,000 lines)

Functions that are identical once constants, numbers, the filter's name and the `tp`/`e` aliases are blanked:

| Families of | Shapes | Functions | Lines |
| --- | --- | --- | --- |
| 20+ members | 23 | 1,381 | 11,739 |
| 10+ members | 49 | 1,714 | 14,962 |
| 5+ members | 108 | 2,081 | 17,974 |

The largest (members x lines each, first example):

| Shape | Members |
| --- | --- |
| Target: `is_existing_matching_card(filter, ..)` then `set_operation_info` (`c10019086::tgtg`) | 333 x 8 |
| Operation: hint, select from a location, `send_to_hand`, `confirm_cards` (`c1035143::thop`) | 210 x 10 |
| Target: zone free and a summonable card exists (`c10209545::sptg`) | 104 x 9 |
| Operation: zone free, hint, select, `special_summon` (`c10209545::spop`) | 77 x 12 |
| Operation: Special Summon the handler if still related (`c10673071::spop`) | 79 + 28 x 8 |
| Target: zone free and the handler can be summoned (`c12213463::sstg`) | 35 + 34 + 18 + 13 x 9 |
| Operation: destroy the first target if related (`c10000010::desop`) | 58 x 8 |
| Operation: select and `send_to_grave` (`c10019086::tgop`) | 50 x 9 |
| Operation: draw or damage from the chain info (`c10189126::operation`, `c102380::damop`) | 49 + 24 x 6 |
| Target: `is_player_can_draw`, target player and param (`c11765832::drtg`) | 42 x 10 |
| Operation: first target to hand / Special Summon it (`c10426067::thop`, `c16237004::spop`) | 37 + 35 x 8 |
| Cost: select and send to GY / banish / discard (`c17259470::spcost`, `c10808715::spcost`, `c13117073::regcost`) | 27 + 22 + 19 + 15 x 8-10 |

Sketch, in a new `crates/scripts/src/stock.rs` (the setters take plain `fn` pointers, and a closure that captures
nothing coerces to one, so no engine change is needed):

```rust
pub fn exists_target(duel: &mut Duel, a: &Args, chk: i32, filter: impl Fn(CardId, &Duel) -> bool, location: u16, category: u32) -> Script<bool>
pub fn search(duel: &mut Duel, a: &Args, filter: impl Fn(CardId, &Duel) -> bool, location: u16) -> Script<()>

// in the card file, keeping the Lua's function names:
fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}
fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
```

- Keeping the named wrapper (3 lines instead of 8-12) saves about 7,600 lines for the 23 largest shapes and about
  9,800 for the 49. Passing the helper as a closure in `initial_effect` instead saves about 2,000 more but drops the
  Lua names.
- The migration is mechanical: the functions are textually identical apart from the blanked parts, so a script can
  rewrite them, and the full diff suite (cached oracle) verifies it.
- Risk: none to parity if each helper is the exact sequence. The helper list must stay small and exact; a member
  that differs by one call (an extra `break_effect`, a different check order) stays written out.
- Leaves the Lua shape: yes. The Lua writes each one out. A port is then reviewed as "this Lua function is the stock
  search" rather than line against line. PORTING.md and the porting brief need a list of the stock shapes.
- Recommendation: do the 23 shapes with 20+ members and stop there. Below that the helpers multiply faster than the
  lines they save.

### 1.2 Effects registered inside operations (1,500 - 2,000 lines)

986 effects are created outside `initial_effect`, in 7,992 lines. The largest groups by effect code:

| Code | Blocks | Lines | Most common shape |
| --- | --- | --- | --- |
| `EFFECT_CANNOT_SPECIAL_SUMMON` (the "rest of this turn" lock) | 133 | 1,339 | 39 identical setter lists |
| `EFFECT_UPDATE_ATTACK` | 117 | 759 | 55 x type, code, value, reset |
| `EFFECT_SET_ATTACK_FINAL`, `EFFECT_UPDATE_LEVEL`, `EFFECT_CHANGE_LEVEL`, `EFFECT_CHANGE_TYPE` | 95 | 634 | type, (property,) code, value, reset |
| `EFFECT_LEAVE_FIELD_REDIRECT` (banish when it leaves) | 29 | 232 | 19 identical |
| `EFFECT_CANNOT_ATTACK`, `EFFECT_DISABLE`, `EFFECT_EQUIP_LIMIT` | 97 | 677 | 11 - 13 identical each |

- `CardUtility::update_attack` already exists in `utility.rs` (it ports `Card.UpdateAttack`), but these cards'
  Lua builds the effect by hand, so the port does too. A helper is only exact where the setter order and the reset
  match; setter order is not observable, the created effect is.
- Sketch: `aux::spsummon_lock(duel, c, tp, description, target, reset)`, `c.temporary_stat(duel, code, value,
  property, reset)`, `aux::banish_on_leave(duel, c, description)`.
- Risk: low. Each block must be compared for the reset and property before it is folded in.
- Leaves the Lua shape: yes.

### 1.3 Conditions written as the same closure (clarity, about 300 sites)

`|duel, _| duel.is_main_phase()` (53), `|duel, a| a.e.get_handler(duel).is_synchro_summoned(duel)` (47),
`is_turn_player(1 - a.tp)` (39), `is_turn_player(a.tp)` (38), `is_link_summoned` (31), `is_fusion_summoned` (27),
`is_previous_location(duel, LOCATION_ONFIELD)` (21), `is_xyz_summoned` (20). Named functions in `aux`
(`aux::synchro_summoned`, `aux::own_turn`) would shorten lines, not remove them. Marginal; only worth doing together
with 1.1.

### 1.4 Checked and left alone

- **Registration blocks in `initial_effect`** (4,758 blocks, 46,161 lines, 1,597 distinct shapes): one setter per
  line, matching the Lua line for line. A builder saves characters, not lines, and the commonest exact shape covers
  only 164 blocks.
- **Bare codes in `SUPPORT`** (6,924 uses of 1,064 codes against 1,288 named constants): the top 30 codes cover only
  37% (Sangan 610, Raigeki 290, Elfnote Lucina 229, Dark Hole 217). Names in `crate::testing` for those four or five
  would help reading; the tail stays numeric with its comment line.
- **Comments** (2.5%): 2,160 are the card name and 3,005 the `SUPPORT` rationale, both required by PORTING.md; only
  220 others.

## 2. Engine

Ordered by what I would do first. "C++ repeats" means the helper leaves the side-by-side layout with ocgcore; the
port already does that in places (`activate_from_command`, `battle_window`, the merged `get_scale`).

### 2.1 No judgement needed

| # | Finding | Where | Saves |
| --- | --- | --- | --- |
| E1 | `Processor::new` lists 147 lines of fields; 4 are not the default. Derive `Default`, write `..Self::default()` (checked) | `field.rs:484-630` | 135 |
| E2 | Same for `Effect::new` (38 fields, 2 not default) | `effect.rs:248-295` | 38 |
| E3 | Units listed three times: the enum, `units!`, and 66 dispatch arms `Unit::X(arg) => self.run_steps(arg, Self::process_x, bool)`. One table macro generates all three plus `Unit::step()`, which also removes 17 `if arg.step > 0` in `action.rs` (checked) | `processor.rs:280-349, 372-440, 461-526`, `action.rs:204-261` | 150 |
| E4 | Dead mirror functions: 8 never called (`get_coin_result`, `get_dice_result`, `set_coin_result`, `set_dice_result`, `enable_gemini_status`, `mset`, `is_msetable`, `is_battle_phase_full`) and 17 `_full` variants whose only caller is their plain twin (`synchro_summon_full`, `xyz_summon_full`, `toss_dice_full`, `release_full`, ...). Delete; add `_full` back when a card needs it, as PORTING.md says (checked a sample) | `libduel.rs`, `libcard.rs`, `libeffect.rs`, `proc_workaround.rs`, `utility.rs:713` | 130 |
| E5 | `send_to(.., LOCATION_GRAVE, 0, POS_FACEUP as u8, false)` written out at 15 sites, 11 of them 11 lines each: add `send_to_grave(targets, reason_effect, reason, reason_player, playerid)` (checked the count) | `operations.rs:1035, 1086, 2452, 2946, 4158, 4298, 4374, 4562, 5408, 5441`, `chain.rs:1269`, + 4 one-liners | 100 |
| E6 | The Lua-facing `send_to_*` fronts each repeat reason effect + 9-argument `send_to` + yield + read the count; 31 sites end in `yield_script()?; Ok(returns.at_i32(0))`. Add `yield_count`, `yield_success`, `send_cards` | `libduel.rs:966, 987, 1007, 1347, 1373, 1848` and the 31 tails | 100 |
| E7 | `MSG_RETRY; return false` 23 times, 8 of them after `return_cards.clear()`; the skip hint 9 times. Add `accept(valid)`, `reject_cards()`, `skip_prompt(player)` (checked the count) | `playerop.rs` | 55 |
| E8 | `is_activateable(effect, player, event, false, false, false, false, false)` at 19 of 25 call sites; the C++ has these as default arguments. Add `can_activate(effect, player, event)` (checked the count) | `processor.rs` 13, `chain.rs` 3, `operations.rs` 3 | 10, and 19 calls stop wrapping |
| E9 | `copy_from` destructures 34 fields then writes 30 `clone_from` lines. A `copy_fields!` macro keeps the exhaustive destructure, so a new field still fails to compile | `script.rs:272-345` | 40 |
| E10 | `FieldEffect` list bookkeeping (oath, pheff, cheff, rechargeable) identical in two files: `FieldEffect::track` / `untrack` | `card.rs:1293-1307, 1383-1394`, `field.rs:1849-1861, 1901-1912` | 20 |
| E11 | "previous = current" 5-line copy at 6 sites; reason save and set at 6 more; `summon.{location, sequence, pzone}` at 4. Add `save_location`, `save_reason`, `set_reason` beside the existing `restore_reason` | `operations.rs:489, 1243, 2214, 397, 1559, 2690, 2736, 3155`, `field.rs:807, 872, 960` | 40 |
| E12 | Existing helpers not used: `write_move` is private in `field.rs` while `operations.rs` repeats its body 4 times; `hint_selection_full` writes what `set_target_card` and `change_target_card` write by hand; `take_replaced_targets` duplicates the opening of Destroy steps 2 and 12; `filter_effects(card, &[..])` does what `get_summon_codes` and `is_summon_code` do with three calls and a sort | `operations.rs:1254, 2272, 4281, 5502, 1604, 1753`, `libduel.rs:804, 840`, `card.rs:3165, 3198` | 45 |
| E13 | `Vec`s collected only to get around a borrow that is not there (`self.trace` and `self.cards` are separate fields); `set_target_card` allocates on every targeted activation | `libduel.rs:804, 840, 1123, 1979`, `libgroup.rs:145`, `playerop.rs:334`, `operations.rs:1001, 5685` | 20, and allocations |
| E14 | `tests/duel.rs` redefines two constants the engine exports and builds the same `Duel` twice | `crates/engine/tests/duel.rs:16-17, 60-66, 154-161` | 10 |

### 2.2 Same sequence, but the C++ repeats

| # | Finding | Where | Saves |
| --- | --- | --- | --- |
| E15 | Command-window steps shared by PhaseEvent, IdleCommand and BattleCommand: solve the activated chain (3 sites), open a point event (3), end the phase or solve first (2), pass priority (2). Step numbers stay at the call site | `processor.rs:779-791, 1585-1597, 1875-1887, 1682-1692, 1892-1902, 1569-1581, 1811-1823` | 35 |
| E16 | "Activate the chosen chain link" three times; two of them repeat `chain_candidate`'s prefix | `processor.rs:749-770, 1313-1326, 1362-1382` | 25 |
| E17 | Continuous-effect queueing: push + `SolveContinuous` at 7 sites, the same `Chain` literal at 3, the event-player rule at 5 | `chain.rs:187-201, 367, 409-421, 458, 1068-1083, 1204-1210` | 35 |
| E18 | Destroy: steps 1 and 11 differ only in `battle`; the undo triple (`restore_reason`, clear `STATUS_DESTROY_CONFIRMED`, remove) at 7 sites | `operations.rs:1550-1821` | 45 |
| E19 | Summon tails: the negation window at 3 sites, the after-summon point event at 5 (give `point_event_after_summon` a timing parameter) | `operations.rs:3657, 3748, 4914, 2624, 3707, 3973, 5730` | 40 |
| E20 | Summon procedures: building the options (3 sites), picking the chosen effect (3), the target step (2), operation + `dec_count` (4), one `Procedure` literal built three times in SpSummonRule; `summon_full` and `mset_full` differ in one call | `operations.rs:3400-3561, 3831-3927, 4783-4986`, `libduel.rs:2240-2445` | 70 |
| E21 | `negate_chain` and `disable_chain` differ in a flag, a check and a message | `operations.rs:289-366` | 30 |
| E22 | Prompt helpers: hint + SelectCard at 4 sites, hint + SelectPlace at 4; the deck-top hint at 7; the final "operated set" step at 14; raising a single event with the card's own reason at 7 | `operations.rs` (lists in the reviewer notes), `processor.rs:3241`, `field.rs:1638` | 145 |
| E23 | Level, rank and scale share one update loop and one clamp; attribute and race share one add/remove/change fold; summon and set tribute counts differ in one code; "not a monster now" is written 7 times | `card.rs:604-800, 2136-2156, 2904-2955` and `libcard.rs:151, 1338` | 70 |
| E24 | The player-limit target `match` (Card / PlayerCard / SpSummon) 5 times | `field.rs:2872-3027` | 25 |
| E25 | Value dispatch boilerplate at 12 sites; `reach_unported(concat!(file!(), ":", line!()))` written out 13 times (add a bare `unported!(self)` arm) | `effect.rs:1351-1565`, `card.rs:2511-3109` | 30 |
| E26 | `Cost::self_*` and seven more costs share "check on chk 0, else pay": `Cost::two_phase(duel, chk, check, pay)`; the functions stay named, so the `fn_addr_eq` table still works | `utility.rs:1267-1650, 248` | 40 |
| E27 | Small: per-player loops by hand (5 + 2 + 3 sites), `raise_phase_start` (6), `has_new_triggers` (6), the idle-command lists spelled out 4 times, `bits(mask)` in `action.rs` (5 + 3), `Duel::position()` re-walks what `located_cards()` walks, `set_operation_info` twin, `reset_both_turns` (4) and `flag_code` (9), `CardSet::toggle` (3), location-to-pile mapping (6), 18 `impl` blocks that could be 4 | various | 200 |

### 2.3 Needs a benchmark

- **E28, one lazy iterator over a card's effect sources** (Single, Equip, XMaterial, Aura), now written out in
  `filter_effect_into`, `is_affected_by_effect_lookup` and `is_affected_by_effect_target_lookup`
  (`card.rs:1616-1844`). About 50 lines. This is the hottest lookup path, the Aura check order differs per function
  as in the C++ (and `is_available` writes field ids, so the order must stay), and `xmaterial_effects` must stay
  lazy. Only with a before/after benchmark.

## 3. Script libraries

| # | Finding | Where | Saves |
| --- | --- | --- | --- |
| S1 | `build.rs` writes six tables with the same filter-map, format and `writeln`; `official/mod.rs` has six lookups with the same binary search. One loop over a table of (name, constants, type) and one `lookup<T>(table, code)` (checked) | `crates/scripts/build.rs:42-101`, `official/mod.rs:31-66` | 65 |
| S2 | Synchro: `Search` built three ways; the hand-material copy-and-check block twice (the Lua inlines it three times, so the helper also serves Dark Synchro) | `proc_synchro.rs:391-428, 753-763, 1016-1065` | 30 |
| S3 | Xyz and Link: must-be-material group + merge at 5 sites, `Search` literals twice each | `proc_xyz.rs:238-355`, `proc_link.rs:164-247` | 25 |
| S4 | Pendulum: three near-identical `SPSUMMON_PROC_G` effects and two copies of the select loop. Setter order would change (not observable) | `proc_pendulum.rs:27-75, 116-140, 266-290` | 30 |
| S5 | Fusion: the must/gc validation block twice, the CheckExact/Min/Max triple four times | `proc_fusion.rs:382-391, 593-602, 1050-1197` | 16 |
| S6 | `update_attack` / `update_defense` / `update_level` share a body | `utility.rs:1853-1900` | 22 |
| S7 | Setter preambles: the uncopyable SPSUMMON_CONDITION effect at 8 sites, the global continuous effect at 8 | `utility.rs:747, 2162-2229, 51`, `proc_workaround.rs:335-401` | 30 |

## 4. Harness, tests and tools

| # | Finding | Where | Saves |
| --- | --- | --- | --- |
| H1 | `diff/src/lib.rs` redefines 29 constants that `engine::constants` exports with the same type; only `HINT_REPLAY` is missing there (checked) | `crates/diff/src/lib.rs:23-52` | 28 |
| H2 | Card-pool loading written five times (vendor path, cdb, banlist, `Pool::advanced`), the Normal Monster filter twice, the Elfnote deck `include_str!` three times, fixed-or-random deck three times. Move to `carddata`: `vendor()`, `load_advanced()`, a `Decks` with `deal(rng)`. `bench::engine_duel` is then a copy of `bench::setup` | `diff/src/lib.rs:397-457`, `diff/src/bin/pool.rs:11`, `bench/src/lib.rs:29-64`, `bench/src/main.rs:35-47, 139-154`, `oracle/tests/oracle.rs:23` | 55 |
| H3 | Unused FFI items: all `TYPE_*`, 25 `OPCODE_*` (carddata's copies are the ones used), four externs with no caller (checked) | `crates/ocgcore-sys/src/lib.rs:88-94, 124-163` | 36 |
| H4 | Test helpers copied across integration tests: `records`, `scenario_records`, `median`, `type Result`, `STEP`, and the replay loop five times. Export from `diff` | `diff/tests/gamestate.rs`, `diff/tests/observation.rs`, `sim/tests/sim.rs` | 40 |
| H5 | The three bench binaries hand-roll the same flag loop: `flags(names, defaults) -> [u64; N]` | `bench/src/main.rs:20-34`, `bin/stack.rs:234-246`, `bin/throughput.rs:82-98` | 30 |
| H6 | `Tally::count` matches `MSG_MOVE` in five separate arms and decodes u32s inline next to `read_u32` | `diff/src/lib.rs:210-320` | 15 |
| H7 | Message framing twice, null-buffer-to-slice twice | `oracle/src/duel.rs:148-188`, `diff/src/lib.rs:682-690` | 15 |
| H8 | Observation buffer shapes spelled out three times | `sim/tests/sim.rs:86`, `bench/src/bin/stack.rs:123`, `py/src/lib.rs:120` | 12 |
| H9 | `tools/*.py`: the Lua path three times, the cdb opened three ways. A `tools/common.py` with `ROOT`, `cdb()`, `ported_codes()`, `lua_script()` also fixes the inconsistency in section 6 | `tools/batch.py`, `ported.py`, `prep.py`, `queue.py` | 10 |

Dependencies: every dependency in every `Cargo.toml` has a user.

## 5. Comments

Comment lines as a share of all lines, against the guideline's "well under 1%":

| Area | Comment lines | Share | Multi-line continuations |
| --- | --- | --- | --- |
| `crates/engine/src` | 1,006 | 3.4% | 221 |
| `crates/scripts/src` (without cards) | 448 | 4.5% | 109 |
| `crates/diff` | 112 | 5.8% | 34 |
| `oracle`, `sim`, `py`, `bench`, `carddata` | 153 | 2.8 - 5.1% | 31 |

- **Keep:** the C++ and Lua mapping lines where the Rust name differs (`// card::is_link_state` on `is_linked`),
  the Lua signatures on the `Value`/`Target`/`Operation` variants, mirrored oracle bugs, `unported` rationale.
  These are external constraints.
- **Delete, about 600 - 900 lines:**
  - Names restated, where the Rust name is the C++ or Lua name in snake case: `card.rs:743, 1749, 2055`,
    `field.rs:2510`, `processor.rs:351, 571, 1386, 1726, 2621, 2632`, `utility.rs:220, 1128, 1222, 1229, 1808, 2504`.
  - Conventions PORTING.md already states: "`player` None is the reason player" (9 times in `libcard.rs`,
    `libduel.rs`), "Lua passes ... as further arguments" (6 times).
  - Commented-out C++: `// operation_replace(EFFECT_SEND_REPLACE, 5, targets)` at `operations.rs:1666, 2060, 4007,
    4050`, each next to the `unported!` that already marks the gap.
  - Multi-line blocks (about 300), cut to one line: `script.rs:1` (9 lines), `script.rs:25` (7),
    `observation.rs:391` (9), `proc_fusion.rs:1-6`, `proc_synchro.rs:459-462`.
  - Plain `//` on pyo3 methods, which Python never sees: `py/src/lib.rs:192-399`. Make them `///` or drop them.
- **Fix (reported by a reviewer, not checked by me):** `chain.rs:1438` describes `take_select_chain` as pushing to
  `new_chains`, which it does not; `utility.rs:2335` sits above the wrong function.
- **Names:** almost every short name mirrors ocgcore and stays. The ones that mislead: `get_stat` beside
  `get_stat_value` (one caller, one unused parameter), `procedure` meaning the effect in SummonRule but the struct in
  SpSummonRule, `raise_material_events` beside `raise_material_event` with different behaviour.

## 6. Not refactors: found on the way

None of these is reached by a ported card as far as I can tell, but none has an `unported!` guard either.

- **Checked, text differs from the C++:** OperationReplace step 11 (`operations.rs:1997`) goes through
  `replace_destroyed`, which skips the restore for a self-destroy reason; C++ case 11 (`operations.cpp:5549`)
  restores unconditionally. Battle destruction normally has no reason effect, so it should not show.
- **Checked, missing source:** `filter_effect_into` and the two `is_affected_by_effect` lookups skip the target
  effects of `effect_target_owner` (`card.cpp:2411-2419`). No ported card uses `EFFECT_TYPE_TARGET`.
- **Checked, tools disagree:** `batch.py` and `ported.py` read only `cards.cdb`; `queue.py` and
  `CardDatabase::load` also read `release-*.cdb`, and `release-betb.cdb` exists. Cards from it are batched without
  an archetype and never listed by `ported.py`.
- **Checked the derive, impact not measured:** `PerPlayer<T>` and `PlayerInfo` derive `Clone`, so
  `self.player.clone_from(player)` in `copy_from` (`script.rs:311`) is a plain clone and does not reuse buffers.
  If a pile spills its inline capacity, every snapshot restore reallocates it.
- **Not checked:** `xmaterial_effects` (`card.rs:1651`) allocates a `Vec` on the hot lookup path whenever a card has
  materials; `filter_field_card` and `filter_matching_card` ignore `DUEL_3_COLUMNS_FIELD` (Speed Duel, out of
  scope); `is_summon_code` drops a `code1` of 0 that `libcard.cpp:610` inserts; `Pendulum::operation` has no
  counterpart to the Lua's 511007000 branch (an unofficial effect code).

## 7. Looked at and rejected

- A macro for the roughly 100 plain/`_full` pairs: two lines saved per pair, and `fn name(` stops being greppable.
- A general effect-setter builder: characters, not lines.
- Unused mirror parameters (`_scard`, `_sumtype`, `_reason`): they match the Lua arity and removing them touches
  hundreds of card calls.
- Merging SummonRule with MonsterSet, Destroy step 0 with 10, Synchro P31/P32 with P41/P42, Fusion
  `condition_mix` with `condition_mix_rep`: the differences are in lookup or message order.
- The free-chain collection loops in PhaseEvent, QuickEffect and PointEvent: they differ in id assignment and speed
  checks.
- `clone_by_field!` field lists: needed so `clone_from` reuses buffers.
- The duplicated signatures in the `CardUtility`/`DuelUtility`/`EffectUtility` traits: required, since `CardId` and
  `Duel` are foreign to the scripts crate.
- `oracle/src/message.rs` against the engine's own decoding, and the `MSG_*` constants in `ocgcore-sys`: the oracle
  must not share code with the engine it checks.

## Suggested order

1. Section 2.1, S1, H1 - H5: about 1,150 lines, no judgement, one full validation at the end.
2. The comment pass (section 5), in the same files while they are open.
3. Decide on 1.1. If yes: write `stock.rs` for the 23 shapes, migrate by script, validate, update PORTING.md and the
   porting brief so the next wave uses it from the start.
4. Section 2.2 and S2 - S7, file by file, each with the diff suites.
5. E28 only with a benchmark. Section 6 separately, as fidelity work in PLAN.md.
