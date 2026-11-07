// Fairy Tail - Luna
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 86937530;

// Copies in the Deck to search, and an opponent's monster to return.
pub const SUPPORT: &[(u32, u16)] = &[(ID, DECK), (ID, DECK), (ID, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.get_attack(duel) == 1850 && c.is_race(duel, RACE_SPELLCASTER) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let mut g = duel
        .select_target(tp, thfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?
        .unwrap_or_default();
    g.merge(a.e.get_handler(duel).into());
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, Some(g), 2, 0, LOCATION_MZONE as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, 1 - tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    Ok(false)
}

fn cfilter(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_code(duel, code) && c.is_able_to_grave_as_cost(duel)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let tc = duel.get_first_target();
    // The script indexes a nil target, a Lua error.
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if duel.is_chain_disablable(0) {
        let code = tc.get_code(duel);
        let g = duel.get_matching_group(|c, duel| cfilter(c, duel, code), tp, 0, LOCATION_DECK | LOCATION_EXTRA, None);
        if !g.is_empty() && duel.select_yes_no(1 - tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_TOGRAVE);
            let sg = g.select(duel, 1 - tp, 1, 1, None)?;
            duel.send_to_grave(sg, REASON_EFFECT)?;
            duel.negate_effect(0);
            return Ok(());
        }
    }
    if tc.is_relate_to_effect(duel, e) {
        duel.send_to_hand(CardSet::from_iter([c, tc]), PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
