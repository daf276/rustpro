// Primite Scream
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, SZONE};

const ID: u32 = 27556460;

pub const LISTED_SERIES: &[u16] = &[SET_PRIMITE];
// Set, in the Graveyard and with Primite Dragon Ether Beryl and Mystical Elf (Normal Monster) on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, GRAVE),
    (ID, GRAVE),
    (63198739, MZONE),
    (63198739, MZONE),
    (15025844, MZONE),
    (15025844, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_SUMMON);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, negsumcon);
    e1.set_target(duel, negsumtg);
    e1.set_operation(duel, negsumop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_LEAVE_GRAVE + CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_hint_timing(duel, TIMING_STANDBY_PHASE, TIMING_STANDBY_PHASE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, setcon);
    e3.set_target(duel, settg);
    e3.set_operation(duel, setop);
    c.register_effect(duel, e3);
}

fn negsumconfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_type(duel, TYPE_NORMAL) || (c.is_set_card(duel, &[SET_PRIMITE]) && c.is_level_above(duel, 5)))
        && c.is_faceup(duel)
        && !c.is_type(duel, TYPE_TOKEN)
}

fn negsumcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_current_chain_full(true) == 0
        && a.eg
            .unwrap_or_default()
            .is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - tp), 1, None)
        && duel.is_existing_matching_card(negsumconfilter, tp, LOCATION_MZONE, 0, 1, None)
}

fn negsumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let eg = a.eg.unwrap_or_default();
    if chk == 0 {
        return Ok(eg.is_exists(duel, CardId::is_able_to_remove, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, Some(eg), eg.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(eg), eg.len() as u8, 0, 0);
    Ok(false)
}

fn negsumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let eg = a.eg.unwrap_or_default();
    duel.negate_summon(eg);
    duel.remove(eg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}

fn setcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.is_phase(PHASE_STANDBY)
        && duel.is_turn_player(tp)
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_PRIMITE])),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_ssetable(duel));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_ssetable(duel) && duel.sset(a.tp, c)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect(duel, e1);
    }
    Ok(())
}
