# Refactor plan for after porting

Status: plan, not started. Do it once every card of the pool is ported (TCG queue, then the OCG queue), so no batch
branch is open while files move. The previous survey (2026-11-11, items E1-E27 and H1-H9) is done; its one open item,
E28, is carried over below. Git keeps its text.

Survey of 2026-11-12 at c86bc26 by five read-only audits (card files, engine core, engine data model and Lua API, script
library, harness, tools and docs). Line counts are non-blank lines; estimates come from scans and samples, and items
marked "unverified" were not traced end to end.

## Rules for every step

- **Byte-identical behaviour.** Nothing below may change the message stream, random draws, effect-lookup order or effect
  and field id order. Each step is verified with the full card suites (`DIFF_ALL_CARDS`, oracle cache warm), the
  Elfnote and Normal Monster suites, clippy and the Python tests, and is merged on its own.
- **Mechanical first.** Scripted rewrites (formatting, imports, metadata) go first and alone, so the hand-written steps
  that follow diff cleanly against them.
- **Measure what is hot.** Anything on an effect-lookup path (E28, filters) gets a before/after `stack` bench.

## Summary

| Area | Lines now | Could go now | At the full pool (~13.7k cards) | Effort | Risk |
| --- | --- | --- | --- | --- | --- |
| Card files (3,590) | 312,900 | 70,000 - 81,000 (22-26%) | about 270,000 - 310,000 | 4-6 days, mostly scripted | none to low |
| Engine core (`operations`, `processor`, `field`, `chain`, ...) | 14,900 | 550 - 650 | same | 3-4 days | low, E28 medium |
| Engine data model and Lua API (`card`, `lib*`, `effect`, `hash`, ...) | 14,700 | 360 (+220 optional) | same | 2 days | low |
| Script library (`utility.rs`, `proc_*`, `stock`, `build.rs`) | 7,900 | 600 | same | 2 days | low, Synchro target medium |
| Harness, benches, tools | 6,800 | 1,000 - 1,150 Rust and Python, plus 14,100 lines of queue data | same | 1-2 days | none |
| Docs | 260 KB | about 215 KB | same | 1 day | none |

The card files hold 87% of the code, so almost all of the reduction is there and almost all of it is scripted. The
engine and library work is smaller in lines but is where readability improves most (functions of 200-570 lines,
7-9-parameter calls).

## Correctness items found by the audits

Small and independent of the refactor; worth doing during porting rather than after.

- [x] `tools/stock.py` folded cards outside the batch (it globbed every card file; c18865703 was folded in two
  parallel batches). Now it reads only changed files or the given codes, `--all` for everything (committed).
- [ ] `observation.rs` `remap_events`/`referenced_cards` skip `desrep_chain`, which `hash.rs` hashes: a determinize at a
  prompt between Destroy steps 11/13 and 15 would not remap those events (unverified impact). One list accessor per kind
  of processor list fixes this and removes the three hand-kept copies (engine API item A1).
- [ ] `card_hash` and `effect_hash` list fields by hand without an exhaustive destructure, so a new field goes unhashed
  silently; `effect_hash` omits `active_sequence` and `active_handler` (on purpose or not: unverified).
- [ ] `libcard.rs:204` says previous set codes equal current ones "until a card that changes a code is ported": 59
  ported cards use EFFECT_CHANGE_CODE and 14 read `is_previous_set_card`, so this may be a reachable fidelity gap.
- [ ] `field.rs:1649` `list_extra.len() - extra_p_count as usize` can underflow; use `saturating_sub` as `add_card` does.
- [ ] `misread` (`effect.rs:888`) returns a default silently in release; record it like `unported!` so the harness sees it.
- [ ] `build.rs` finds `SUPPORT`/`CHECKS`/tables by substring and silently falls back when a declaration is misspelled
  or not `pub`; fail the build instead (about 5 lines).
- [ ] `build.rs` `uses_zones`/`uses_positions` scan the whole file including comments (a SUPPORT comment "zone effect"
  flips the flag). ygo-search reads these flags: strip comments before the scan and match words, not substrings.
- [ ] Script-state keys are allocated by hand across files; `aux::global_check` (`(1<<61)|id`) and Fusion MATERIAL
  (`(1<<61)|code<<8|i`) share a range and could collide for low Fusion codes (none collide today). One key enum.
- [ ] `list.rs` `insert` panics past `len` while `remove` returns `None`; make them consistent.

## 1. Card files

Measured over 3,590 files, 312,888 lines (1.22x the Lua's non-comment lines; median 1.3x per card). The extra length is
formatting, metadata and boilerplate, not logic. Scripts used for the counts: scratchpad `a1.py` - `a14.py` of the audit.

| # | Change | Lines | Effort | Automatable |
| --- | --- | --- | --- | --- |
| C1 | `use_small_heuristics = "Max"` in rustfmt.toml: rustfmt wraps calls at 60% of 140 columns today (c16306932.rs:40 spreads 105 columns over 4 lines); 2,581 wrapped calls and 241 let-else blocks fold | -12,600 (5,700 of them metadata) | minutes | yes, rustfmt in batches of 100 |
| C2 | Test metadata out of the card files (section 1.1) | -41,900 from card files | half a day | yes, plus a dump check |
| C3 | `Args` destructured in the parameter list (`&Args { e, tp, .. }: &Args`) instead of 5,849 alias lines | -5,849 | half a day | yes; update stock.py `ALIASES` and PORTING.md |
| C4 | Near-duplicate families stock.py does not fold yet: 47 families of 10+ (795 functions, 8,129 lines); single-target `select_target` targets (349 of that shape, 102 strict), "act on the first target" operations (67), Necro Valley searches (31, only the `SLOT` regex in stock.py:26 blocks them), draw/damage/recover targets with constants, discard costs, negate operations | -5,300 (up to -8,000 with the variants) | 1-2 days | yes, new `stock` shapes |
| C5 | One-line `stock_fn!(thtg = exists_target(...))` for the 1,238 functions that only call a stock helper | -2,950 | small | yes, in stock.py |
| C6 | One-line helpers for effect blocks that set only type, code, value, reset and property (1,336 blocks: UPDATE_ATTACK 95, bare ACTIVATE 94, DISABLE 45, SET_ATTACK_FINAL 33, LEAVE_FIELD_REDIRECT 32, ...), returning the `EffectId` so clone chains keep working | -5,000 | 1-2 days | semi (each reset checked) |
| C7 | Optional: `check!(chk, expr)` and `chkc!(..)` for the 4,171 `if chk == 0 { return ..; }` and 1,368 `if let Some(chkc) = chkc { return ..; }` blocks (3 lines each, rustfmt never joins them) | -11,000 | small | yes |
| C8 | Comments: about 15 restate PORTING's rounding rule; 1,516 multi-line SUPPORT comments (moot with C2) | -15 (-1,600 without C2) | small | partly |

Total: about 70,000 lines without C7, about 81,000 with it (22-26%). C1, C2 and C3 alone are about 55,000.

Rules for the stock shapes (C4, C5): fold only exact shapes, keep the Lua function names (`thtg = ...`), and never fold
a function whose address is compared (`has_self_discard_cost` in utility.rs:2440 and proc_fusion.rs:1122 use
`fn_addr_eq`). Related latent issue (unverified): release builds may merge identical function bodies, so a hand-written
cost identical to `Cost::self_discard` could compare equal where the Lua would not; none exists today.

Not worth doing: a general effect builder (saves at most one line per block over 9,204 blocks, REFACTOR 2026-11-11 came
to the same view); splitting the 308 functions over 40 lines (187 are `initial_effect`, they mirror the Lua); renaming
Lua abbreviations (`tc`, `sg`, `thtg`), which PORTING requires so ports can be compared with the Lua.

### 1.1 Test metadata (SUPPORT and CHECKS)

Today: 25,270 lines of SUPPORT (22,877 entries), 7,962 of CHECKS, 5,198 of rationale comments and 3,506 `use
crate::testing` lines: 41,900 lines in production files, compiled into every build through `CARD_SCRIPTS`
(build.rs:71-76) although only `diff` reads them (`scenario.rs:22`, `lib.rs:709`). About 150,000 lines at the full pool.

Options considered:

| Option | Card files | Cleanliness | Cost |
| --- | --- | --- | --- |
| A. Table in the diff crate, `crates/diff/cards.tsv` (`code, support, checks, note`, e.g. `84013237:removed*3`, `effect:0`, `message:ATTACK`, `report:effect:1`), parsed once at runtime, with a `should_parse_every_row` test | -41,900 | test data leaves production code; `Check`, `check_count` and the `testing.rs` location constants move to diff; `CardScript.support/checks` and build.rs:71-72 go | half a day; one row per card replaces ~12 lines; named constants (e.g. JUNOLDO) resolved to codes |
| B. Keep it in the card files, put `Check` and the location constants in the prelude, pack both lists into one brace macro with run-length (`ID HAND x2`) | -26,900 | still in production code | half a day |
| C. Cargo `testing` feature | 0 | `cargo test --workspace` unifies features, so it is on anyway, and each const needs a `#[cfg]` | small |
| D. Sibling `c<code>.test.rs` files | 0 | doubles the file count | medium |

Recommended: A. It is what the separation should look like, it saves the most, and the one-file-per-card argument that
put the metadata in the card files only mattered while many agents ported in parallel. After porting, a new card adds
one sorted row. SUPPORT order must be kept, because placement draws random numbers in list order; verify by dumping
every card's support and checks before and after and comparing. The oracle cache is unaffected (it keys on the placed
position). Optional on top: 1,306 cards' CHECKS are exactly "every own `stringid(ID, n)` resolves"; the harness could
derive that default (stricter for the 698 cards without CHECKS, so it is a harness change, not a refactor).

## 2. Engine core

`operations.rs` 5,426, `processor.rs` 3,392, `field.rs` 3,187, `chain.rs` 1,391, `playerop.rs`, `duel.rs`, `list.rs`:
14,900 lines, comments 2.0%. No unwrap, panic or `#[allow]` outside tests.

| # | Change | Lines |
| --- | --- | --- |
| K1 | `field_move_to_field` has 9 parameters with 2 bools (`enable` true at 1 site, `rule` at 2); 10 of 11 sites wrap to 11 lines. A 7-parameter `move_to_field`, the 3 odd sites build `MoveToField { rule: true, .. }` | -85 |
| K2 | `Default` on the unit structs (Destroy, OperationReplace, ChangePos, MoveToField, SpSummonStep, SummonRule, MonsterSet, Damage, SelectTribute, Release, SpellSet, ...) and `..Default::default()` in literals | -60 |
| K3 | A `Reason { effect, reason, player }` Copy struct for the triple every `process_*` destructures and `save_reason`/`restore_reason` copy; ends the 7-9-parameter calls (`field_send_to`, `field_special_summon(_step)`, `field_raise_event`, `field_destroy`, `field_damage`, ...). Plus an `ActivateCheck` struct for `is_activateable`'s 5 bools | -100 |
| K4 | Replacement units: the `Chain` literal at operations.rs:1777 and 1828 is `Chain::continuous` (make it `pub(crate)`); `start_replace(effect, event, insert)` for the shared event-build, push, `solving_event.insert`, Execute | -30 |
| K5 | Destroy and Release: push then `sort_by_key(id)` + `dedup()` instead of three `if !contains { push }`; `take_substitutes` (steps 0 and 10 share it); the GY/banished restore check (3 sites); `continue_send_to`; the `HINT_CARD` owner hint (4 sites) | -35 |
| K6 | Summon family: `set_hand_summon` (4 sites), the SPSUM_PARAM positions block (3), the "summoning finished" block (4), `summon.{location, sequence, pzone} = current` (3) | -36 |
| K7 | Events: `raise_event_if_any` (7 sites), adjacent single + group raises, CONTROL_CHANGED + MOVE pairs. Never fold pairs with a `process_single_event` between them | -30 |
| K8 | chain.rs: `process_single_event_effect` re-implements `new_trigger`; the `activate_chain` tail repeated in QuickEffect step 1; SolveChain step 1 re-implements `remove_procedure_oath`; the delayed-continuous dequeue written twice; `add_disable_chain` in a second `impl` block | -30 |
| K9 | Disabled zones (`refresh_location_info_instant` repeats RefreshLoc steps 0/7), zone occupancy `occupy(p, loc, seq, card)` (4 sites in field.rs), and about 10 small repeats (`field_destroy(.., PLAYER_NONE, PLAYER_NONE, LOCATION_GRAVE, 0)` at ~10 sites, the lost-target equip destroy, the turn-status mask, attack-counter reset, SpSummon param pack/unpack as a struct, ...) | -70 |
| K10 | Comments from 2.0% to under 1%: names restated (field.rs:926, 1319, 1820, ...), duplicated (field.rs:745), narration (processor.rs:2896), stale "phase 2" (processor.rs:1771, operations.rs:1901, 1966) and the ten porting-era "comes with the cards" notes (turn each into an `unported!` guard or delete it) | -150 |
| K11 | E28 (carried over): one walker `effect_sources(card, code, aura_target_first)` for the seven functions that walk Single, Equip, XMaterial, Aura (`card.rs:1520, 1607, 1640, 1654, 1681, 1719, 2836`); the Aura order and `is_available`'s id writes make the order a parameter. Bench first | -60 |

Long functions: each keeps its step `match` and moves step bodies into named functions (navigation, about 0 lines):
`process_attack` 571 lines (damage calculation, battle-destroy set, battle-destroy events), `process_send_to` 414
(`snapshot_previous`, `apply_redirects`, `send_one`, `raise_arrival_events`), `process_add_chain` 393,
`process_sp_summon_rule` 362 (the PROC_G group steps 20-28), `process_summon_rule` 341, `process_move_to_field` 291,
`process_turn` 289, `process_idle_command` 282, `process_phase_event` 252, `process_point_event` 244, `process_destroy`
210, `process_quick_effect` 207, `process_adjust` 202, `process_solve_chain` 201, `calculate_battle_damage` 182.

Module split (pure moves, no risk): `operations/` (send, destroy, summon, special_summon, control, life, xyz),
`processor/` (units, turn, window, command, battle), `field/` (state, zones, cards, effects, limits, chain_query).

Performance found on the way (exact, small): gate `filter_single_continuous_effect` with `registry.has_code` like
`field_effect_range` (it builds and sorts a set only to test emptiness at SendTo step 1 and Destroy steps 1/11); iterate
`delayed_quick` by index instead of cloning it (processor.rs:1179); avoid `tpchain = costs.to_vec()` per attack and the
`list_hand.clone()` / `list_main.clone()` copies (field.rs:1547, 1598; processor.rs:674, 1373, 1422).

Magic numbers: `RESET_EVENT + 0x1fe_0000` is Lua's `RESETS_STANDARD` (move it to engine `constants.rs`); `0xff00ff` (the
default releasable mask, 14 times), `0x1f_001f` and `0xff00_ffff` (summon-type mask) get names.

The 66 `unported!` sites in these files evaluate real lookups in release, so they can only go where the C++ makes the
same lookup. One `DIFF_ALL_CARDS` run that logs every site hit shows which are reachable.

## 3. Engine data model and Lua API

`card.rs` 3,200, `libduel.rs` 2,838, `effect.rs` 1,729, `libcard.rs` 1,290, `hash.rs` 962, `observation.rs` 756,
`action.rs` 710, `constants.rs` 705, `libeffect.rs` 394, `script.rs` 389: 14,700 lines, comments 2.4-9%.

| # | Change | Lines |
| --- | --- | --- |
| A1 | `Processor::chain_lists()`, `event_lists()`, `continuous_lists()` (and `_mut`) instead of the hand-kept lists in `hash.rs:655-713`, `observation.rs:686-725` and `:747-768` (fixes the `desrep_chain` drift above) | -50 |
| A2 | One `filter_summon_procedure` for Normal Summon and Set (`card.rs:1797`, `:1932` share their head and tail) returning an enum instead of `-2/-1/0/1` | -30 |
| A3 | Previous stats as one struct: the nine `previous_*` fields become `Card.previous_stats: CardStats` (= `TriggeringState`'s fields), read by one `stats(card)` shared with chain.rs:156 and SendTo step 2. Do not add the set-code lookup to SendTo unless the core does (it would renumber ids) | -25 |
| A4 | Dead and misleading names: `send_to_full` folds into `send_to`; `select_field_zone_full` has no short form, so rename it; `CardSet::first` duplicates `get_first`. (The audit also listed `get_range`, `get_count_code` and `get_count_hopt_index` as unused: ygo-search uses them, keep them.) | -20 |
| A5 | Processor sub-structs `core.activity`, `core.idle`, `core.temps` (the 14 counters listed twice in hash.rs), which cuts `flow` (334 lines) | -35 |
| A6 | Raise-event calls take the existing `Event` struct instead of 7 positional arguments (114 engine sites; `confirm_cards` spends 20 lines on 4 calls) | -25 here, -80 engine-wide |
| A7 | `get_value` lists 25 variants only to reach `misread` (every other getter uses `_ =>`); `get_player_battle_damage_value` has one caller and reads only `Int` | -25 |
| A8 | Small helpers: legality conditions written twice in action.rs (`idle_flag(command)`), `select_disfield` re-derives `response_zone`, `with_reason(..)` for the save/set/restore pattern (4 sites), the material-limit check (3), the cost-to-hand/deck/extra prefix, one `shuffle` on the RNG (3 copies), `place_key` re-implements `field_sequence` | -60 |
| A9 | Comments: libeffect.rs:203-323 repeats the variant comments of effect.rs:143-186; stale `card.rs:937` (ASSUME_ATTACK/DEFENSE are ported); multi-line blocks at `card.rs:348, 2834`, `effect.rs:41, 656`, `hash.rs:50, 230, 406`, `observation.rs:132, 420, 629`, `script.rs:1, 160, 169, 380` | -40 |
| A10 | Optional: one width (or a bitflag newtype) for LOCATION_* (u8/u16/u32 today) and POS_* (u32 constants, u8 fields): removes 140 casts, touches the prelude types card scripts see | about -140 casts |

Also: rename the confusable pairs `is_player_can_sendto_hand` (Lua mirror) vs `is_player_can_send_to_hand` (internal)
and `is_player_can_special_summon` vs `is_player_can_spsummon` per PORTING's `field_` prefix rule; order `libcard.rs` by
Lua family (`is_level_below` at :382, `is_level_above` at :1345); move counters and column zones out of `card.rs`
(`:3111-3433`) into `counter.rs`; `Effect.flag: [u32; 2]` as `flag` and `flag2`; a `debug_assert!` in the `cards[id]`
and `effects[id]` Index impls so tests catch a bad id (they read a scratch object today).

## 4. Script library

`utility.rs` 2,743, `proc_synchro.rs` 1,130, `proc_fusion.rs` 1,126, `proc_workaround.rs` 422, other `proc_*`,
`stock.rs`, `chain.rs`, `build.rs`: 7,900 lines without the generated `constant.rs`, comments 4.5%.

| # | Change | Lines |
| --- | --- | --- |
| L1 | Optional-argument chains longer than plain + `_full`: `delayed_operation` has 5 tiers (`_hint`, `_reset` with one caller each), `remove_until_desc`, `select_unselect_group_break` (1 caller), `detach_from_self_op` (1 caller), `get_to_be_linked_zone_full` (only its twin calls it, so `clink`/`emz` are always false: 3 dead branches) | -85 |
| L2 | Near-duplicates: `chain::is_setcode/is_type/is_code` as one `chain_property`, the `add_must_be_*_summoned` family, `lnklimit`/`fuslimit`/`ritlimit`/`synlimit`/`xyzlimit` as `summon_type_limit::<T>` (27 call sites), the Pendulum filter closure (4 copies), `toggle` written three times (`CardSet::toggle` exists) | -70 |
| L3 | `Synchro::target` (proc_synchro.rs:827, 215 lines): the P31/P32 and P41/P42 loops repeat the same bookkeeping; a `Selection` struct and one `complete(extra)` closure. Keep the order of lookups in `finish` (medium risk) | -60 |
| L4 | `summon_eff_op` (proc_fusion.rs:1057, 127 lines): extract the extra-material-effect block; lines 1178-1180 are `set_checks(duel, [None; 3])`. The must-be-materials prelude repeats in proc_fusion.rs (4 sites) | -30 |
| L5 | Comments from 4.5% to under 1% (mostly "Auxiliary.X: what it does" where the name says it, e.g. utility.rs:1146-1194); stale utility.rs:771 (c3682106 reads the Double Snare marker) and official/mod.rs:14 | -270 |
| L6 | `constant.rs` is generated by `tools/constants.py`: add an `// @generated` header so reviews and line counts skip it; keep it checked in (generating in build.rs would need `vendor/` to build) and keep every Lua name (the prelude rule) | 0 |
| L7 | `build.rs`: write the `mod c<code>;` list into a checked-in `official/mod.rs` by a tool instead of `include!`, so `cargo fmt` reaches the card files and PORTING's separate rustfmt step goes | small |

The stock extensions are counted under C4 and C5.

## 5. Harness, benches and tools

| # | Change | Lines |
| --- | --- | --- |
| H1 | Duel setup written 3 times (`diff/src/lib.rs:445-501`, `bench/src/lib.rs:45-62`, `py/src/lib.rs:94-106`): one `Position::dealt(rng, lp, deal)` | -25 |
| H2 | Vendor loading: `Scripts::vendored()` (4 sites), `Decks::advanced` in carddata (bench repeats it), `MAX_DECISIONS` defined 3 times; `carddata/src/lib.rs:11` `include_str!`s a deck from the diff crate's directory: move the file | -25 |
| H3 | `Vendor::leak` reads all ~13.5k scripts and leaks them, 24 times in tests: a memoized `Vendor::shared(kind)` (`OnceLock`), faster tests and shorter call sites | -10, faster tests |
| H4 | Test helpers: `type Result` redefined 3 times, `STEP = 25` twice plus literals, records rebuilt in `observation.rs:94, 138` that `sim.rs:17` caches (move the cache to `diff::records`), `Tally` folding by clone 3 times (`AddAssign<&Tally>` + `Sum`), `diff/tests/diff.rs` as one table of (vendor, seeds, assertion) | -50 |
| H5 | `card` binary: `--trace` reruns the scenario although `results[index]` has the stream; the port-status match repeats `pool.rs` | -10 |
| H6 | Dead code: `Report.messages/.decisions/.winner`, `Tally.tribute_prompts` | -15 |
| H7 | Benches: delete `perft.rs` (one-off), fold `throughput.rs` into `stack` (a deck flag), move `search_api.rs`'s `--check` into an engine test and its timings into two `stack` rows, delete the hard-coded old phase targets in `stack.rs:271-319, 352-356`, delete the report-only test `observation.rs:304-340`; optional: `bench/main.rs` | -500 to -650 |
| H8 | Long functions: `stack.rs` main 151, `bench/main.rs` 99, `diff/lib.rs:504` play 98 (split comparison and report), `Tally::count` 73, `scenario.rs:112` place 61 | navigation |
| H9 | The oracle cache key hashes `diff/src/lib.rs`, so any Tally refactor invalidates the cache: move `Tally`, `Mismatch` and summarize to an unhashed module first | 0, saves a cache rebuild |
| H10 | Comments: diff 4.4%, py 5.6%, oracle 3.8%, sim 3.2%, bench 2.7% (fiber's 11% are SAFETY notes, keep) | -100 |
| H11 | Tools obsolete after porting: `batch.py`, `queue.py`, `prep.py`, `diff/src/bin/pool.rs`, `porting/queue.tsv` and `queue-ocg.tsv` (14,100 lines of data). Keep `ported.py` (drop its ported filter), `constants.py`, `pgo.py`; keep `stock.py` only while new sets are ported | -215 Python and Rust, -14,100 data |

Smaller tool fixes while they live: `batch.py:24` reads `--top 1000` (no `=`) as the batch count; `batch.py:27` never
closes its file; the code regex `\d{4,9}` (`batch.py:37`) also catches ATK and LP numbers.

## 6. Docs

PLAN.md is 201 KB (its Findings section is about 190 KB), PORTING.md 27 KB, this file, README.md 7 KB.

- **Drift to fix now:** PORTING.md:121 says "at most three labels" (the code has 8 and overflow is `unported!`);
  PLAN.md:82 names `prep.sh`, which is a scratch script outside the repo; README.md:61 says 2,160 card files;
  README.md:90 promises a re-measure "when it ends" (phase 8 is done); `card.rs:2` says `--release` where the docs say
  `--profile card`; PLAN.md:21 still calls the RNG "believed"; PLAN.md:18 describes choice-log replay only (fiber now
  resumes in place); PLAN.md:42 ticks the 1M decisions/s/thread target while 322-395k is measured.
- **After porting:** PLAN.md keeps Decisions, Architecture and Verification plus a short "Open" list (the .yrpX replay
  runner, the known engine gaps, E28, BattleCommand step 16, Pendulum 511007000); phases and findings move to
  `docs/history.md` or stay in git. PORTING.md stays as the guide for porting new sets (workflow, mirror API,
  fidelity) without the queue and pipeline material and the machine-specific Python path. README.md keeps intro,
  layout, setup, usage, accuracy, re-measured benchmarks and license; Status and the batch.py line go. This file is
  deleted when the plan is done.

## Order

1. The correctness items above (during porting).
2. C1 (rustfmt heuristics), workspace-wide, alone.
3. C2 (metadata to `crates/diff/cards.tsv`) with the before/after dump; H9 first so the oracle cache survives.
4. C3 (`Args` destructuring), then C4 and C5 (stock shapes and the wrapper macro) through stock.py, then C6; C7 if
   wanted.
5. Engine: K2 and A4 (mechanical), K1, K3 and A6 (signatures), K4-K9, A1-A3, A5, A7, A8, then the long-function
   splits and the module split (pure moves, last, so earlier diffs stay readable). K11 (E28) with a bench.
6. Library L1-L4, L6, L7; harness H1-H8, H10.
7. Comment passes (K10, A9, L5, H10) and docs, then delete the porting tools (H11).

## Benefits

- **Size:** about 75,000-85,000 fewer lines now (about 21-24% of the repo), about 280,000-320,000 at the full pool,
  almost all of it scripted.
- **Separation:** fuzz scenarios and checks leave the production crates; release builds and the Python module carry no
  test data.
- **Readability:** no function over about 150 lines in the engine; no call with more than 7 positional arguments; the
  porting-era comments and dead tiers gone; one way to do each common thing in card files.
- **Safety:** the hash and observation lists can no longer drift apart; new fields fail to compile until hashed;
  metadata typos fail the build; script-state keys cannot collide.
- **Speed:** a few exact hot-path wins (the `has_code` gate, fewer clones); faster tests (memoized vendor loading); the
  oracle cache survives harness refactors.

## Decisions for the owner

- **C7:** one-line `check!`/`chkc!` macros hide an early return in a macro (-11,000 lines). Yes or no.
- **Section 1.1:** test metadata as a table in the diff crate (recommended) or packed in the card files.
- **Comment target:** under 1% is not reachable while the Lua-signature comments on the `Value`/`Target`/`Operation`
  variants stay (effect.rs, script.rs); they document an external contract, so the plan keeps them.
- **Names:** card files keep the Lua's abbreviations (`tc`, `sg`, `thtg`) for comparability with the Lua, against the
  complete-word rule.
