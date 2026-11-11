// Courageous Crimson Chevalier Bradamante
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 91479482;

// In the hand and on the field with Elemental HERO Burstinatrix (a Warrior) to equip to and Infernoble Arms - Durendal in the Deck,
// destroyed in battle by Frostosaurus on the opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (58932615, MZONE),
    (37478723, DECK),
    (37478723, DECK),
    (6631034, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_LEAVE_FIELD_P);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_operation(duel, leavepop);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_race(duel, RACE_WARRIOR)
        && duel.is_existing_matching_card(|ec, duel| eqfilter(ec, duel, c, tp), tp, LOCATION_DECK, 0, 1, None)
}

fn eqfilter(c: CardId, duel: &Duel, tc: CardId, tp: u8) -> bool {
    c.is_equip_spell(duel) && c.check_equip_target(duel, tc) && c.check_unique_on_field(duel, tp) && !c.is_forbidden(duel)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && tgfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(|c, duel| tgfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, |c, duel| tgfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_SZONE) == 0 {
        return Ok(());
    }
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_matching_card(tp, |c, duel| eqfilter(c, duel, tc, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(gc) = g.get_first() {
        duel.equip(tp, gc, tc)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let bc = c.get_battle_target(duel);
    if !(c.is_location(duel, LOCATION_GRAVE) && a.rp == 1 - a.tp) {
        return false;
    }
    unported!(duel, bc.is_none());
    bc.is_some_and(|bc| bc.is_faceup(duel) && bc.is_relate_to_battle(duel)) && a.e.get_label(duel) == 1
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let bc = c.get_battle_target(duel);
    unported!(duel, bc.is_none());
    let Some(bc) = bc else { return Ok(false) };
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.get_location_count(tp, LOCATION_SZONE) > 0
            && bc.is_controler(duel, 1 - tp)
            && bc.is_able_to_change_controler(duel));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(bc.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    if duel.get_location_count(tp, LOCATION_SZONE) == 0 {
        return Ok(());
    }
    let bc = c.get_battle_target(duel);
    unported!(duel, bc.is_none());
    let Some(bc) = bc else { return Ok(()) };
    if !(bc.is_faceup(duel) && bc.is_relate_to_battle(duel) && bc.is_controler(duel, 1 - tp)) {
        return Ok(());
    }
    if duel.equip(tp, bc, c)? {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_label_object_group(duel, Some(c.into()));
        e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        bc.register_effect(duel, e1);
    }
    Ok(())
}

fn leavepop(duel: &mut Duel, a: &Args) -> Script<()> {
    let eff = a.e.get_label_object(duel);
    let label = i64::from(a.r & REASON_BATTLE > 0 && a.e.get_handler(duel).get_equip_group(duel).is_empty());
    if let Some(eff) = eff {
        eff.set_label(duel, label);
    }
    Ok(())
}
