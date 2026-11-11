// Vampire Lord
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 53839837;

// On the field and in the GY, with the opponent's Dark Hole to destroy it, and Sangans to hit for battle damage.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, GRAVE),
    (53129443, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 4)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_operation(duel, spr);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 4));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_BATTLE_DAMAGE);
    e3.set_condition(duel, |_, a| a.ep != a.tp);
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
}

fn spr(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if a.r & (REASON_DESTROY | REASON_EFFECT) != (REASON_DESTROY | REASON_EFFECT) || a.rp == tp || !c.is_previous_controler(duel, tp) {
        return Ok(());
    }
    let reset = RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN;
    if duel.is_turn_player(tp) && duel.is_phase(PHASE_STANDBY) {
        let turn = duel.get_turn_count();
        e.set_label(duel, i64::from(turn));
        c.register_flag_effect(duel, ID, reset, 0, 2);
    } else {
        e.set_label(duel, 0);
        c.register_flag_effect(duel, ID, reset, 0, 1);
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let label = a.e.get_label_object(duel).map_or(0, |e| e.get_label(duel));
    label != i64::from(duel.get_turn_count()) && a.tp == duel.get_turn_player() && a.e.get_handler(duel).get_flag_effect(duel, ID) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    c.reset_flag_effect(duel, ID);
    Ok(false)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 5));
    let op = duel.select_option(tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2), aux::stringid(ID, 3)])?;
    a.e.set_label(duel, i64::from(op));
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, 1 - tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel, typ: u32) -> bool {
    c.is_type(duel, typ) && c.is_able_to_grave(duel)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let typ = match a.e.get_label(duel) {
        0 => TYPE_MONSTER,
        1 => TYPE_SPELL,
        _ => TYPE_TRAP,
    };
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(1 - tp, |c, duel| tgfilter(c, duel, typ), 1 - tp, LOCATION_DECK, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_EFFECT)?;
    Ok(())
}
