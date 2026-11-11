// Red-Eyes Soul
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 6556909;

pub const LISTED_NAMES: &[u32] = &[CARD_REDEYES_B_DRAGON, ID];

// In the hand and on the field, with Red-Eyes Black Dragons in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_REDEYES_B_DRAGON, DECK),
    (CARD_REDEYES_B_DRAGON, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e1.set_value(duel, CARD_REDEYES_B_DRAGON as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - a.tp), 1, None))
    });
    e2.set_cost(duel, Cost::self_to_grave);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit_full(duel, 1, (ID, 1), EFFECT_COUNT_CODE_DUEL);
    e3.set_target(duel, damtg);
    e3.set_operation(duel, damop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RED_EYES]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let handler = Some(e.get_handler(duel).into());
        return Ok(duel.get_mzone_count(tp, handler) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(
        duel,
        a,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_DECK,
        POS_FACEUP,
    )
}

fn damfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, CARD_REDEYES_B_DRAGON) && c.get_base_attack(duel) > 0
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && damfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(damfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, damfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let attack = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_base_attack(duel));
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, attack);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, a.e)
    {
        let attack = tc.get_base_attack(duel);
        duel.damage(1 - a.tp, attack as u32, REASON_EFFECT)?;
    }
    Ok(())
}
