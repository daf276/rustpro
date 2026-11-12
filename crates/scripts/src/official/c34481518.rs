// Grenosaurus Giga-Cannon
use super::c71594310::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 34481518;

// In the Extra Deck, Xyz Summoned with Level 4 Dinosaurs (Uraby), with Dinosaurs (Frostosaurus) to destroy
// the opponent's monsters by battle (Sangan, Lucky Trinket) and Raigeki on the opponent's side to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (1784619, MZONE),
    (1784619, MZONE),
    (6631034, MZONE),
    (6631034, MZONE),
    (26202165, MZONE | OPPONENT),
    (3985011, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (12580477, DECK | OPPONENT),
    (12580477, DECK | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure<4, 2>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, damcon);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, damtg);
    e1.set_operation(duel, damop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    if !(c.is_previous_controler(duel, 1 - tp) && c.is_location(duel, LOCATION_GRAVE)) {
        return false;
    }
    let Some(bc) = c.get_reason_card(duel) else { return false };
    if bc.is_relate_to_battle(duel) {
        bc.is_controler(duel, tp) && bc.is_race(duel, RACE_DINOSAUR)
    } else {
        bc.is_previous_controler(duel, tp) && c.get_previous_race_on_field(duel) & RACE_DINOSAUR > 0
    }
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| cfilter(c, duel, tp), 1, None)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(1000);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1000);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.damage(p, d, REASON_EFFECT)? > 0 {
        let c = e.get_handler(duel);
        if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel)) {
            return Ok(());
        }
        duel.break_effect()?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 1000);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_xyz_summoned(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1000);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && duel.destroy(tc, REASON_EFFECT)? > 0 {
        duel.damage(1 - a.tp, 1000, REASON_EFFECT)?;
    }
    Ok(())
}
