// Sacred Noble Knight of King Custennin
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 78876707;
pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Xyz Summoned with Level 4 Noble Knights (Noble Knight Artorigus, Shield-Bearer), with the opponent's cards to return
// and Noble Knight Xyz Monsters in the Extra Deck to Special Summon once it is destroyed by the opponent's Raigeki and Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (21223277, EXTRA),
    (10613952, EXTRA),
    (92125819, MZONE),
    (92125819, MZONE),
    (34242278, MZONE),
    (34242278, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, SZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> =
        Some(|c, duel, scard, sumtype, tp| c.is_set_card_full(duel, &[SET_NOBLE_KNIGHT], Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_LEAVE_GRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_able_to_hand(duel) && c.is_can_be_effect_target(duel, e)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let e = a.e;
    let max = duel.get_matching_group_count(|c, duel| thfilter(c, duel, e), a.tp, 0, LOCATION_ONFIELD, None) as u16;
    Cost::detach_from_self_with(duel, a, chk, 1, max, |duel, e, og| e.set_label(duel, og.len() as i64))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    let ct = e.get_label(duel) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, ct, ct, None)?;
    let count = g.map_or(0, |g| g.len());
    duel.set_operation_info(0, CATEGORY_TOHAND, g, count as u8, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_DESTROY) && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_NOBLE_KNIGHT])
        && c.is_type(duel, TYPE_XYZ)
        && !c.is_code(duel, ID)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
        sc.complete_procedure(duel);
        if c.is_relate_to_effect(duel, e) {
            duel.overlay(sc, c)?;
        }
    }
    Ok(())
}
