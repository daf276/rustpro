// Ally of Justice Decisive Armor
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 9888196;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// A copy on the field and in the Extra Deck, Synchro Summoned with Infernoble Knight Oliver (a Level 4 Tuner), Noble Knight Artorigus (Level 4) and
// Melffy Rabby (Level 2), against LIGHT monsters in the opponent's hand and field, and set Traps on their side, with monsters in hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (58346901, MZONE),
    (92125819, MZONE),
    (20129614, MZONE),
    (39552864, MZONE | OPPONENT),
    (39552864, MZONE | OPPONENT),
    (39552864, HAND | OPPONENT),
    (39552864, HAND | OPPONENT),
    (14315573, SZONE | OPPONENT),
    (14315573, SZONE | OPPONENT),
    (1929294, HAND),
    (1929294, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, con);
    e1.set_target(duel, destg1);
    e1.set_operation(duel, desop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, con);
    e2.set_cost(duel, descost2);
    e2.set_target(duel, destg2);
    e2.set_operation(duel, desop2);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_HANDES + CATEGORY_TOGRAVE + CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, con);
    e3.set_cost(duel, hdcost);
    e3.set_target(duel, hdtg);
    e3.set_operation(duel, hdop);
    c.register_effect(duel, e3);
}

fn confilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_LIGHT)
}

fn con(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(confilter, a.tp, 0, LOCATION_MZONE, 1, None)
}

fn destg1(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel) && chkc.is_facedown(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_facedown, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, e.get_description(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_facedown, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn desop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_facedown(duel)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn descost2(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_grave_as_cost, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, e.get_description(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_grave_as_cost, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn destg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(CardId::is_spell_trap, a.tp, 0, LOCATION_ONFIELD, None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}

fn hdcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_grave_as_cost, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, e.get_description(duel));
    let g = duel.get_field_group(tp, LOCATION_HAND, 0);
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn hdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0);
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, 1 - tp, 0);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 0, 1 - tp, LOCATION_HAND as i32);
    Ok(false)
}

fn hdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_HAND);
    duel.confirm_cards(tp, g)?;
    let sg = g.filter(duel, |c, duel| c.is_attribute(duel, ATTRIBUTE_LIGHT), None);
    if !sg.is_empty() {
        duel.send_to_grave(sg, REASON_EFFECT)?;
        let atk: i32 = sg.into_iter().map(|tc| tc.get_attack(duel).max(0)).sum();
        duel.break_effect()?;
        duel.damage(1 - tp, atk as u32, REASON_EFFECT)?;
    }
    duel.shuffle_hand(1 - tp)?;
    Ok(())
}
