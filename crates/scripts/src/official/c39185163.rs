// Doomking Balerdroch
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 39185163;

pub const LISTED_NAMES: &[u32] = &[ID];

// On the field and in the Graveyard, with Wasteland to activate and Zombie Master (an ignition effect) on each side to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (23424603, HAND),
    (17259470, MZONE),
    (17259470, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| {
        duel.is_existing_matching_card(CardId::is_faceup, 0, LOCATION_FZONE, LOCATION_FZONE, 1, None)
    });
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2.set_condition(duel, effcon);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    let trig_typ: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_TYPE);
    let trig_race: u64 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_RACE);
    let code1: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_CODE);
    let code2: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_CODE2);
    trig_typ & TYPE_MONSTER > 0 && trig_race & RACE_ZOMBIE > 0 && code1 != ID && code2 != ID
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_remove(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let b1 = !duel.has_flag_effect(tp, ID) && duel.is_chain_disablable(a.ev as u8);
        let b2 = !duel.has_flag_effect(tp, ID + 1)
            && duel.is_existing_matching_card(
                rmfilter,
                tp,
                LOCATION_MZONE | LOCATION_GRAVE,
                LOCATION_MZONE | LOCATION_GRAVE,
                1,
                None,
            );
        return Ok(b1 || b2);
    }
    duel.set_possible_operation_info(0, CATEGORY_DISABLE, a.eg, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, PLAYER_EITHER, (LOCATION_MZONE | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let location = LOCATION_MZONE | LOCATION_GRAVE;
    let filter = aux::necro_valley_filter(rmfilter);
    let b1 = !duel.has_flag_effect(tp, ID) && duel.is_chain_disablable(a.ev as u8);
    let b2 = !duel.has_flag_effect(tp, ID + 1) && duel.is_existing_matching_card(&filter, tp, location, location, 1, None);
    if !(b1 || b2) {
        return Ok(());
    }
    match duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])? {
        Some(1) => {
            duel.negate_effect(a.ev as u8);
            duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        }
        Some(2) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
            let g = duel.select_matching_card(tp, &filter, tp, location, location, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
            }
            duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        }
        _ => {}
    }
    Ok(())
}
