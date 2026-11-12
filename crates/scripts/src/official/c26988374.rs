// Magikey Deity - Ashtartu
use super::c71594310::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};
const ID: u32 = 26988374;

// On the field with Xyz Import to attach Sangan, or Xyz Summoned with Level 8 Normal Monsters (Tri-Horned Dragon, Gogiga Gagagigo), with
// Black Luster Soldier to attack the opponent's Sangan and Normal Monsters in the Graveyard for the attribute match.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (ID, EXTRA),
    (ID, EXTRA),
    (39111158, MZONE),
    (39674352, MZONE),
    (10000100, MZONE),
    (10000100, MZONE),
    (39111158, GRAVE),
    (39674352, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure<8, 2>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, damcon);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, damtg);
    e1.set_operation(duel, stock::damage);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp) && duel.is_main_phase());
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let (Some(rc), Some(bc)) = duel.get_battle_monster(tp) else {
        return false;
    };
    bc.get_base_attack(duel) > 0
        && (rc.is_type(duel, TYPE_NORMAL) || rc.is_set_card(duel, &[SET_MAGIKEY]))
        && rc.is_controler(duel, tp)
        && bc.is_previous_controler(duel, 1 - tp)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let dam = duel.get_battle_monster(tp).1.map_or(0, |bc| bc.get_base_attack(duel));
    if chk == 0 {
        return Ok(dam > 0);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}

fn attrfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && (c.is_type(duel, TYPE_NORMAL) || c.is_set_card(duel, &[SET_MAGIKEY])) && c.is_faceup(duel)
}

fn rmfilter(c: CardId, duel: &Duel, ag: CardSet) -> bool {
    if !(c.is_faceup(duel) && c.is_able_to_remove(duel)) {
        return false;
    }
    let attribute = c.get_attribute(duel);
    ag.is_exists(duel, |c, duel| c.is_attribute(duel, attribute), 1, None)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ag = duel.get_matching_group(attrfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && rmfilter(chkc, duel, ag));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| rmfilter(c, duel, ag), tp, 0, LOCATION_MZONE, 1, None)
                && a.e.get_handler(duel).check_remove_overlay_card(duel, tp, 1, REASON_EFFECT),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, |c, duel| rmfilter(c, duel, ag), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, g.map_or(0, |g| g.len()) as u8, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)? > 0
        && let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
