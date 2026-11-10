// Darkness Resonator
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 83445539;
pub const LISTED_NAMES: &[u32] = &[CARD_RED_DRAGON_ARCHFIEND];

// In the hand with Red Dragon Archfiend in the Extra Deck to reveal, and on the field with Tuners to change the Level of.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_RED_DRAGON_ARCHFIEND, EXTRA),
    (62991792, MZONE),
    (62991792, HAND),
    (98396890, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, revealcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_player_can_additional_summon(a.tp));
    e2.set_target(duel, |duel, a, chk, _| Ok(chk == 0 && duel.is_player_can_summon(a.tp)));
    e2.set_operation(duel, extransop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_LVCHANGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_target(duel, lvtg);
    e3.set_operation(duel, lvop);
    c.register_effect(duel, e3);
}

fn revealcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    Cost::reveal(
        duel,
        a,
        chk,
        |c, duel, _, _| c.is_code(duel, CARD_RED_DRAGON_ARCHFIEND),
        false,
        (1, 1),
        None,
        LOCATION_EXTRA,
    )
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_synchro_monster(duel)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_SYNCHRO));
    Ok(())
}

fn extransop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 4)));
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 5));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e1.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_RESONATOR]));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn lvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TUNER) && !c.is_level(duel, &[1]) && c.is_faceup(duel)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && lvfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(lvfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    let max_ct = duel.get_target_count(lvfilter, tp, LOCATION_MZONE, 0, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, lvfilter, tp, LOCATION_MZONE, 0, 1, max_ct as u8, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_LVCHANGE, g, count, tp, 1);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    for tc in duel.get_target_cards(a.e).unwrap_or_default() {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        // The Lua sets a flag constant as the category.
        e1.set_category(duel, EFFECT_FLAG_CANNOT_DISABLE.into());
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
