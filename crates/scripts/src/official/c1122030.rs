// Artmage Vandalism -Assault-
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 1122030;
const ACROPOLIS: u32 = 74733322;

pub const LISTED_NAMES: &[u32] = &[CARD_MEDIUS_THE_PURE, ACROPOLIS];

// In hand to activate and on the field for the other effects, with Medius the Pure in the Deck to add and on the field to target,
// Acropolis in a random zone, and Mystical Space Typhoon of the opponent to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (ID, SZONE),
    (CARD_MEDIUS_THE_PURE, DECK),
    (CARD_MEDIUS_THE_PURE, MZONE),
    (ACROPOLIS, 0),
    (5318639, HAND | OPPONENT),
    (5318639, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Replaced,
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, fusmattg);
    e2.set_operation(duel, fusmatop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target(duel, reptg);
    e3.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e3.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        duel.send_to_grave(c, REASON_EFFECT | REASON_REPLACE).map(drop)
    });
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_MEDIUS_THE_PURE) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}

fn fusmattg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn fusmatop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 3));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_ADD_SETCODE);
        e1.set_value(duel, SET_ARTMAGE as i32);
        e1.set_code_condition(duel, |_, _, sumtype, _| {
            sumtype & MATERIAL_FUSION > 0 || sumtype & SUMMON_TYPE_FUSION as u64 > 0
        });
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, ACROPOLIS)
        && c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_on_field(duel)
        && c.is_reason(duel, REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_grave(duel)
            && a.eg
                .unwrap_or_default()
                .is_exists(duel, |sc, duel| repfilter(sc, duel, tp), 1, None));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}
