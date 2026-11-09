// Exstellarknight Constellar Ptolemy Ω7
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 6195332;

pub const LISTED_NAMES: &[u32] = &[ID];

// Three copies in the Extra Deck, Xyz Summoned with three Level 7 Monsters (Slot Machine, Launcher Spider and Gaia the Fierce Knight).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (3797883, MZONE),
    (87322377, MZONE),
    (6368038, MZONE),
    (3797883, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        duel.get_current_phase() == PHASE_MAIN2 && duel.is_turn_player(tp) && !duel.has_flag_effect(tp, ID)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE);
    e1a.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_condition(duel, atkdefimmcon);
    e1a.set_value(duel, 2700);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e1b);
    let e1c = e1a.clone_effect(duel);
    e1c.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1c.set_value(
        duel,
        Value::Effect(|duel, e, te| te.get_owner_player(duel) != e.get_handler_player(duel) && te.is_activated(duel)),
    );
    c.register_effect(duel, e1c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, tdcost);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    c.is_set_card_full(duel, &[SET_TELLARKNIGHT, SET_CONSTELLAR], Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
        && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
        && c.is_faceup(duel)
}

fn atkdefimmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TELLARKNIGHT]) && c.is_monster(duel) && c.is_faceup(duel)
}

fn atkdefimmcon(duel: &Duel, a: &Args) -> bool {
    duel.get_matching_group(
        atkdefimmfilter,
        a.e.get_handler_player(duel),
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        None,
    )
    .get_class_count(duel, |c, duel| i64::from(c.get_code(duel)))
        >= 7
}

fn tdcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let max = duel.get_target_count(CardId::is_able_to_deck, a.tp, 0, LOCATION_MZONE, None) as u16;
    Cost::detach_from_self_with(duel, a, chk, 1, max, |duel, e, og| e.set_label(duel, og.len() as i64))
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_deck(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_deck, tp, 0, LOCATION_MZONE, 1, None));
    }
    let ct = a.e.get_label(duel) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, CardId::is_able_to_deck, tp, 0, LOCATION_MZONE, ct, ct, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, g.map_or(0, |g| g.len() as u8), tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
