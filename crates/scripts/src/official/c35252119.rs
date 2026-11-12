// Virtual World Beast - Jiujiu
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 35252119;

// A copy on the field and two in the Extra Deck, Synchro Summoned with Guardragon Justicia (a Level 2 Tuner) and Prevent Rat
// (Level 4), with three WATER Aqua Normal Monsters of different names in the Graveyard and on the field to banish and to
// make it indestructible.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (11012154, MZONE),
    (549481, MZONE),
    (2118022, GRAVE),
    (4042268, GRAVE),
    (2483611, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, indcon);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, gycost);
    e3.set_target(duel, gytg);
    e3.set_operation(duel, gyop);
    c.register_effect(duel, e3);
}

fn indfilter2(c: CardId, duel: &Duel, tc: CardId) -> bool {
    c.is_monster(duel)
        && c.get_original_attribute(duel) == tc.get_original_attribute(duel)
        && c.get_original_race(duel) == tc.get_original_race(duel)
        && !c.is_code(duel, tc.get_code(duel))
}

fn indfilter1(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel) && duel.is_existing_matching_card(|g, duel| indfilter2(g, duel, c), tp, LOCATION_GRAVE, 0, 1, Some(c.into()))
}

fn indcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.is_existing_matching_card(|c, duel| indfilter1(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, None)
}

fn gycfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn gycheck(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    (
        sg.get_class_count(duel, |c, duel| i64::from(c.get_original_attribute(duel))) == 1
            && sg.get_class_count(duel, |c, duel| c.get_original_race(duel) as i64) == 1
            && sg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) == sg.len()
            && duel.is_existing_target(CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(sg)),
        false,
    )
}

fn gycost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(gycfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| {
            gycheck(duel, sg, tp)
        }));
    }
    let sg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| gycheck(duel, sg, tp), tp, HINTMSG_REMOVE)?;
    duel.remove(sg, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn gytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_able_to_grave(duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, 1, 0, 0);
    Ok(false)
}

fn gyop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_grave(tc, REASON_EFFECT)?;
    }
    Ok(())
}
