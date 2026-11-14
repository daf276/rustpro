// Floowandereeze & Empen
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 80611581;

pub const LISTED_SERIES: &[u16] = &[SET_FLOOWANDEREEZE];

// In the hand with Sangans to tribute, Floowandereeze and the Magnificent Map in the Deck to add, and on the field with cards in the
// hand to banish, against the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (28126717, DECK),
    (28126717, DECK),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_condition(duel, trsumcon);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, 1);
    e2.set_condition(duel, trsumcon);
    e2.set_value(duel, Value::EffectPlayer(actlmtval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, adcon);
    e3.set_cost(duel, adcost);
    e3.set_operation(duel, adop);
    c.register_effect(duel, e3);
}

fn trsumcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_tribute_summoned(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FLOOWANDEREEZE]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn sumfilter(c: CardId, duel: &Duel) -> bool {
    c.is_summonable(duel, true, None)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    duel.confirm_cards(1 - tp, g)?;
    if !tc.is_location(duel, LOCATION_HAND) {
        return Ok(());
    }
    let sg = duel.get_matching_group(sumfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, None);
    if !sg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.break_effect()?;
        duel.shuffle_hand(tp)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
        if let Some(sc) = sg.select(duel, tp, 1, 1, None)?.get_first() {
            duel.summon(tp, sc, true, None)?;
        }
    }
    Ok(())
}

fn actlmtval(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    rc.is_special_summoned(duel) && rc.is_attack_pos(duel) && rc.is_location(duel, LOCATION_MZONE) && re.is_monster_effect(duel)
}

fn adcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_battle_target(duel).is_some()
}

fn adcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(CardId::is_able_to_remove_as_cost, tp, LOCATION_HAND, 0, 1, None)
                && c.get_flag_effect(duel, ID) == 0,
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_remove_as_cost, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    c.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    Ok(false)
}

fn adop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(bc) = c.get_battle_target(duel) else { return Ok(()) };
    if bc.is_relate_to_battle(duel) && bc.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, (bc.get_attack(duel) + 1) / 2);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        bc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_SET_DEFENSE_FINAL);
        e2.set_value(duel, (bc.get_defense(duel) + 1) / 2);
        bc.register_effect(duel, e2);
    }
    Ok(())
}
