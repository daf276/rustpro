// Fateful Adventure
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 39568067;

pub const LISTED_NAMES: &[u32] = &[TOKEN_ADVENTURER];
// In the hand to activate, with Wandering Gryphon Rider (listing the Adventurer Token) in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (2563463, DECK), (2563463, DECK), (2563463, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_count_limit(duel, 1, 0);
    e2.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE == REASON_BATTLE));
    e2.set_target_card(duel, |duel, _, c| c.get_equip_count(duel) > 0);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, mthtg);
    e3.set_operation(duel, mthop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_EQUIP);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_target(duel, sthtg);
    e4.set_operation(duel, sthop);
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EVENT_SUMMON_SUCCESS);
    c.register_effect(duel, e5);
}

fn mthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.lists_code(duel, TOKEN_ADVENTURER) && c.is_able_to_hand(duel)
}

fn mthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(mthfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 1, tp, 1);
    Ok(false)
}

fn mthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, mthfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, |_, _| true, 1, 1, REASON_EFFECT)?;
    }
    Ok(())
}

fn adventurerfilter(c: CardId, duel: &Duel, ec: CardId) -> bool {
    c.is_faceup(duel) && c.is_code(duel, TOKEN_ADVENTURER) && ec.check_equip_target(duel, c)
}

fn eqfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.check_unique_on_field(duel, tp)
        && !c.is_forbidden(duel)
        && duel.get_location_count(tp, LOCATION_SZONE) > 0
        && duel.is_existing_target(|tc, duel| adventurerfilter(tc, duel, c), tp, LOCATION_MZONE, 0, 1, None)
}

fn sthfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_equip_spell(duel) && c.lists_code(duel, TOKEN_ADVENTURER) && (c.is_able_to_hand(duel) || eqfilter(c, duel, tp))
}

fn sthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0 && duel.is_existing_matching_card(|c, duel| sthfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None))
}

fn sthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let Some(eqc) = duel
        .select_matching_card(tp, |c, duel| sthfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    aux::to_hand_or_else(
        duel,
        eqc,
        tp,
        |eqc, duel| eqfilter(eqc, duel, tp),
        |duel| {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
            let g = duel.select_matching_card(tp, |c, duel| adventurerfilter(c, duel, eqc), tp, LOCATION_MZONE, 0, 1, 1, None)?;
            duel.hint_selection_full(g, true);
            if let Some(tc) = g.get_first() {
                duel.equip(tp, eqc, tc)?;
            }
            Ok(())
        },
        aux::stringid(ID, 2),
    )
}
