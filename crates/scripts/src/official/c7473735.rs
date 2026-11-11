// Harmonic Synchro Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 7473735;

// In the hand with Lunar Rabbit Omajinai (a LIGHT Tuner) and Key Mace (LIGHT, another Race) on the field, and with Mudragon of the
// Swamp (two Monsters of one Attribute and different Races), Formula Synchron and Angelechy Enlisted (Level 2) in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (47643326, MZONE),
    (47643326, MZONE),
    (1929294, MZONE),
    (1929294, MZONE),
    (1929294, MZONE),
    (54757758, EXTRA),
    (50091196, EXTRA),
    (81797573, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(54757758)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(counterfilter));
}

fn counterfilter(duel: &Duel, c: CardId) -> bool {
    !(c.is_summon_location(duel, LOCATION_EXTRA) && !c.is_type(duel, TYPE_FUSION | TYPE_SYNCHRO))
}

fn cfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_type(duel, TYPE_TUNER)
        && c.is_able_to_grave_as_cost(duel)
        && !c.is_original_type(duel, TYPE_TRAP)
        && duel.is_existing_matching_card(|sc, duel| cfilter2(sc, duel, e, tp, c), tp, LOCATION_MZONE, 0, 1, Some(c.into()))
}

fn cfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8, tun: CardId) -> bool {
    if !(c.is_faceup(duel) && !c.is_type(duel, TYPE_TUNER) && c.is_able_to_grave_as_cost(duel) && !c.is_original_type(duel, TYPE_TRAP)) {
        return false;
    }
    let g = CardSet::from_iter([tun, c]);
    for tc in g {
        tc.assume_property(duel, ASSUME_CODE, u64::from(tc.get_original_code_rule(duel)));
        tc.assume_property(duel, ASSUME_TYPE, u64::from(tc.get_original_type(duel)));
        tc.assume_property(duel, ASSUME_LEVEL, u64::from(tc.get_original_level(duel)));
        tc.assume_property(duel, ASSUME_ATTRIBUTE, u64::from(tc.get_original_attribute(duel)));
        tc.assume_property(duel, ASSUME_RACE, tc.get_original_race(duel));
        tc.assume_property(duel, ASSUME_ATTACK, i64::from(tc.get_text_attack(duel)) as u64);
        tc.assume_property(duel, ASSUME_DEFENSE, i64::from(tc.get_text_defense(duel)) as u64);
    }
    let chk = duel.get_location_count_from_ex(tp, tp, Some(g), TYPE_FUSION) >= 2
        && duel.is_existing_matching_card(|fc, duel| fusfilter(fc, duel, e, tp, g), tp, LOCATION_EXTRA, 0, 1, None)
        && duel.is_existing_matching_card(|sc, duel| syncfilter(sc, duel, e, tp, g), tp, LOCATION_EXTRA, 0, 1, None);
    duel.assume_reset();
    chk
}

// The Fusion Monsters whose script sets s.material_location to LOCATION_ONFIELD, which has no LOCATION_GRAVE bit.
fn fusfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mg: CardSet) -> bool {
    if !(c.is_facedown(duel) && c.is_type(duel, TYPE_FUSION) && ![47172959, 41209827, 27118421].contains(&c.get_original_code(duel))) {
        return false;
    }
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && c.check_fusion_material(duel, Some(mg), None, u64::from(PLAYER_NONE))
}

fn syncfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mg: CardSet) -> bool {
    if !(c.is_facedown(duel) && c.is_type(duel, TYPE_SYNCHRO)) {
        return false;
    }
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && c.is_synchro_summonable(duel, None, Some(mg))
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, 100);
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0
            && duel.is_existing_matching_card(|c, duel| cfilter1(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let tun = duel
        .select_matching_card(tp, |c, duel| cfilter1(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?
        .get_first();
    let Some(tun) = tun else { return Ok(false) };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let nt = duel.select_matching_card(
        tp,
        |c, duel| cfilter2(c, duel, e, tp, tun),
        tp,
        LOCATION_MZONE,
        0,
        1,
        1,
        Some(tun.into()),
    )?;
    let Some(nt) = nt.get_first() else { return Ok(false) };
    let g = CardSet::from_iter([tun, nt]);
    duel.send_to_grave(g, REASON_COST)?;
    duel.set_target_card(g);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_type(duel, TYPE_FUSION | TYPE_SYNCHRO)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_FUSION | TYPE_SYNCHRO));
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if e.get_label(duel) != 100 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none());
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mg = duel.get_target_cards(e).unwrap_or_default();
    if mg.len() != 2
        || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some()
        || duel.get_location_count_from_ex(tp, tp, Some(mg), TYPE_FUSION) < 2
    {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let fc = duel.select_matching_card(tp, |c, duel| fusfilter(c, duel, e, tp, mg), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(fc) = fc.get_first() else { return Ok(()) };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sc = duel.select_matching_card(tp, |c, duel| syncfilter(c, duel, e, tp, mg), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(sc) = sc.get_first() else { return Ok(()) };
    duel.special_summon(CardSet::from_iter([fc, sc]), 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}
