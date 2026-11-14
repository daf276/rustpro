// Secret Six Samurai - Rihan
use super::c44686185::repop;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 33964637;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::GroupFilter(ffilter); 3];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::SpSummonCondition(splimit));

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(CardId::is_able_to_grave_as_cost, tp, LOCATION_ONFIELD, 0, None)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, _: CardId) -> Script<()> {
        duel.send_to_grave(g, REASON_COST | REASON_MATERIAL)?;
        Ok(())
    }
}

pub const LISTED_SERIES: &[u16] = &[SET_SIX_SAMURAI];
pub const MATERIAL_SETCODE: &[u16] = &[SET_SIX_SAMURAI];
// In the Extra Deck with Six Samurai of three Attributes (Kageki, Hatsume, Genba) on the field as materials, a copy in the
// Graveyard to replace a destruction by the opponent's Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (2511717, MZONE),
    (44686185, MZONE),
    (7291576, MZONE),
    (2511717, HAND),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Replaced];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_FUSION_MATERIAL);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, rmcost);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_target(duel, reptg);
    e3.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e3.set_operation(duel, repop);
    c.register_effect(duel, e3);
}

fn ffilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, sg: Option<CardSet>) -> bool {
    c.is_set_card_full(duel, &[SET_SIX_SAMURAI], Some(fc), sumtype, tp)
        && c.get_attribute_full(duel, Some(fc), sumtype, tp) != 0
        && sg.is_none_or(|sg| {
            let attribute = c.get_attribute_full(duel, Some(fc), sumtype, tp);
            !sg.is_exists(duel, |mc, duel| fusfilter(mc, duel, attribute, fc, sumtype, tp), 1, Some(c.into()))
        })
}

fn fusfilter(c: CardId, duel: &Duel, attribute: u32, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, attribute, Some(fc), sumtype, tp) && c.is_has_effect(duel, 511002961).is_none()
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, _: u32, _: u8, _: u8) -> bool {
    e.get_handler(duel).get_location(duel) != LOCATION_EXTRA
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SIX_SAMURAI])
        && c.is_able_to_remove_as_cost(duel)
        && ((c.is_on_field(duel) && c.is_faceup(duel)) || c.is_location(duel, LOCATION_HAND))
        && duel.is_existing_target(CardId::is_able_to_remove, 0, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into()))
}

fn rmcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, costfilter, LOCATION_ONFIELD | LOCATION_HAND)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_able_to_remove(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_SIX_SAMURAI])
        && !c.is_reason(duel, REASON_REPLACE)
        && c.is_on_field(duel)
        && c.is_controler(duel, tp)
        && c.is_reason(duel, REASON_EFFECT | REASON_BATTLE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let eg = a.eg.unwrap_or_default();
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel) && eg.is_exists(duel, |tc, duel| repfilter(tc, duel, tp), 1, None));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}
