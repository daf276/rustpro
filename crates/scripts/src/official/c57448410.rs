// Prima Materiactor
use crate::prelude::*;
use crate::testing::{Check, MZONE, PZONE};

const ID: u32 = 57448410;

// In the Pendulum Zone, with Materiactor Xyz Monsters on the field to attach to.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (70597485, MZONE), (72409226, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_xyz_monster(duel));
    e1.set_value(duel, Value::Card(|duel, _, _| (duel.get_overlay_count(0, 1, 1) * 100) as i32));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, attachtg);
    e2.set_operation(duel, attachop);
    c.register_effect(duel, e2);
}

fn xyzfilter(c: CardId, duel: &Duel, tp: u8, hc: CardId) -> bool {
    c.is_set_card(duel, &[SET_MATERIACTOR])
        && c.is_xyz_monster(duel)
        && c.is_faceup(duel)
        && hc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && xyzfilter(chkc, duel, tp, c));
    }
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1)
            && duel.is_existing_target(|c2, duel| xyzfilter(c2, duel, tp, c), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c2, duel| xyzfilter(c2, duel, tp, c), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
        && !tc.is_immune_to_effect(duel, e)
    {
        duel.overlay(tc, c)?;
        if duel.is_player_can_draw(tp, 0) {
            duel.break_effect()?;
            duel.draw(tp, 1, REASON_EFFECT)?;
        }
    }
    Ok(())
}
