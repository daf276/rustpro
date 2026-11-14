// Ariane the Labrynth Servant
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, SZONE};

const ID: u32 = 75730490;

pub const LISTED_NAMES: &[u32] = &[ID];

// On the field with Waboku in the hand to send, Headless Knights in the Deck to summon, and Raigeki Break and Trap Hole Set to
// remove monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (12607053, HAND),
    (5434080, DECK),
    (5434080, DECK),
    (4178474, SZONE),
    (4178474, SZONE),
    (4206964, SZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, ID);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW + CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, drcon);
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
}

fn spcfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_normal_trap(duel)
        && c.is_able_to_grave_as_cost(duel)
        && (c.is_location(duel, LOCATION_HAND) || c.is_facedown(duel))
        && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let locations = LOCATION_HAND | LOCATION_ONFIELD;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spcfilter(c, duel, tp), tp, locations, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| spcfilter(c, duel, tp), tp, locations, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_FIEND)
        && c.is_level_below(duel, 4)
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_DECK,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP_DEFENSE)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_reason(duel, REASON_EFFECT)
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    a.rp == a.tp
        && a.re
            .is_some_and(|re| re.is_trap_effect(duel) && re.get_handler(duel).get_original_type(duel) == TYPE_TRAP)
        && a.eg.unwrap_or_default().is_exists(duel, cfilter, 1, None)
}

fn ffilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_FIEND) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn stfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.draw(p, d, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| ffilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None);
    let b2 = duel.is_existing_matching_card(stfilter, tp, LOCATION_HAND, 0, 1, None);
    if !((b1 || b2) && duel.select_yes_no(tp, aux::stringid(ID, 2))?) {
        return Ok(());
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])?;
    if op == Some(1) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| ffilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    } else if op == Some(2) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let g = duel.select_matching_card(tp, stfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.sset_full(tp, g, tp, false)?;
        }
    }
    Ok(())
}
