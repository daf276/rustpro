// Dark Tyranno
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 46826483;

// Copies in the hand with Pot of Greed and Dark Hole (a Spell, then a destroyer), Level 8 Dinosaurs in the Deck, and a target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (55144522, HAND),
    (53129443, HAND),
    (93332803, DECK),
    (93332803, DECK),
    (18940556, DECK),
    (99171160, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    e1.set_condition(duel, directattackcond);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, |duel, _| {
        duel.get_custom_activity_count(ID, 0, ACTIVITY_CHAIN) > 0 || duel.get_custom_activity_count(ID, 1, ACTIVITY_CHAIN) > 0
    });
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(|duel, re, _, _| !re.is_spell_effect(duel)));
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn directattackcond(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_field_group(a.e.get_handler_player(duel), 0, LOCATION_MZONE);
    g.len() == g.filter_count(duel, CardId::is_defense_pos, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 8) && c.is_race(duel, RACE_DINOSAUR) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, sc)?;
        duel.shuffle_hand(tp)?;
        if duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.break_effect()?;
            duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
