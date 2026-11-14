// Radiant Typhoon Vision
use crate::prelude::*;
use crate::testing::{Check, DECK, SZONE};

const ID: u32 = 20508881;

pub const LISTED_SERIES: &[u16] = &[SET_RADIANT_TYPHOON];
pub const LISTED_NAMES: &[u32] = &[CARD_MYSTICAL_SPACE_TYPHOON];
// Mystical Space Typhoon in the Deck to search, and a set one to destroy this card with.
pub const SUPPORT: &[(u32, u16)] = &[(CARD_MYSTICAL_SPACE_TYPHOON, DECK), (CARD_MYSTICAL_SPACE_TYPHOON, SZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    add_set_effect(duel, c, aux::stringid(ID, 1));
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_MYSTICAL_SPACE_TYPHOON) && c.is_able_to_hand(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = !duel.has_flag_effect(tp, ID) && duel.is_player_can_draw(tp, 2);
    let b2 = !duel.has_flag_effect(tp, ID + 1) && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_DRAW + CATEGORY_HANDES);
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 1, tp, 2);
        duel.set_operation_info(0, CATEGORY_HANDES, None, 1, tp, 1);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    }
    Ok(false)
}

fn discardfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_RADIANT_TYPHOON]) || c.is_quick_play_spell(duel)) && c.is_discardable_full(duel, REASON_EFFECT, None)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    match a.e.get_label(duel) {
        1 => {
            if duel.draw(tp, 2, REASON_EFFECT)? > 0 {
                duel.shuffle_hand(tp)?;
                duel.break_effect()?;
                let handg = duel.get_field_group(tp, LOCATION_HAND, 0);
                if handg.is_exists(duel, discardfilter, 1, None) {
                    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
                    let dg = handg.filter_select(duel, tp, discardfilter, 1, 1, None)?;
                    if !dg.is_empty() {
                        duel.send_to_grave(dg, REASON_EFFECT | REASON_DISCARD)?;
                    }
                } else {
                    duel.send_to_grave(handg, REASON_EFFECT | REASON_DISCARD)?;
                }
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let filter = aux::necro_valley_filter(thfilter);
            let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
        _ => {}
    }
    Ok(())
}

// The "if destroyed by Mystical Space Typhoon, Set this card" effect of the Radiant Typhoon Spells and Traps.
pub fn add_set_effect(duel: &mut Duel, c: CardId, description: u64) {
    let e2 = duel.create_effect(c);
    e2.set_description(duel, description);
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.get_handler(duel).is_code(duel, CARD_MYSTICAL_SPACE_TYPHOON))
    });
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_ssetable(duel));
    }
    if c.is_location(duel, LOCATION_GRAVE) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, a.tp, 0);
    }
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.sset(a.tp, c)?;
    }
    Ok(())
}
