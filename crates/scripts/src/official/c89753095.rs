// Lightning Crash
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 89753095;

// Copies in the hand, with Thunder monsters (Wattkid) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (27324313, DECK), (27324313, DECK), (27324313, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_THUNDER) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.first()
        && duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_HAND)
    {
        duel.confirm_cards(1 - tp, sc)?;
        if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
            return Ok(());
        }
        let code = sc.get_code(duel);
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 1));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_SUMMON);
        e1.set_target_range(duel, 1, 0);
        e1.set_label(duel, code.into());
        e1.set_target_card(duel, |duel, e, c| c.is_code(duel, e.get_label(duel) as u32));
        e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
        duel.register_effect(e1, tp);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_CANNOT_MSET);
        duel.register_effect(e2, tp);
    }
    Ok(())
}
