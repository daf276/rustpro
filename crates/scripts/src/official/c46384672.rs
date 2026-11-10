// Armed Dragon LV5
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 46384672;
const ARMED_DRAGON_LV7: u32 = 73879377;

pub const LISTED_NAMES: &[u32] = &[ARMED_DRAGON_LV7];
// Weak monsters (Tomozaurus, Anthrosaurus) on the opponent's side to destroy in battle and by the effect, Armed Dragon LV7 in the
// Deck to level up to, and monsters in the hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ARMED_DRAGON_LV7, DECK),
    (ARMED_DRAGON_LV7, DECK),
    (46457856, MZONE | OPPONENT),
    (89904598, MZONE | OPPONENT),
    (46457856, HAND),
    (89904598, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_operation(duel, bdop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, descost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) > 0);
    e3.set_cost(duel, Cost::self_to_grave);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn bdop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let atk = c.get_attack(duel).max(0);
    c.is_monster(duel)
        && c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_target(|c, duel| dfilter(c, duel, atk), tp, 0, LOCATION_MZONE, 1, None)
}

fn dfilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_faceup(duel) && c.get_attack(duel) <= atk
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| cfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| cfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    let atk = g.get_first().map_or(0, |c| c.get_attack(duel).max(0));
    e.set_label(duel, i64::from(atk));
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let atk = e.get_label(duel) as i32;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && dfilter(chkc, duel, atk));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |c, duel| dfilter(c, duel, atk), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let atk = a.e.get_label(duel) as i32;
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, a.e)
        && tc.get_attack(duel) <= atk
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, ARMED_DRAGON_LV7) && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if e.get_handler(duel).get_sequence(duel) < 5 {
        ft += 1;
    }
    if chk == 0 {
        return Ok(
            ft > 0 && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if let Some(tc) = g.get_first()
        && duel.special_summon(tc, 0, tp, tp, true, false, POS_FACEUP)? > 0
    {
        tc.complete_procedure(duel);
    }
    Ok(())
}
