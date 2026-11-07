// Triple Tactics Thrust
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 35269904;

pub const LISTED_NAMES: &[u32] = &[ID];

// Raigeki, Trap Hole and Mirror Force to take from the Deck, and Elfnote Lucina for the opponent to activate.
pub const SUPPORT: &[(u32, u16)] = &[(12580477, DECK), (4206964, DECK), (44095762, DECK), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, |duel, a| duel.get_custom_activity_count(ID, 1 - a.tp, ACTIVITY_CHAIN) > 0);
    e1.set_target(duel, settg);
    e1.set_operation(duel, setop);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_CHAIN,
        CounterFilter::Chain(|duel, re, _, _| !re.is_monster_effect(duel)),
    );
}

fn stfilter(c: CardId, duel: &Duel, set_chk: bool, tohand_chk: bool) -> bool {
    c.is_normal_spell_trap(duel) && !c.is_code(duel, ID) && ((tohand_chk && c.is_able_to_hand(duel)) || (set_chk && c.is_ssetable(duel)))
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let mut ft = duel.get_location_count(tp, LOCATION_SZONE);
    if a.e.get_handler(duel).is_location(duel, LOCATION_HAND) {
        ft -= 1;
    }
    let set_chk = ft > 0;
    let tohand_chk = duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| stfilter(c, duel, set_chk, tohand_chk), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let set_chk = duel.get_location_count(tp, LOCATION_SZONE) > 0;
    let tohand_chk = duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0;
    if !set_chk && !tohand_chk {
        return Ok(());
    }
    let desc = match (set_chk, tohand_chk) {
        (false, _) => HINTMSG_ATOHAND,
        (_, false) => HINTMSG_SET,
        _ => aux::stringid(ID, 1),
    };
    duel.hint(HINT_SELECTMSG, tp, desc);
    let g = duel.select_matching_card(
        tp,
        |c, duel| stfilter(c, duel, set_chk, tohand_chk),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if tohand_chk && sc.is_able_to_hand(duel) && (!set_chk || !sc.is_ssetable(duel) || duel.select_yes_no(tp, aux::stringid(ID, 2))?) {
        if duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? == 0 {
            return Ok(());
        }
        duel.confirm_cards(1 - tp, sc)?;
    } else if duel.sset(tp, sc)? > 0 {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
        e1.set_reset(duel, RESET_EVENT | RESETS_CANNOT_ACT | RESET_PHASE | PHASE_END);
        sc.register_effect(duel, e1);
    }
    Ok(())
}
