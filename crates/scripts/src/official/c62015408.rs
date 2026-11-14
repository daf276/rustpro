// Ghost Reaper & Winter Cherries
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 62015408;

// Ancient Fairy Dragon in both Extra Decks to banish, and the opponent's Sangans to have more monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (25862681, EXTRA),
    (25862681, EXTRA | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, |duel, a| {
        duel.get_field_group_count(a.tp, LOCATION_MZONE, 0) < duel.get_field_group_count(a.tp, 0, LOCATION_MZONE)
            && duel.is_existing_matching_card(aux::TRUE, a.tp, LOCATION_EXTRA, 0, 1, None)
    });
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, 30459350).is_none()
            && duel.get_field_group_count(tp, LOCATION_EXTRA, 0) > 0
            && duel.get_field_group_count(tp, 0, LOCATION_EXTRA) > 0);
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let sg = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(sc) = sg.get_first() else { return Ok(()) };
    duel.confirm_cards(1 - tp, sg)?;
    duel.break_effect()?;
    let g = duel.get_field_group(tp, 0, LOCATION_EXTRA);
    duel.confirm_cards(tp, g)?;
    let code = sc.get_code(duel);
    let tg = g.filter(duel, |c, duel| c.is_code(duel, code), None);
    if !tg.is_empty() {
        duel.remove(tg, POS_FACEUP, REASON_EFFECT)?;
    }
    duel.shuffle_extra(1 - tp)?;
    Ok(())
}
