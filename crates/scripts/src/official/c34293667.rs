// Ice Barrier
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 34293667;

// Set copies and one in the Graveyard, Level 5 or higher WATER monsters in the Deck, and the opponent's Elfnote Lucina to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, GRAVE),
    (4929256, DECK),
    (4929256, DECK),
    (78060096, DECK),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let (_, bc) = duel.get_battle_monster(tp);
    if chk == 0 {
        return Ok(bc.is_some_and(|bc| {
            bc.is_faceup(duel)
                && (bc.is_attack_above(duel, 1)
                    || bc.is_negatable_monster(duel)
                    || bc.is_has_effect(duel, EFFECT_CANNOT_CHANGE_POSITION).is_none())
        }));
    }
    let Some(bc) = bc else { return Ok(false) };
    duel.set_target_card(bc);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(bc.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(bc.into()), 1, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(bc) = duel.get_first_target() else { return Ok(()) };
    if !(bc.is_relate_to_effect(duel, a.e) && bc.is_faceup(duel) && bc.is_relate_to_battle(duel) && bc.is_controler(duel, 1 - a.tp)) {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    bc.negate_effects(duel, c, 0, false);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3313);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e1.set_value(duel, 0);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    bc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
    bc.register_effect(duel, e2);
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 5) && c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_able_to_hand(duel)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = if duel.is_turn_player(tp) { 2 } else { 1 };
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_attribute(duel, ATTRIBUTE_WATER));
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END | RESET_SELF_TURN, ct);
    duel.register_effect(e1, tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let Some(tgc) = duel
        .select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    if duel.send_to_grave(tgc, REASON_EFFECT)? == 0 || !tgc.is_location(duel, LOCATION_GRAVE) {
        return Ok(());
    }
    let thg = duel.get_matching_group(aux::necro_valley_filter(thfilter), tp, LOCATION_GRAVE, 0, None);
    if !thg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = thg.select(duel, tp, 1, 1, None)?;
        if !sg.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, sg)?;
        }
    }
    Ok(())
}
