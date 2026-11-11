// Sacred Fire King Garunix
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 66431519;

pub const LISTED_NAMES: &[u32] = &[ID];
// Two in hand and one in the Graveyard to react to Fire King Courtier Ulcanix being destroyed by Raigeki, with Legendary Fire King Ponix in the Deck to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (44455560, MZONE),
    (44455560, MZONE),
    (90681088, DECK),
    (90681088, DECK),
    (12580477, HAND | OPPONENT),
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
    e1.set_property_full(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

pub fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && c.is_original_attribute(duel, ATTRIBUTE_FIRE)
        && c.is_previous_controler(duel, tp)
        && !c.is_previous_location(duel, LOCATION_SZONE)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| spconfilter(c, duel, a.tp), 1, None))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && !(c.is_location(duel, LOCATION_GRAVE) && a.eg.is_some_and(|eg| eg.is_contains(c))));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE)
        && c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB)
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_MZONE))
        && !c.is_code(duel, ID)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(desfilter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(
        0,
        CATEGORY_DESTROY,
        None,
        1,
        tp,
        (LOCATION_HAND | LOCATION_DECK | LOCATION_MZONE) as i32,
    );
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(a.e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, desfilter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_MZONE, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        let atk = tc.get_attack(duel) / 2;
        let c = e.get_handler(duel);
        if duel.destroy(tc, REASON_EFFECT)? > 0 && c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
            c.update_attack(duel, atk, Some(RESETS_STANDARD_DISABLE_PHASE_END), None, None);
        }
    }
    Ok(())
}
