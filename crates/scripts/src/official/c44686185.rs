// Secret Six Samurai - Hatsume
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 44686185;

pub const LISTED_SERIES: &[u16] = &[SET_SIX_SAMURAI];
pub const LISTED_NAMES: &[u32] = &[ID];

// On the field and in the Graveyard with Legendary Six Samurai - Kageki to banish and to Special Summon, and the opponent's
// Raigeki to destroy a Six Samurai monster, for the copy in the Graveyard to replace.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (2511717, GRAVE),
    (2511717, GRAVE),
    (2511717, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Replaced];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, ID);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_target(duel, reptg);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e2.set_operation(duel, repop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SIX_SAMURAI])
        && c.is_monster(duel)
        && c.is_able_to_remove_as_cost(duel)
        && (c.is_location(duel, LOCATION_GRAVE) || c.is_faceup(duel))
        && aux::sp_elim_filter(c, duel, true, true)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SIX_SAMURAI]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(cfilter, tp, LOCATION_GRAVE | LOCATION_MZONE, 0, None);
    let rescon = |duel: &Duel, sg: CardSet, mg: CardSet, c: Option<CardId>| {
        let ok = aux::chkf_mmz(tp, 1)(duel, sg, mg, c).0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, Some(sg));
        (ok, false)
    };
    if chk == 0 {
        return Ok(g.len() > 1 && aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let sg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_REMOVE)?;
    duel.remove(sg, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    // The Lua calls the undefined s.filter2 for chkc and errors.
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

pub fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_SIX_SAMURAI])
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_controler(duel, tp)
        && c.is_reason(duel, REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let eg = a.eg.unwrap_or_default();
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel) && eg.is_exists(duel, |tc, duel| repfilter(tc, duel, tp), 1, None) && eg.len() == 1);
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}

pub fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.remove(a.e.get_handler(duel), POS_FACEUP, REASON_EFFECT | REASON_REPLACE)?;
    Ok(())
}
