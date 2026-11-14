// Gunkan Suship Uni-class Super-Dreadnought
use super::c75215744::{add_material_effects, regtg};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 94798725;

pub const LISTED_SERIES: &[u16] = &[SET_GUNKAN];

const CARD_SUSHIP_UNI: u32 = 42377643;
pub const LISTED_NAMES: &[u32] = &[CARD_SUSHIP_SHARI, CARD_SUSHIP_UNI];

// In the Extra Deck, Xyz Summoned with two Gunkan Suship Uni (Level 5), against the opponent's Sangan to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_SUSHIP_UNI, MZONE),
    (CARD_SUSHIP_UNI, MZONE),
    (CARD_SUSHIP_UNI, MZONE),
    (CARD_SUSHIP_UNI, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(0),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_hint_timing(duel, TIMING_SPSUMMON, TIMING_BATTLE_START);
    e1.set_condition(duel, discon);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    add_material_effects::<CARD_SUSHIP_UNI>(duel, c, ID, regtg, regop);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    (duel.is_turn_player(tp) && duel.is_main_phase()) || (duel.is_turn_player(1 - tp) && duel.is_battle_phase())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GUNKAN]) && c.is_summon_location(duel, LOCATION_EXTRA)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_negatable(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_ONFIELD));
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_target(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let ct = duel.get_matching_group_count(cfilter, tp, LOCATION_MZONE, 0, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, ct as u8, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let g = duel.get_target_cards(e).unwrap_or_default();
    for tc in g.iter() {
        tc.negate_effects(duel, e.get_handler(duel), RESET_EVENT | RESETS_STANDARD, true);
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let effs = e.get_label(duel);
    if effs & 1 > 0 {
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    if effs & 2 > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3205);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_DIRECT_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    Ok(())
}
