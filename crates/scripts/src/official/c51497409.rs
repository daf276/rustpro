// D/D/D Stone King Darius
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 51497409;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_FIEND, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

// On the field with Xyz Import to attach Sangan, Dark Contract with the Witch to activate as the target and Elfnote Lucina to attack into it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (9765723, HAND),
    (9765723, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, ddtg);
    e1.set_operation(duel, ddop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e2.set_condition(duel, incon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_operation(duel, inop);
    c.register_effect(duel, e2);
}

fn ddfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DARK_CONTRACT])
}

fn ddtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && ddfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1) && duel.is_existing_target(ddfilter, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, ddfilter, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn ddop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && duel.destroy(tc, REASON_EFFECT)? > 0 {
        duel.draw(a.tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}

fn incon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_battle_target(duel)
        .is_some_and(|bc| bc.is_controler(duel, 1 - a.tp))
}

fn inop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
        e2.set_code(duel, EVENT_BATTLED);
        e2.set_operation(duel, desop);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let dc = a.e.get_handler(duel).get_battle_target(duel);
    if let Some(dc) = dc
        && duel.destroy(dc, REASON_EFFECT)? > 0
    {
        duel.damage(1 - a.tp, 500, REASON_EFFECT)?;
    }
    Ok(())
}
