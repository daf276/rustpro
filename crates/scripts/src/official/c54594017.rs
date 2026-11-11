// Blackwing - Sharnga the Waning Moon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 54594017;

// Blackwing Armor Master (2800 ATK) on the field to Special Summon from the hand and to enable the Graveyard effect.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (69031175, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, TIMINGS_CHECK_MONSTER_E, TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit_full(duel, 1, (ID, 1), EFFECT_COUNT_CODE_DUEL);
    e2.set_condition(duel, descond);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_attack_above(duel, 2000)),
            0,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            None,
        )
}

fn bwfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BLACKWING]) && c.is_type(duel, TYPE_SYNCHRO) && c.is_faceup(duel)
}

fn descond(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(bwfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}
