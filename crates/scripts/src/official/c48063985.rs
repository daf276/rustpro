// Ritual Beast Ulti-Cannahawk
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, REMOVED};

const ID: u32 = 48063985;

pub const LISTED_SERIES: &[u16] = &[SET_RITUAL_BEAST, SET_RITUAL_BEAST_TAMER, SET_SPIRITUAL_BEAST];
pub const MATERIAL_SETCODE: &[u16] = &[SET_RITUAL_BEAST, SET_RITUAL_BEAST_TAMER, SET_SPIRITUAL_BEAST];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_RITUAL_BEAST_TAMER], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_SPIRITUAL_BEAST], Some(fc), sumtype, tp)),
    ];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::Int(0));

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(CardId::is_able_to_remove_as_cost, tp, LOCATION_MZONE, 0, None)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, _: CardId) -> Script<()> {
        duel.remove(g, POS_FACEUP, REASON_COST | REASON_MATERIAL).map(|_| ())
    }
}

// In the Extra Deck, contact Summoned with Ritual Beast Tamer Elder and Spiritual Beast Rampengu on the field, which are then
// banished to summon back; banished Ritual Beast monsters to return and more in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (14513016, MZONE),
    (88123329, MZONE),
    (14513016, REMOVED),
    (88123329, REMOVED),
    (88123329, DECK),
    (14513016, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_FREE_CHAIN);
    e4.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_cost(duel, spcost);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_RITUAL_BEAST])
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && chkc.is_controler(duel, tp) && tgfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tgfilter, tp, LOCATION_REMOVED, 0, 2, None)
            && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    let description = e.get_description(duel);
    duel.hint(HINT_OPSELECTED, 1 - tp, description);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, tgfilter, tp, LOCATION_REMOVED, 0, 2, 2, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, 2, 0, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT | REASON_RETURN)?;
        stock::search(duel, a, thfilter, LOCATION_DECK)?;
    }
    Ok(())
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_extra_as_cost(duel));
    }
    duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKTOP, REASON_COST)?;
    Ok(false)
}

fn filter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_RITUAL_BEAST_TAMER])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
        && duel.is_existing_target(|c, duel| filter2(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, Some(c.into()))
}

fn filter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_SPIRITUAL_BEAST])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if e.get_handler(duel).get_sequence(duel) < 5 {
        ft += 1;
    }
    if chk == 0 {
        return Ok(ft > 1
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.is_existing_target(|c, duel| filter1(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, None));
    }
    let description = e.get_description(duel);
    duel.hint(HINT_OPSELECTED, 1 - tp, description);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let mut g1 = duel
        .select_target(tp, |c, duel| filter1(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, 1, None)?
        .unwrap_or_default();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let exception = g1.get_first().map(CardSet::from);
    let g2 = duel.select_target(tp, |c, duel| filter2(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, 1, exception)?;
    g1.merge(g2.unwrap_or_default());
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g1), 2, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if g.is_empty() {
        return Ok(());
    }
    if g.len() as i32 <= ft {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let count = ft.clamp(0, i32::from(u8::MAX)) as u8;
        let sg = g.select(duel, tp, count, count, None)?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        duel.send_to_grave(g.difference(sg), REASON_RULE)?;
    }
    Ok(())
}
