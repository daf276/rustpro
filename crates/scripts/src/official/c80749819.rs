// Call of the Forgotten
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 80749819;
pub const LISTED_NAMES: &[u32] = &[CARD_CALL_OF_THE_HAUNTED];

// Call of the Haunted in the Deck, Graveyard and hand to Set and activate, Psychic Kappa in the Graveyard for it to revive, and
// Sangan on the opponent's side to send to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (CARD_CALL_OF_THE_HAUNTED, DECK),
    (CARD_CALL_OF_THE_HAUNTED, DECK),
    (CARD_CALL_OF_THE_HAUNTED, GRAVE),
    (CARD_CALL_OF_THE_HAUNTED, HAND),
    (CARD_CALL_OF_THE_HAUNTED, SZONE),
    (7892180, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, gycon);
    e2.set_target(duel, gytg);
    e2.set_operation(duel, gyop);
    c.register_effect(duel, e2);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_CALL_OF_THE_HAUNTED) && c.is_ssetable(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let g = duel.get_matching_group(aux::necro_valley_filter(setfilter), tp, LOCATION_DECK | LOCATION_GRAVE, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        let ft = duel.get_location_count(tp, LOCATION_SZONE).min(3);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let sg = g.select(duel, tp, 1, ft as u8, None)?;
        if !sg.is_empty() && duel.sset(tp, sg)? > 0 {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, aux::stringid(ID, 3));
            e1.set_type(duel, EFFECT_TYPE_FIELD);
            e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
            e1.set_target_range(duel, 1, 0);
            e1.set_target_card(duel, |duel, _, c| {
                c.is_race_except(duel, RACE_ZOMBIE) && !c.is_location(duel, LOCATION_GRAVE)
            });
            e1.set_reset(duel, RESET_PHASE | PHASE_END | RESET_OPPO_TURN);
            duel.register_effect(e1, tp);
        }
    }
    Ok(())
}

fn gycon(duel: &Duel, a: &Args) -> bool {
    a.rp == a.tp
        && a.re
            .is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.get_handler(duel).is_code(duel, CARD_CALL_OF_THE_HAUNTED))
}

fn gytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_grave(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_grave, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, CardId::is_able_to_grave, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, 1, tp, 0);
    Ok(false)
}

fn gyop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_grave(tc, REASON_EFFECT)?;
    }
    Ok(())
}
