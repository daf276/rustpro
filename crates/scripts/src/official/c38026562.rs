// Vola-Chemicritter Methydraco
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 38026562;

// Twice in the Extra Deck, Xyz Summoned with Darkstorm Dragons (Level 8 Gemini), with Gemini Lancers in the Graveyard and the hand to
// Special Summon and Normal Summon, and Sangans on the opponent's side to send to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (57662975, MZONE),
    (57662975, MZONE),
    (57662975, MZONE),
    (26254876, GRAVE),
    (26254876, HAND),
    (26254876, HAND),
    (26254876, HAND),
    (26254876, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_type_full(duel, TYPE_GEMINI, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_condition(duel, con);
    e2.set_value(
        duel,
        Value::Card(|duel, _, c| i32::from(c.is_faceup(duel) && c.is_type(duel, TYPE_GEMINI))),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_condition(duel, con);
    e3.set_target_card(duel, |duel, _, c| c.is_type(duel, TYPE_GEMINI));
    e3.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_TOGRAVE);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_SUMMON_SUCCESS);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_type(duel, TYPE_GEMINI), 1, None))
    });
    e4.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e4.set_target(duel, tgtg);
    e4.set_operation(duel, tgop);
    c.register_effect(duel, e4);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_GEMINI) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn con(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_overlay_count(duel) > 0
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, 0, LOCATION_ONFIELD | LOCATION_HAND) > 0);
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, 0, (LOCATION_ONFIELD | LOCATION_HAND) as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::TRUE, 1 - tp, LOCATION_ONFIELD | LOCATION_HAND, 0, None);
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_TOGRAVE);
        let sg = g.select(duel, 1 - tp, 1, 1, None)?;
        duel.hint_selection_full(sg, true);
        duel.send_to_grave_full(sg, REASON_RULE, PLAYER_NONE, 1 - tp)?;
    }
    Ok(())
}
