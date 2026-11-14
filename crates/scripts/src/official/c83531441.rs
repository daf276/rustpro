// Dante, Traveler of the Burning Abyss
use super::c71594310::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 83531441;

pub const LISTED_SERIES: &[u16] = &[SET_BURNING_ABYSS];
// In the Extra Deck, Xyz Summoned with Level 3 Rubic and Normal Monsters, with Rubic in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (734741, MZONE),
    (734741, MZONE),
    (1761063, MZONE),
    (4931562, MZONE),
    (734741, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure<3, 2>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<1, 1>, atkcost])
    });
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_attacked_count(duel) > 0);
    e2.set_operation(duel, |duel, a| {
        duel.change_position(a.e.get_handler(duel), POS_FACEUP_DEFENSE)?;
        Ok(())
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e3);
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck_as_cost(tp, 1));
    }
    let max_ct = duel.get_field_group_count(tp, LOCATION_DECK, 0).min(3) as u32;
    let op = if max_ct == 1 {
        1
    } else {
        duel.announce_number_range(tp, 1, max_ct)?
    };
    a.e.set_label(duel, op as i64);
    duel.discard_deck(tp, op as u16, REASON_COST)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, e.get_label(duel) as i32 * 500);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BURNING_ABYSS]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && chkc != c && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}
