// Abyss Shark
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 61496006;

pub const LISTED_SERIES: &[u16] = &[SET_NUMBER];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand with WATER monsters on the field, and Fish monsters (7 Colored Fish, Great White) in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (23771716, MZONE),
    (23771716, MZONE),
    (23771716, DECK),
    (23771716, DECK),
    (13429800, DECK),
    (13429800, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_XYZ_LEVEL);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::XyzLevels(xyzlv));
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    !(c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_WATER))
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_field_group(a.tp, LOCATION_MZONE, 0);
    !g.is_empty() && !g.is_exists(duel, cfilter, 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FISH) && c.is_level(duel, &[3, 4, 5]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, LOCATION_HAND as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_attribute(duel, ATTRIBUTE_WATER));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 0, 1);
    e2.set_condition(duel, damcon);
    e2.set_value(duel, DOUBLE_DAMAGE as i32);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    if !c.is_relate_to_effect(duel, e) || duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    if duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sc = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !sc.is_empty() && duel.send_to_hand(sc, tp, REASON_EFFECT)? > 0 {
            duel.confirm_cards(1 - tp, sc)?;
        }
    }
    Ok(())
}

// The Lua registers a flag effect here once the doubling applies, which a condition of this shape cannot do.
fn damcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    let (bc0, bc1) = duel.get_battle_monster(tp);
    let doubles = duel.get_flag_effect(tp, ID) == 0
        && bc1.is_some()
        && bc0.is_some_and(|bc0| bc0.is_set_card(duel, &[SET_NUMBER]) && bc0.is_status(duel, STATUS_OPPO_BATTLE));
    unported!(duel, doubles);
    false
}

fn xyzlv(duel: &Duel, e: EffectId, _: CardId, rc: CardId) -> Vec<i32> {
    let level = e.get_handler(duel).get_level(duel) as i32;
    if rc.is_set_card(duel, &[SET_NUMBER]) {
        vec![4, 3, level]
    } else {
        vec![level]
    }
}
