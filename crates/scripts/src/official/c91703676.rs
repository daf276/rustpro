// Fire King Sky Burn
use engine::constants::MSG_SELECT_EFFECTYN;

use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 91703676;

pub const LISTED_SERIES: &[u16] = &[SET_FIRE_KING];
// Copies in hand and the Graveyard, Fire King monsters on the field to destroy and protect, face-up opponent monsters, and the
// opponent's Raigeki to destroy the Fire Kings.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (44455560, MZONE),
    (44455560, MZONE),
    (69000994, MZONE),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
// The destruction replacement is no chain link; its prompt is the MSG_SELECT_EFFECTYN, counted for any card.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_target(duel, reptg);
    e2.set_count_limit(duel, 1, ID);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e2.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        duel.remove(c, POS_FACEUP, REASON_EFFECT)?;
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_controler(duel, 1 - tp) || (c.is_faceup(duel) && c.is_set_card(duel, &[SET_FIRE_KING]))) && c.is_can_be_effect_target(duel, e)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_FIRE_KING])),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        ) && duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(|c, duel| desfilter(c, duel, e, tp), tp, LOCATION_MZONE, LOCATION_ONFIELD, None);
    let ct = g.filter_count(duel, |c, duel| c.is_controler(duel, tp), None);
    let maxc = ct.min(g.len() - ct);
    let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| {
        let ct1 = sg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None);
        let ct2 = sg.len() - ct1;
        (ct1 == ct2, ct1 > maxc || ct2 > maxc)
    };
    let tg = aux::select_unselect_group_full(duel, g, 2, g.len(), &rescon, tp, HINTMSG_DESTROY, Some(&rescon), false)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(tg), tg.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_on_field(duel)
        && c.is_set_card(duel, &[SET_FIRE_KING])
        && c.is_reason(duel, REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel)
            && a.eg
                .unwrap_or_default()
                .is_exists(duel, |tc, duel| repfilter(tc, duel, tp), 1, None));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}
