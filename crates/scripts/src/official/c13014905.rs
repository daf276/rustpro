// Agnimal Candle
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 13014905;

// Copies in the Deck for Fire Formation - Tenki to add, and on the field with weak monsters (Key Mace) the opponent's attackers
// (Elfnote Lucina) send to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, DECK),
    (ID, DECK),
    (ID, MZONE),
    (57103969, HAND),
    (1929294, MZONE),
    (1929294, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_reason(duel, REASON_DRAW));
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LVCHANGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 3, 0);
    e2.set_condition(duel, lvcon);
    e2.set_target(duel, lvtg);
    e2.set_operation(duel, lvop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel) && c.is_controler(duel, tp)
}

fn lvcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, tp), 1, None))
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.has_level(duel) && c.get_flag_effect(duel, ID) == 0);
    }
    c.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    duel.set_operation_info(0, CATEGORY_LVCHANGE, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && c.has_level(duel)) {
        return Ok(());
    }
    let b1 = true;
    let b2 = c.is_level_above(duel, 2);
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    let value = if op == Some(1) { 1 } else { -1 };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
    e1.set_value(duel, value);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    c.register_effect(duel, e1);
    Ok(())
}
