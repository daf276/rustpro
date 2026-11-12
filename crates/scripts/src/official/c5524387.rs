// Marincess Marbled Rock
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 5524387;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(lc), sumtype, tp)
    }
}

// Link Summoned from WATER Normal Monsters (Psychic Kappa, Slime Toad), with Marincess Blue Slug in the Graveyard to add and Marincess Sea Star in hand against an attacker.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (7892180, MZONE),
    (7892180, MZONE),
    (68638985, MZONE),
    (68638985, MZONE),
    (43735670, GRAVE),
    (62886670, HAND),
    (62886670, HAND),
    (62886670, HAND),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, negcost);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MARINCESS]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, a.e)) {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    !duel.get_attacker().is_some_and(|ac| ac.is_controler(duel, a.tp))
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MARINCESS]) && c.is_monster(duel) && c.is_able_to_grave_as_cost(duel)
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, cfilter, 1, 1, REASON_COST)?;
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let (attacker, target) = (duel.get_attacker(), duel.get_attack_target());
    for card in attacker.into_iter().chain(target) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
        card.register_effect(duel, e1);
    }
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_target_range(duel, 1, 0);
    e3.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
    duel.register_effect(e3, tp);
    Ok(())
}
