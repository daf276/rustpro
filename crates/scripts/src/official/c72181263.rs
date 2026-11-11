// D/D Orthros
use crate::prelude::*;
use crate::testing::{Check, HAND, PZONE, SZONE};

const ID: u32 = 72181263;

// In the Pendulum Zone and the hand, with Dark Contract with the Witch (Spells, and a damage source) on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, HAND),
    (ID, HAND),
    (9765723, SZONE),
    (9765723, SZONE),
    (9765723, SZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_DAMAGE);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_condition(duel, |_, a| a.ep == a.tp);
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_operation(duel, regop);
    c.register_effect(duel, e3);
}

fn desfilter1(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_spell_trap(duel) && c.is_can_be_effect_target(duel, e)
}

fn desfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DD, SET_DARK_CONTRACT]) && c.is_controler(duel, tp) && c.is_can_be_effect_target(duel, e)
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> (bool, bool) {
    let handler = e.get_handler(duel);
    (
        sg.is_exists(duel, |c, duel| desfilter1(c, duel, e), 1, None)
            && sg.is_exists(duel, |c, duel| desfilter2(c, duel, e, tp), 1, Some(handler.into())),
        false,
    )
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g1 = duel.get_matching_group(|c, duel| desfilter1(c, duel, e), tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    let g2 = duel.get_matching_group(|c, duel| desfilter2(c, duel, e, tp), tp, LOCATION_ONFIELD, 0, None);
    let rg = g1.union(g2);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, rg, 2, 2, &|duel, sg, _, _| {
            rescon(duel, sg, e, tp)
        }));
    }
    let tg = aux::select_unselect_group(duel, rg, 2, 2, &|duel, sg, _, _| rescon(duel, sg, e, tp), tp, HINTMSG_DESTROY)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(tg), 2, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    if let Some(g) = duel.get_target_cards(a.e)
        && !g.is_empty()
    {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_race(duel, RACE_FIEND));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
