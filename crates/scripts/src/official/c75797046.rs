// Photon Alexandra Queen
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 75797046;

pub const LISTED_SERIES: &[u16] = &[SET_BUTTERSPY];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_set_card_full(duel, &[SET_BUTTERSPY], Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

// Monsters on both sides to return.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, MZONE | OPPONENT), (26202165, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, rettg);
    e1.set_operation(duel, retop);
    c.register_effect(duel, e1);
}

fn rettg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_hand, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_able_to_hand, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, PLAYER_ALL, 0);
    Ok(false)
}

fn hfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_location(duel, LOCATION_HAND) && c.is_controler(duel, tp)
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_able_to_hand, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if g.is_empty() {
        return Ok(());
    }
    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    let ct1 = g.filter_count(duel, |c, duel| hfilter(c, duel, tp), None) as u32;
    let ct2 = g.filter_count(duel, |c, duel| hfilter(c, duel, 1 - tp), None) as u32;
    duel.damage_full(tp, ct1 * 300, REASON_EFFECT, true, None)?;
    duel.damage_full(1 - tp, ct2 * 300, REASON_EFFECT, true, None)?;
    duel.rd_complete()?;
    Ok(())
}
