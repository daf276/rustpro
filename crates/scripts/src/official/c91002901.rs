// Duel Evolution - Assault Zone
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 91002901;

pub const LISTED_SERIES: &[u16] = &[SET_ASSAULT_MODE];
pub const LISTED_NAMES: &[u32] = &[CARD_ASSAULT_MODE];
// Two copies in the hand, with monsters that mention Assault Mode Activate (Assault Sonic Warrior, Crimson Blader/Assault Mode) in the
// Deck to add. Releasing a Synchro Monster in the Extra Deck for Assault Mode Activate is unported in the engine, so none is placed.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (18711696, DECK), (18711696, DECK), (50604072, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_FIELD);
    e2a.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2a.set_range(duel, LOCATION_FZONE);
    e2a.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2a.set_target_card(duel, |duel, _, c| {
        c.is_set_card(duel, &[SET_ASSAULT_MODE]) && c.is_status(duel, STATUS_SPSUMMON_TURN)
    });
    e2a.set_value(duel, 1);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2b.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_cost(duel, Cost::pay_lp::<2000>);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_operation(duel, effop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.lists_code(duel, CARD_ASSAULT_MODE) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::necro_valley_filter(thfilter), tp, LOCATION_DECK | LOCATION_GRAVE, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 3)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_EXTRA_RELEASE_NONSUM);
    e1.set_target_range(duel, LOCATION_EXTRA, 0);
    e1.set_condition(duel, |duel, a| !duel.has_flag_effect(a.e.get_handler_player(duel), ID));
    e1.set_value(duel, Value::Reason(|duel, _, re, r, _| relval(duel, re, r)));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_RELEASE);
    e2.set_condition(duel, |duel, a| {
        relval(duel, a.re, a.r)
            && a.eg
                .unwrap_or_default()
                .is_exists(duel, |c, duel| c.is_previous_location(duel, LOCATION_EXTRA), 1, None)
    });
    e2.set_operation(duel, |duel, a| {
        duel.register_flag_effect(a.tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        Ok(())
    });
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    Ok(())
}

fn relval(duel: &Duel, re: Option<EffectId>, r: u32) -> bool {
    re.is_some_and(|re| re.is_activated(duel) && r & REASON_COST > 0 && re.get_handler(duel).is_code(duel, CARD_ASSAULT_MODE))
}
