// Xyz Import
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 14602126;

// An Xyz Monster of yours and a monster of the opponent with lower ATK to attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (1621413, MZONE),
    (16195942, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (ID, HAND),
    (ID, HAND),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    let timing = TIMINGS_CHECK_MONSTER | TIMING_BATTLE_START;
    e1.set_hint_timing(duel, timing, timing);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, acttg);
    e1.set_operation(duel, actop);
    c.register_effect(duel, e1);
}

fn atchfilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_faceup(duel) && !c.is_type(duel, TYPE_TOKEN) && c.is_attack_below(duel, atk)
}

fn xyzfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let atk = c.get_attack(duel);
    c.is_faceup(duel)
        && c.is_monster(duel)
        && c.is_type(duel, TYPE_XYZ)
        && duel.is_existing_target(|c, duel| atchfilter(c, duel, atk), tp, 0, LOCATION_MZONE, 1, None)
}

fn acttg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| xyzfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    let xyzc = duel
        .select_target(tp, |c, duel| xyzfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?
        .and_then(|g| g.get_first());
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let atk = xyzc.map_or(0, |xyzc| xyzc.get_attack(duel));
    duel.select_target(tp, |c, duel| atchfilter(c, duel, atk), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    e.set_label_object_group(duel, xyzc.map(CardSet::from));
    Ok(false)
}

fn actop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_target_cards(e).unwrap_or_default();
    if g.len() <= 1 {
        return Ok(());
    }
    let mut cards = g.into_iter();
    let (c1, c2) = (cards.next(), cards.next());
    let (Some(xyzc), Some(tgc)) = (if c1 == e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        (c1, c2)
    } else {
        (c2, c1)
    }) else {
        return Ok(());
    };
    if xyzc.is_relate_to_effect(duel, e)
        && xyzc.is_controler(duel, tp)
        && tgc.is_relate_to_effect(duel, e)
        && tgc.is_controler(duel, 1 - tp)
        && !xyzc.is_immune_to_effect(duel, e)
        && !tgc.is_immune_to_effect(duel, e)
    {
        duel.overlay_full(xyzc, tgc, true)?;
    }
    Ok(())
}
