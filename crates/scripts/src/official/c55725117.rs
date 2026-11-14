// Prank-Kids Dropsies
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 55725117;

pub const LISTED_SERIES: &[u16] = &[SET_PRANK_KIDS];
pub const LISTED_NAMES: &[u32] = &[ID];
// Link Summoned from the field with Prank-Kids Fansies and Lampsies to Special Summon, and Dodo-Doodle-Doo in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (81119816, MZONE),
    (81119816, DECK),
    (18236002, HAND),
    (17382973, EXTRA),
    (17382973, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BE_MATERIAL);
    e1.set_category(duel, CATEGORY_RECOVER + CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, reccon);
    e1.set_target(duel, rectg);
    e1.set_operation(duel, recop);
    c.register_effect(duel, e1);
}

pub fn reccon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE)
        && c.get_reason_card(duel).is_some_and(|rc| rc.is_set_card(duel, &[SET_PRANK_KIDS]))
        && a.r & (REASON_FUSION + REASON_LINK) != 0
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(1000);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 1000);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_PRANK_KIDS])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
        && !c.is_code(duel, ID)
}

fn recop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.recover(p, d, REASON_EFFECT)?;
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK | LOCATION_HAND, 0, None);
    if !g.is_empty() && duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
