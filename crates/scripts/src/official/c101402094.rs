// Angelechy Opposition
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, REMOVED};

const ID: u32 = 101402094;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_ANGELECHY];
// In the hand with Angelechy monsters in the Extra Deck, and in the Graveyard with Angelechy cards to shuffle back.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (81797573, EXTRA),
    (55393975, EXTRA),
    (28904860, EXTRA),
    (17782288, GRAVE),
    (17782288, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, spplcost);
    e1.set_target(duel, sppltg);
    e1.set_operation(duel, spplop);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| {
            !c.is_summon_location(duel, LOCATION_EXTRA)
                || (c.is_synchro_monster(duel) && c.is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK))
        }),
    );
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn spplcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, a.tp, ACTIVITY_SPSUMMON) == 0);
    }
    super::c101402092::register_extra_restriction(duel, a.e, a.tp, aux::stringid(ID, 2));
    Ok(false)
}

pub fn stzone_codes(duel: &Duel, tp: u8) -> Vec<i64> {
    duel.get_matching_group(CardId::is_faceup, tp, LOCATION_STZONE, 0, None)
        .get_class(duel, |c, duel| i64::from(c.get_code(duel)))
}

pub fn spplfilter(c: CardId, duel: &Duel, codes: &[i64]) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY]) && !c.is_forbidden(duel) && !codes.iter().any(|&code| c.is_code(duel, code as u32))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, 1 - tp, 0xff)
        && duel.get_location_count_from_ex_full(1 - tp, tp, None, c, ZONES_MMZ) > 0
}

// The Lua passes 0 as the third argument of Duel.GetLocationCount, the player using the zone.
pub fn szone_count(duel: &Duel, tp: u8) -> i32 {
    duel.get_location_count_full(tp, LOCATION_SZONE, 0, LOCATION_REASON_TOFIELD, 0xff)
}

fn sppltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let stzone_needed = if e.get_handler(duel).is_location(duel, LOCATION_HAND) {
            2
        } else {
            1
        };
        if szone_count(duel, tp) < stzone_needed {
            return Ok(false);
        }
        let codes = stzone_codes(duel, tp);
        let g = duel.get_matching_group(|c, duel| spplfilter(c, duel, &codes), tp, LOCATION_EXTRA, 0, None);
        return Ok(g.len() > 1 && g.is_exists(duel, |c, duel| spfilter(c, duel, e, tp), 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spplop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if szone_count(duel, tp) < 1 {
        return Ok(());
    }
    let codes = stzone_codes(duel, tp);
    let g = duel.get_matching_group(|c, duel| spplfilter(c, duel, &codes), tp, LOCATION_EXTRA, 0, None);
    let rescon = |duel: &Duel, sg: CardSet, _, _| (sg.is_exists(duel, |c, duel| spfilter(c, duel, e, tp), 1, None), false);
    let sg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, aux::stringid(ID, 3))?;
    if sg.len() != 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let Some(spc) = sg
        .filter_select(duel, tp, |c, duel| spfilter(c, duel, e, tp), 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    let Some(plc) = sg.difference(spc.into()).get_first() else {
        return Ok(());
    };
    duel.special_summon_full(spc, 0, tp, 1 - tp, false, false, POS_FACEUP, ZONES_MMZ)?;
    if duel.move_to_field(plc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)? {
        treat_as_continuous_spell(duel, e, plc);
    }
    Ok(())
}

pub fn treat_as_continuous_spell(duel: &mut Duel, e: EffectId, plc: CardId) {
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_CHANGE_TYPE);
    e1.set_value(duel, (TYPE_SPELL | TYPE_CONTINUOUS) as i32);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
    plc.register_effect(duel, e1);
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY]) && !c.is_code(duel, ID) && c.is_faceup(duel) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tdfilter, LOCATION_GRAVE | LOCATION_REMOVED, CATEGORY_TODECK)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(tdfilter),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        5,
        None,
    )?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
