// Witchcrafter Pupils
use engine::{Operation, Target};

use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, REMOVED};

const ID: u32 = 69964858;

pub const LISTED_SERIES: &[u16] = &[SET_WITCHCRAFTER];

// The slots of the chain data's revealed_card and pupils_target_effect.
const REVEALED_CARD: u8 = 1;
const TARGET_EFFECT: u8 = 2;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_WITCHCRAFTER], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(fc), sumtype, tp)),
    ];
}

pub const MATERIAL_SETCODE: &[u16] = &[SET_WITCHCRAFTER];

// On the field and in the Extra Deck, Fusion Summoned by Polymerization with two Witchcrafter Genni, with Witchcrafter Spells in the Deck
// to add and Witchcrafter Celebration and Creation in the hand to reveal, and banished Witchcrafter cards to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (24094653, HAND),
    (64756282, HAND),
    (64756282, HAND),
    (6958567, HAND),
    (57916305, HAND),
    (57916305, DECK),
    (32353566, DECK),
    (64756282, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, _| duel.is_main_phase() || duel.is_battle_phase());
    e1.set_cost(duel, cost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(
        duel,
        0,
        TIMING_MAIN_END | TIMING_BATTLE_START | TIMING_BATTLE_END | TIMINGS_CHECK_MONSTER,
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e2.set_target(duel, rtgtg);
    e2.set_operation(duel, rtgop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WITCHCRAFTER]) && c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn revealfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
    c.is_set_card(duel, &[SET_WITCHCRAFTER])
        && (c.is_normal_spell(duel) || c.is_quick_play_spell(duel))
        && c.check_activate_effect(duel, true, false).is_some()
        && c.check_activate_effect(duel, true, false)
            .is_some_and(|te| te.get_operation(duel).is_some())
}

fn record_revealed(duel: &mut Duel, a: &Args, og: CardSet) {
    a.e.set_chain_data_group(duel, REVEALED_CARD, og);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let choices: [CostChoice; 2] = [
        (
            |_, _, _| Ok(true),
            aux::stringid(ID, 2),
            |duel, a| duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK, 0, 1, None),
        ),
        (
            |duel, a, chk| Cost::reveal(duel, a, chk, revealfilter, false, (1, 1), Some(record_revealed), LOCATION_HAND),
            aux::stringid(ID, 3),
            |_, _| true,
        ),
    ];
    Cost::choice(duel, a, chk, &choices)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (e, tp) = (a.e, a.tp);
    match Cost::get_choice(duel, e) {
        Some(1) => {
            e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
            duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
        }
        Some(2) => {
            e.set_category(duel, 0);
            let revealed = e.get_chain_data_group(duel, REVEALED_CARD).and_then(|g| g.get_first());
            let te = revealed.and_then(|revealed| revealed.check_activate_effect_event(duel, true, true));
            // The Lua errors on a nil effect
            unported!(duel, te.is_none());
            let Some((te, event)) = te else { return Ok(false) };
            unported!(duel, event.is_some());
            duel.clear_target_card();
            let (property, property2) = te.get_property_full(duel);
            e.set_property_full(duel, property, property2);
            let (label, label_object) = (te.get_label(duel), te.get_label_object(duel));
            e.set_label(duel, label);
            e.set_label_object(duel, label_object);
            match te.get_target(duel) {
                Target::Action(tg) => {
                    let args = Args {
                        e,
                        tp,
                        ep: PLAYER_NONE,
                        rp: PLAYER_NONE,
                        ..Args::default()
                    };
                    tg(duel, &args, 1, None)?;
                }
                other => unported!(duel, !matches!(other, Target::None)),
            }
            e.set_category(duel, 0);
            duel.clear_operation_info(0);
            e.set_chain_data(duel, TARGET_EFFECT, te.index() as i64);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    match Cost::get_choice(duel, e) {
        Some(1) => stock::search(duel, a, thfilter, LOCATION_DECK)?,
        Some(2) => {
            let te = e
                .get_chain_data(duel, TARGET_EFFECT)
                .and_then(|index| duel.effect_at(index as usize));
            let Some(te) = te else { return Ok(()) };
            let op = te.get_operation(duel);
            unported!(duel, !matches!(op, None | Some(Operation::Event(_) | Operation::Promptless(_))));
            if let Some(Operation::Event(op) | Operation::Promptless(op)) = op {
                op(duel, a)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn rtgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WITCHCRAFTER]) && c.is_faceup(duel) && c.is_able_to_grave(duel)
}

fn rtgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, rtgfilter, LOCATION_REMOVED, CATEGORY_TOGRAVE)
}

fn rtgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOGRAVE);
    let g = duel.select_matching_card(tp, rtgfilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}
