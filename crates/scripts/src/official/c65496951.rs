// Vaalmonica Disarmonia
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, PZONE, REMOVED};

const ID: u32 = 65496951;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_RESONANCE];
const AGATHOKAKOLOGICAL_VOICE: u32 = 39210885;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand with Angello and Dimonno in the Pendulum Zone to place a counter on, and Vaalmonica cards banished and in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (3048768, PZONE),
    (30432463, PZONE),
    (5605529, REMOVED),
    (42193638, GRAVE),
    (78598237, REMOVED),
    (78598237, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_COUNTER + CATEGORY_RECOVER + CATEGORY_DAMAGE + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn can_add_counter(c: CardId, duel: &Duel) -> bool {
    c.is_can_add_counter(duel, COUNTER_RESONANCE, 1)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(can_add_counter, tp, LOCATION_PZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_COUNTER, None, 1, 0, i32::from(COUNTER_RESONANCE));
    duel.set_possible_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 500);
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, tp, 500);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_GRAVE | LOCATION_REMOVED) as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VAALMONICA]) && !c.is_code(duel, ID) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = if a.extra != 0 {
        Some(usize::from(a.extra))
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_COUNTER);
        let g = duel.select_matching_card(tp, can_add_counter, tp, LOCATION_PZONE, 0, 1, 1, None)?;
        let Some(tc) = g.get_first() else { return Ok(()) };
        if !tc.add_counter(duel, COUNTER_RESONANCE, 1) {
            return Ok(());
        }
        duel.raise_event(tc, EVENT_CUSTOM + AGATHOKAKOLOGICAL_VOICE, Some(e), 0, tp, tp, 1)?;
        let op = duel.select_effect(tp, &[(true, aux::stringid(ID, 1)), (true, aux::stringid(ID, 2))])?;
        duel.break_effect()?;
        op
    };
    let (location, description, gain) = match op {
        Some(1) => (LOCATION_REMOVED, aux::stringid(ID, 3), true),
        Some(2) => (LOCATION_GRAVE, aux::stringid(ID, 4), false),
        _ => return Ok(()),
    };
    let g = duel.get_matching_group(thfilter, tp, location, 0, None);
    let done = if gain {
        duel.recover(tp, 500, REASON_EFFECT)?
    } else {
        duel.damage(tp, 500, REASON_EFFECT)?
    };
    if done > 0 && !g.is_empty() && duel.select_yes_no(tp, description)? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.break_effect()?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}
