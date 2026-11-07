// Masked HERO Dark Law
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 58481572;

// On the field, in the Extra Deck for Mask Change to Special Summon, and Pot of Greed in the hand of the opponent, who draws
// outside the Draw Phase.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (21143940, HAND),
    (9411399, MZONE),
    (55144522, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_special_summoned(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(
        duel,
        EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_RANGE + EFFECT_FLAG_IGNORE_IMMUNE,
    );
    e1.set_code(duel, EFFECT_TO_GRAVE_REDIRECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_ALL, LOCATION_ALL);
    e1.set_target_card(duel, |duel, e, c| {
        let tp = e.get_handler_player(duel);
        !c.is_owner(duel, tp) && duel.is_player_can_remove(tp, Some(c))
    });
    e1.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_HAND);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, handrmcon);
    e2.set_target(duel, handrmtg);
    e2.set_operation(duel, handrmop);
    c.register_effect(duel, e2);
}

fn handrmconfilter(c: CardId, duel: &Duel, opp: u8) -> bool {
    c.is_previous_controler(duel, opp) && c.is_previous_location(duel, LOCATION_DECK) && c.is_controler(duel, opp)
}

fn handrmcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    !duel.is_draw_phase_full(None)
        && a.eg
            .is_some_and(|eg| eg.is_exists(duel, |c, duel| handrmconfilter(c, duel, 1 - tp), 1, None))
}

fn handrmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, LOCATION_HAND, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn handrmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel
        .get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_HAND, None)
        .random_select(duel, tp, 1);
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
