// Performapal Popperup
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 11481610;

pub const LISTED_SERIES: &[u16] = &[SET_PERFORMAPAL, SET_ODD_EYES, SET_MAGICIAN];
// Psychic Kappa, a Normal Monster, in the hand to send as the cost. No Pendulum Monster of the archetypes is ported, so the
// Special Summon is not reached.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (7892180, HAND), (7892180, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn costchk(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    (duel.is_player_can_draw(tp, sg.len()), false)
}

fn cost(duel: &mut Duel, a: &Args, _: i32) -> Script<bool> {
    a.e.set_label(duel, 1);
    Ok(true)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(
        CardId::is_able_to_grave_as_cost,
        tp,
        LOCATION_HAND,
        0,
        Some(e.get_handler(duel).into()),
    );
    if chk == 0 {
        if e.get_label(duel) != 1 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(aux::select_unselect_group_check(duel, g, 1, 3, &|duel, sg, _, _| {
            costchk(duel, sg, tp)
        }));
    }
    let sg = aux::select_unselect_group(duel, g, 1, 3, &|duel, sg, _, _| costchk(duel, sg, tp), tp, HINTMSG_TOGRAVE)?;
    let ct = duel.send_to_grave(sg, REASON_COST)?;
    e.set_label(duel, ct.into());
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ls: u32, rs: u32) -> bool {
    ((c.is_set_card(duel, &[SET_MAGICIAN]) && c.is_type(duel, TYPE_PENDULUM))
        || c.is_set_card(duel, &[SET_ODD_EYES])
        || c.is_set_card(duel, &[SET_PERFORMAPAL]))
        && c.get_level(duel) > ls
        && c.get_level(duel) < rs
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ct = e.get_label(duel) as u32;
    if duel.draw(tp, ct, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let mut summoned = false;
    let (pc1, pc2) = (
        duel.get_field_card(tp, LOCATION_PZONE, 0),
        duel.get_field_card(tp, LOCATION_PZONE, 1),
    );
    if let (Some(pc1), Some(pc2)) = (pc1, pc2)
        && pc1.is_faceup(duel)
        && pc2.is_faceup(duel)
    {
        let (mut ls, mut rs) = (pc1.get_left_scale(duel), pc2.get_right_scale(duel));
        if ls > rs {
            (ls, rs) = (rs, ls);
        }
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp, ls, rs), tp, LOCATION_HAND, 0, None);
        let lc = duel.get_location_count(tp, LOCATION_MZONE);
        if !g.is_empty() && lc > 0 && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
            let sg = aux::select_unselect_group(duel, g, 1, ct.min(lc as u32) as usize, &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
            summoned = true;
        }
    }
    if !summoned {
        let lp = duel.get_lp(tp) - 1000 * duel.get_field_group_count(tp, LOCATION_HAND, 0) as i32;
        duel.set_lp(tp, lp.max(0));
    }
    Ok(())
}
