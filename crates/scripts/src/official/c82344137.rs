// Phara the Primordial Goddess
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 82344137;

// In the hand with Raigeki to reveal, on the field with the opponent's Raigeki to be destroyed by, Monster Reborn to bring it back
// from the Graveyard, and a Sangan of the opponent to control.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (12580477, HAND),
    (12580477, HAND | OPPONENT),
    (83764718, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
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
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, effcost);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_CONTROL);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_summon_location(duel, LOCATION_GRAVE));
    e3.set_target(duel, ctrltg);
    e3.set_operation(duel, ctrlop);
    c.register_effect(duel, e3);
}

fn hidden_spell_trap(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && !c.is_public(duel)
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_public(duel) && duel.is_existing_matching_card(hidden_spell_trap, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let Some(rc) = duel
        .select_matching_card(tp, hidden_spell_trap, tp, LOCATION_HAND, 0, 1, 1, Some(c.into()))?
        .get_first()
    else {
        return Ok(false);
    };
    duel.confirm_cards(1 - tp, CardSet::from_iter([c, rc]))?;
    duel.shuffle_hand(tp)?;
    let code = rc.get_original_code_rule(duel);
    a.e.set_label(duel, code.into());
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    aux::register_client_hint_full(duel, c, 0, tp, 1, 1, Some(aux::stringid(ID, 3)), 0, 2);
    let label = a.e.get_label(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_label(duel, label);
    e1.set_operation(duel, |duel, a| {
        let chain = a.ev as u8;
        let code1: u32 = duel.get_chain_info(chain, CHAININFO_TRIGGERING_CODE);
        let code2: u32 = duel.get_chain_info(chain, CHAININFO_TRIGGERING_CODE2);
        let revealed = a.e.get_label(duel);
        if i64::from(code1) == revealed || i64::from(code2) == revealed {
            duel.set_chain_limit(|_, _, _, _, _| false, 0);
        }
        Ok(())
    });
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
    duel.register_effect(e1, tp);
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_HAND | LOCATION_ONFIELD)
        && c.is_reason(duel, REASON_EFFECT)
        && a.re.is_some_and(|re| re.is_spell_trap_effect(duel))
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
    Ok(())
}

fn highest_attack(duel: &Duel, tp: u8) -> Option<CardSet> {
    let g = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    g.get_max_group(duel, |c, duel| c.get_attack(duel).into()).map(|(tg, _)| tg)
}

fn ctrltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = highest_attack(duel, tp);
    if chk == 0 {
        return Ok(g.is_some_and(|g| g.is_exists(duel, CardId::is_controler_can_be_changed, 1, None)));
    }
    duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, tp, 0);
    Ok(false)
}

fn ctrlop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(mut g) = highest_attack(duel, tp) else { return Ok(()) };
    g.keep_matching(duel, CardId::is_controler_can_be_changed, None);
    if g.len() >= 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
        g = g.select(duel, tp, 1, 1, None)?;
    }
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.get_control(g, tp)?;
    }
    Ok(())
}
