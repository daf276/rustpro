// Plunder Patrollship Jord
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 85969517;
const TOKEN_PLUNDER_PATROLL: u32 = ID + 1;

// The Lua assigns listed_names twice, so only the archetype code of the second assignment counts.
pub const LISTED_NAMES: &[u32] = &[SET_PLUNDER_PATROLL as u32];

// In the Extra Deck, Synchro Summoned with Harmonizing Magician (a Pendulum Tuner, which ends up face-up in the Extra Deck) and Kojikocy
// (Level 4), in the Pendulum Zone to return, with Whitebeard in the Graveyard to add and in the Deck to search,
// and the opponent's Cyber Dragons to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, PZONE),
    (73941492, MZONE),
    (73941492, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (31374201, GRAVE),
    (31374201, GRAVE),
    (31374201, DECK),
    (31374201, DECK),
    (70095154, HAND | OPPONENT),
    (70095154, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::SpecialSummonedCode(TOKEN_PLUNDER_PATROLL),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Pendulum::add_procedure_full(duel, c, false, None);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_TOKEN + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, dthcon);
    e2.set_target(duel, dthtg);
    e2.set_operation(duel, dthop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_target(duel, gthtg);
    e3.set_operation(duel, gthop);
    c.register_effect(duel, e3);
}

fn attcheck(duel: &mut Duel, tp: u8, att: u32, targ_p: u8) -> bool {
    let stats = MonsterStats::new(SET_PLUNDER_PATROLL, TYPES_TOKEN, 0, 0, 4, RACE_FIEND, att);
    duel.is_player_can_special_summon_monster_full(tp, TOKEN_PLUNDER_PATROLL, stats, POS_FACEUP_DEFENSE, targ_p, 0)
}

fn getvalidatts(duel: &mut Duel, tp: u8) -> u32 {
    let mut res = ATTRIBUTE_ALL;
    let mut att = ATTRIBUTE_EARTH;
    while att < ATTRIBUTE_ALL {
        if !attcheck(duel, tp, att, tp) || !attcheck(duel, tp, att, 1 - tp) {
            res &= !att;
        }
        att <<= 1;
    }
    res
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let valid_atts = getvalidatts(duel, tp);
    if chk == 0 {
        return Ok(valid_atts > 0
            && c.is_able_to_extra(duel)
            && duel.get_mzone_count(tp, Some(c.into())) > 0
            && duel.get_mzone_count_full(1 - tp, Some(c.into()), tp, LOCATION_REASON_TOFIELD, 0xff) > 0
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none());
    }
    let att = duel.announce_attribute(tp, 1, valid_atts)?;
    duel.set_target_param(att);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e)
        || duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? == 0
        || !c.is_location(duel, LOCATION_EXTRA)
    {
        return Ok(());
    }
    let att: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if !(att > 0
        && attcheck(duel, tp, att, tp)
        && attcheck(duel, tp, att, 1 - tp)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none())
    {
        return Ok(());
    }
    for p in [tp, 1 - tp] {
        let Some(token) = duel.create_token(tp, TOKEN_PLUNDER_PATROLL) else {
            continue;
        };
        token.set_attribute(duel, att);
        duel.special_summon_step(token, 0, tp, p, false, false, POS_FACEUP_DEFENSE)?;
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn dthcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - a.tp), 1, None))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PLUNDER_PATROLL]) && c.is_able_to_hand(duel)
}

fn dthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_PLUNDER_PATROLL]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn dthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? == 0 || !tc.is_location(duel, LOCATION_HAND) {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    if e.get_handler(duel).get_equip_group(duel).is_exists(
        duel,
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_PLUNDER_PATROLL])),
        1,
        None,
    ) && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if sg.is_empty() {
            return Ok(());
        }
        duel.break_effect()?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn gthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None)
            && a.e.get_handler(duel).is_type(duel, TYPE_PENDULUM)
            && duel.check_pendulum_zones(tp));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn gthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let c = e.get_handler(duel);
    if tc.is_relate_to_effect(duel, e)
        && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND)
        && duel.check_pendulum_zones(tp)
        && c.is_relate_to_effect(duel, e)
        && !c.is_forbidden(duel)
    {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
