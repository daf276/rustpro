// Apprentice Illusion Magician
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 30603688;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN];

// In the hand with Dark Magician in the Deck to search and on the field to boost, and a Sangan of the opponent's to fight.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_DARK_MAGICIAN, DECK),
    (CARD_DARK_MAGICIAN, DECK),
    (CARD_DARK_MAGICIAN, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_ATKCHANGE);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e4.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e4.set_condition_mut(duel, atkcon);
    e4.set_cost(duel, Cost::self_to_grave);
    e4.set_operation(duel, atkop);
    c.register_effect(duel, e4);
}

fn discard_group(duel: &Duel, e: EffectId, tp: u8) -> CardSet {
    duel.get_matching_group(CardId::is_discardable, tp, LOCATION_HAND, 0, Some(e.get_handler(duel).into()))
}

fn chkf_mmz(duel: &Duel, tp: u8, sg: CardSet) -> (bool, bool) {
    (duel.get_mzone_count(tp, Some(sg)) >= 1, false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let (e, tp) = (a.e, c.get_controler(duel));
    aux::select_unselect_group_check(duel, discard_group(duel, e, tp), 1, 1, &|duel, sg, _, _| chkf_mmz(duel, tp, sg))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = discard_group(duel, e, tp);
    let g = aux::select_unselect_group_full(
        duel,
        rg,
        1,
        1,
        &|duel, sg, _, _| chkf_mmz(duel, tp, sg),
        tp,
        HINTMSG_DISCARD,
        None,
        true,
    )?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.send_to_grave(g, REASON_DISCARD | REASON_COST)?;
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_DARK_MAGICIAN) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn atkcon(duel: &mut Duel, a: &Args) -> bool {
    let Some(mut c) = duel.get_attack_target() else { return false };
    if c.is_controler(duel, 1 - a.tp) {
        let Some(attacker) = duel.get_attacker() else { return false };
        c = attacker;
    }
    a.e.set_label_object_group(duel, Some(c.into()));
    c != a.e.get_handler(duel) && c.is_race(duel, RACE_SPELLCASTER) && c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_relate_to_battle(duel)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(c) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    if c.is_faceup(duel) && c.is_relate_to_battle(duel) {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE_CAL);
        e1.set_value(duel, 2000);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        c.register_effect(duel, e2);
    }
    Ok(())
}
