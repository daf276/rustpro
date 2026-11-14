// Epsilon The Magnet Warrior
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 52566270;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_MAGNET_WARRIOR, SET_MAGNA_WARRIOR];

// Copies in the hand to summon, Alpha and Beta The Magnet Warrior in the Deck to send, and Gamma in the GY to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (99785935, DECK),
    (39256679, DECK),
    (99785935, DECK),
    (11549357, GRAVE),
    (11549357, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(11549357)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_cost(duel, cost);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_monster(duel)
        && c.is_level_below(duel, 4)
        && c.is_set_card(duel, &[SET_MAGNET_WARRIOR])
        && !(c.is_code(duel, ID) || c.is_code(duel, code))
        && c.is_able_to_grave_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let code = a.e.get_handler(duel).get_code(duel);
    let filter = |c, duel: &Duel| cfilter(c, duel, code);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let Some(tc) = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?.get_first() else {
        return Ok(false);
    };
    duel.send_to_grave(tc, REASON_COST)?;
    let label = tc.get_code(duel);
    a.e.set_label(duel, i64::from(label));
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_set_card(duel, &[SET_MAGNET_WARRIOR]) || c.is_set_card(duel, &[SET_MAGNA_WARRIOR]))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && !face_up_on_field(c, duel, tp)
}

// Whether tp controls a face-up card with c's name, which the Magnet Warrior summons exclude.
pub fn face_up_on_field(c: CardId, duel: &Duel, tp: u8) -> bool {
    let code = c.get_code(duel);
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_code(duel, code)),
        tp,
        LOCATION_ONFIELD,
        0,
        1,
        None,
    )
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel)) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    let label = e.get_label(duel);
    e1.set_value(duel, label as i32);
    c.register_effect(duel, e1);
    if c.is_immune_to_effect(duel, e) || duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let sg = duel.get_matching_group(
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
        tp,
        LOCATION_GRAVE,
        0,
        None,
    );
    if !sg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sc = sg.select(duel, tp, 1, 1, None)?;
        if sc.is_empty() {
            return Ok(());
        }
        duel.break_effect()?;
        duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
