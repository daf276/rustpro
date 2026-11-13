// Gagaga Girl - Cell Phone Subtraction
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 48393693;

// A Level 4 Normal Monster to target and Gagaga Cowboy (Rank 4) to Xyz Summon, against the opponent's Extra Deck Summons.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (2311603, MZONE),
    (2311603, MZONE),
    (12014404, EXTRA),
    (12014404, EXTRA),
    (24094653, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(12014404),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_ATKCHANGE);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_TO_GRAVE);
    e2a.set_count_limit(duel, 1, (ID, 1));
    e2a.set_condition(duel, atkcon);
    e2a.set_target(duel, atktg);
    e2a.set_operation(duel, atkop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_REMOVE);
    c.register_effect(duel, e2b);
}

fn spconfilter(c: CardId, duel: &Duel, opp: u8) -> bool {
    c.is_summon_player(duel, opp) && c.is_summon_location(duel, LOCATION_EXTRA)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| spconfilter(c, duel, 1 - a.tp), 1, None)
}

fn xyzlevel(duel: &mut Duel, hc: CardId, lv: u32) -> EffectId {
    let e1 = duel.create_effect(hc);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_XYZ_LEVEL);
    e1.set_value(duel, lv as i32);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    hc.register_effect_full(duel, e1, true);
    e1
}

fn xyzsummonable(c: CardId, duel: &Duel, mg: CardSet) -> bool {
    c.is_xyz_summonable_full(duel, None, Some(mg), 2, 2)
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8, hc: CardId) -> bool {
    if !(c.is_faceup(duel) && c.has_level(duel)) {
        return false;
    }
    duel.check_mut(|duel| {
        let e1 = xyzlevel(duel, hc, c.get_level(duel));
        let mg = CardSet::from_iter([c, hc]);
        let res = duel.is_existing_matching_card(|xc, duel| xyzsummonable(xc, duel, mg), tp, LOCATION_EXTRA, 0, 1, None);
        e1.reset(duel);
        res
    })
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && tgfilter(chkc, duel, tp, c));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_player_can_special_summon_count(tp, 2)
            && duel.is_existing_target(|tc, duel| tgfilter(tc, duel, tp, c), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |tc, duel| tgfilter(tc, duel, tp, c), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
    {
        let mg = CardSet::from_iter([c, tc]);
        let e1 = xyzlevel(duel, c, tc.get_level(duel));
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = |xc, duel: &Duel| xyzsummonable(xc, duel, mg);
        if let Some(xyz) = duel
            .select_matching_card(tp, filter, tp, LOCATION_EXTRA, 0, 1, 1, None)?
            .get_first()
        {
            duel.xyz_summon_full(tp, xyz, Some(mg), None, 2, 2)?;
        } else {
            e1.reset(duel);
        }
    }
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_COST)
        && a.re.is_some_and(|re| re.is_activated(duel) && re.is_active_type(duel, TYPE_XYZ))
        && c.is_previous_location(duel, LOCATION_OVERLAY)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::has_non_zero_attack, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, None, 1, 1 - tp, 0);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    if let Some(sc) = duel
        .select_matching_card(tp, CardId::has_non_zero_attack, tp, 0, LOCATION_MZONE, 1, 1, None)?
        .get_first()
    {
        duel.hint_selection(sc);
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        sc.register_effect(duel, e1);
    }
    Ok(())
}
