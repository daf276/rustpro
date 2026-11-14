// Lunalight Serenade Dance
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 13935001;

const LUNALIGHT_PERFUME_DANCER: u32 = 81196066;

pub const LISTED_NAMES: &[u32] = &[13935002];
pub const LISTED_SERIES: &[u16] = &[SET_LUNALIGHT];

// On the field and in hand, with Polymerization and Lunalight Silver Hound and Gold Leo in hand and on the field to Fusion
// Summon Lunalight Perfume Dancer, and in the GY with Silver Hound in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (ID, GRAVE),
    (CARD_POLYMERIZATION, HAND),
    (LUNALIGHT_PERFUME_DANCER, EXTRA),
    (LUNALIGHT_PERFUME_DANCER, EXTRA),
    (35763582, HAND),
    (8379983, MZONE),
    (35763582, DECK),
    (35763582, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(ID + 1),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_FIELD);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| cfilter(c, duel, a.e, a.tp), 1, None)
    });
    e2.set_target(duel, tktg);
    e2.set_operation(duel, tkop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(a.tp) && duel.is_main_phase());
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_type(duel, TYPE_FUSION)
        && c.is_controler(duel, tp)
        && c.is_fusion_summoned(duel)
        && c.is_can_be_effect_target(duel, e)
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(SET_LUNALIGHT, TYPES_TOKEN, 2000, 2000, 1, RACE_BEASTWARRIOR, ATTRIBUTE_DARK);
    duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0
        && duel.is_player_can_special_summon_monster_full(tp, ID + 1, stats, POS_FACEUP, 1 - tp, 0)
}

fn tktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let eg = a.eg.unwrap_or_default();
    if let Some(chkc) = chkc {
        return Ok(eg.contains(chkc) && cfilter(chkc, duel, e, tp));
    }
    let filter = |c, duel: &Duel| cfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(eg.is_exists(duel, filter, 1, None) && can_summon_token(duel, tp));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = eg.filter_select(duel, tp, filter, 1, 1, None)?;
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 0, 0);
    Ok(false)
}

fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    if can_summon_token(duel, tp)
        && let Some(token) = duel.create_token(tp, ID + 1)
    {
        duel.special_summon(token, 0, tp, 1 - tp, false, false, POS_FACEUP)?;
    }
    let atk = duel.get_field_group_count(tp, 0, LOCATION_MZONE) as i32 * 500;
    if let Some(tc) = tc
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        let c = e.get_handler(duel);
        tc.update_attack(duel, atk, Some(RESET_EVENT | RESETS_STANDARD), Some(c), None);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_LUNALIGHT]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(CardId::is_able_to_grave, tp, LOCATION_HAND, 0, 1, None)
                && duel.get_location_count(tp, LOCATION_MZONE) > 0
                && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_HAND as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let tgc = duel
        .select_matching_card(tp, CardId::is_able_to_grave, tp, LOCATION_HAND, 0, 1, 1, None)?
        .get_first();
    if let Some(tgc) = tgc
        && duel.send_to_grave(tgc, REASON_EFFECT)? > 0
        && tgc.is_location(duel, LOCATION_GRAVE)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !sg.is_empty() {
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
