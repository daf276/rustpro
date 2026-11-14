// Mermail Abyssocea
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 28577986;

pub const LISTED_SERIES: &[u16] = &[SET_MERMAIL];
// Mermail Abyssteus summons itself as a Level 7 Mermail to send; Mermail Abyssgunde and Mermail Abysspike in the Deck
// to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (22446869, HAND),
    (22446869, MZONE),
    (26976414, HAND),
    (69293721, DECK),
    (69293721, DECK),
    (58471134, DECK),
    (58471134, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let lv = c.get_level(duel);
    lv > 0
        && c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_MERMAIL])
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, lv, e, tp), tp, LOCATION_DECK, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, lv: u32, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, lv.min(4)) && c.is_set_card(duel, &[SET_MERMAIL]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let f = |c, duel: &Duel| cfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && f(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(f, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, f, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let slv = tc.get_level(duel);
    let sg = duel.get_matching_group(|c, duel| spfilter(c, duel, slv, e, tp), tp, LOCATION_DECK, 0, None);
    if sg.is_empty() {
        return Ok(());
    }
    let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| {
        (sg.get_sum(duel, |c, duel| i64::from(c.get_level(duel))) <= i64::from(slv), false)
    };
    let tg = aux::select_unselect_group(duel, sg, 1, ft as usize, &rescon, tp, HINTMSG_SPSUMMON)?;
    duel.special_summon(tg, 0, tp, tp, false, false, POS_FACEUP)?;
    duel.break_effect()?;
    duel.send_to_grave(tc, REASON_EFFECT)?;
    Ok(())
}
