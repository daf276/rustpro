// Ritual Beast Ulti-Apelio
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, REMOVED};

const ID: u32 = 86274272;

pub const LISTED_SERIES: &[u16] = &[SET_RITUAL_BEAST_TAMER, SET_SPIRITUAL_BEAST];
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
        duel.remove(g, POS_FACEUP, REASON_COST | REASON_MATERIAL)?;
        Ok(())
    }
}

// Copies in the Extra Deck, Contact Fused from a Ritual Beast Tamer and a Spiritual Beast on the field, with banished ones
// to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (14513016, MZONE),
    (88123329, MZONE),
    (14513016, REMOVED),
    (88123329, REMOVED),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_FREE_CHAIN);
    e4.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_cost(duel, spcost);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_value(duel, Value::Effect(|duel, e, te| te.get_owner(duel) != e.get_owner(duel)));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
    c.register_effect(duel, e1);
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
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.is_existing_target(|tc, duel| filter2(tc, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, Some(c.into()))
}

fn filter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SPIRITUAL_BEAST]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
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
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let mut g1 = duel
        .select_target(tp, |c, duel| filter1(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, 1, None)?
        .unwrap_or_default();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let exclude = g1.get_first().map(CardSet::from);
    if let Some(g2) = duel.select_target(tp, |c, duel| filter2(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, 1, exclude)? {
        g1.merge(g2);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g1), 2, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let Some(g) = duel.get_target_cards(a.e).filter(|g| !g.is_empty()) else {
        return Ok(());
    };
    if g.len() as i32 <= ft {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let ft = u8::try_from(ft).unwrap_or(u8::MAX);
        let sg = g.select(duel, tp, ft, ft, None)?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        duel.send_to_grave(g.difference(sg), REASON_RULE)?;
    }
    Ok(())
}
