// Machina Gearframe
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 42940404;

struct Procedure;

impl UnionProcedure for Procedure {
    fn f(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_MACHINE)
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand and on the field with a Machine monster to equip (Mechanicalchaser), and Machina Ruinforce in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (7359741, MZONE),
    (46033517, DECK),
    (46033517, DECK),
    (46033517, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 2)), Check::ResolvesEffect(1068)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Union::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, stg);
    e1.set_operation(duel, sop);
    c.register_effect(duel, e1);
}

fn sfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MACHINA]) && c.get_code(duel) != ID && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn stg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(sfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn sop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, sfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
