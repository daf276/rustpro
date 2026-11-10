// Gordian Slicer
use crate::prelude::*;
use crate::testing::{EXTRA, HAND};

const ID: u32 = 66236707;
const BLUE_EYES_ULTIMATE_DRAGON: u32 = 23995346;

// A copy in the hand and two Fusion Monsters that are rarely summoned in the Extra Deck: with the usual 15 cards there, random
// picks of cards to banish seldom reach the exact count for the most targets, and the duel stalls.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (BLUE_EYES_ULTIMATE_DRAGON, EXTRA), (BLUE_EYES_ULTIMATE_DRAGON, EXTRA)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(
        duel,
        EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CANNOT_NEGATE + EFFECT_FLAG_CANNOT_INACTIVATE,
    );
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, |duel, _| duel.is_main_phase() && !duel.check_phase_activity());
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn removable(tp: u8) -> impl Fn(CardId, &Duel) -> bool {
    move |c, duel| c.is_able_to_remove_full(duel, Some(tp), POS_FACEDOWN, REASON_EFFECT)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_able_to_hand(duel) && chkc.is_faceup(duel) && chkc != c);
    }
    let ct1 = duel.get_matching_group_count(removable(tp), tp, LOCATION_HAND, 0, Some(c.into()));
    let ct2 = duel.get_matching_group_count(removable(tp), tp, LOCATION_EXTRA, 0, None);
    let filter = aux::faceup_filter(CardId::is_able_to_hand);
    if chk == 0 {
        return Ok((ct1 >= 1 || ct2 >= 6) && duel.is_existing_target(&filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(
        tp,
        &filter,
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        (ct1 + ct2 / 6) as u8,
        Some(c.into()),
    )?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, (LOCATION_HAND | LOCATION_EXTRA) as i32);
    Ok(false)
}

fn rescon(duel: &Duel, sg: CardSet, ct: usize) -> (bool, bool) {
    let remaining = ct as i32 - sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None) as i32;
    if remaining < 0 {
        return (false, true);
    }
    let extracnt = sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_EXTRA), None) as i32;
    (extracnt == remaining * 6, extracnt > remaining * 6)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    let ct = tg.len();
    if ct == 0 {
        return Ok(());
    }
    let g1 = duel.get_matching_group(removable(tp), tp, LOCATION_HAND, 0, None);
    let g2 = duel.get_matching_group(removable(tp), tp, LOCATION_EXTRA, 0, None);
    if g1.len() + g2.len() / 6 < ct {
        return Ok(());
    }
    let rg = g1.union(g2);
    let rescon = |duel: &Duel, sg, _, _| rescon(duel, sg, ct);
    let sg = aux::select_unselect_group_break(
        duel,
        rg,
        1,
        rg.len(),
        &rescon,
        tp,
        HINTMSG_REMOVE,
        Some(&rescon),
        Some(&rescon),
        false,
    )?;
    if duel.remove(sg, POS_FACEDOWN, REASON_EFFECT)? > 0 {
        duel.break_effect()?;
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
