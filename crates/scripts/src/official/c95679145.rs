// Dogmatika Maximus
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 95679145;

pub const LISTED_NAMES: &[u32] = &[ID];

// A copy in the hand and on the field, Salamangreat Almiraj (Link) in the GY to banish, and Almiraj, M-X-Saber Invoker and Gagaga
// Cowboy in the Extra Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (60303245, GRAVE),
    (60303245, EXTRA),
    (4423206, EXTRA),
    (12014404, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK) && c.is_monster(duel) && c.is_able_to_remove_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, costfilter, LOCATION_GRAVE)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_able_to_grave, tp, LOCATION_EXTRA, 0, None);
    let g2 = duel.get_matching_group(CardId::is_able_to_grave, tp, 0, LOCATION_EXTRA, None);
    if chk == 0 {
        return Ok(g.get_class_count(duel, |c, duel| c.get_code(duel).into()) >= 2 && g2.len() >= 2);
    }
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(CardId::is_able_to_grave, tp, LOCATION_EXTRA, 0, None);
    if g.get_class_count(duel, |c, duel| c.get_code(duel).into()) >= 2 {
        let sg = aux::select_unselect_group(duel, g, 2, 2, &aux::dncheck, tp, HINTMSG_TOGRAVE)?;
        duel.send_to_grave(sg, REASON_EFFECT)?;
    }
    let dg = duel.get_matching_group(CardId::is_able_to_grave, tp, 0, LOCATION_EXTRA, None);
    if dg.len() >= 2 {
        duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_TOGRAVE);
        let tg = dg.select(duel, 1 - tp, 2, 2, None)?;
        duel.send_to_grave(tg, REASON_EFFECT)?;
    }
    let c = e.get_handler(duel);
    let ge1 = duel.create_effect(c);
    ge1.set_type(duel, EFFECT_TYPE_FIELD);
    ge1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    ge1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    ge1.set_target_range(duel, 1, 0);
    ge1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| c.is_location(duel, LOCATION_EXTRA));
    ge1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(ge1, tp);
    aux::register_client_hint(duel, c, EFFECT_FLAG_OATH, tp, 1, 0, Some(aux::stringid(ID, 2)));
    aux::add_temp_lizard_check(duel, c, tp);
    Ok(())
}
