// Lunalight Kaleido Chick
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 35618217;
const LUNALIGHT_BLACK_SHEEP: u32 = 11317977;
const DARK_HOLE: u32 = 53129443;

pub const LISTED_NAMES: &[u32] = &[CARD_POLYMERIZATION];

// On the field with Lunalight Black Sheep in the Deck to send, and Polymerization in the Graveyard to retrieve once the
// opponent's Dark Hole destroys it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (LUNALIGHT_BLACK_SHEEP, DECK),
    (LUNALIGHT_BLACK_SHEEP, DECK),
    (CARD_POLYMERIZATION, GRAVE),
    (DARK_HOLE, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, cost);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |_, a| a.r & REASON_EFFECT != 0);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_REMOVE);
    e3.set_operation(duel, actop);
    c.register_effect(duel, e3);
}

fn costfilter(c: CardId, duel: &Duel, ec: CardId) -> bool {
    c.is_set_card(duel, &[SET_LUNALIGHT])
        && c.is_monster(duel)
        && c.is_able_to_grave_as_cost(duel)
        && !c.is_summon_code(
            duel,
            None,
            u64::from(SUMMON_TYPE_FUSION),
            PLAYER_NONE,
            &ec.get_codes_full(duel, None, u64::from(SUMMON_TYPE_FUSION), duel.get_reason_player()),
        )
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let location = LOCATION_DECK | LOCATION_EXTRA;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|g, duel| costfilter(g, duel, c), tp, location, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let cg = duel.select_matching_card(tp, |g, duel| costfilter(g, duel, c), tp, location, 0, 1, 1, None)?;
    duel.send_to_grave(cg, REASON_COST)?;
    if let Some(cc) = cg.get_first() {
        e.set_label(duel, i64::from(cc.get_code(duel)));
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let label = e.get_label(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_ADD_CODE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    e1.set_value(duel, label as i32);
    e1.set_code_condition(duel, |_, _, sumtype, _| sumtype & MATERIAL_FUSION != 0);
    c.register_effect(duel, e1);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_POLYMERIZATION) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn actop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 0, 1);
    e1.set_condition(duel, |duel, _| duel.is_battle_phase());
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE + PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}
