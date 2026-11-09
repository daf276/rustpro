// DoomZ V Five - Amalthe
use super::c54265980::add_xyz_effect;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 28877382;

pub const LISTED_NAMES: &[u32] = &[ID];
// On the field, with Axe of Despair in the hand to equip, DoomZ monsters in the Deck to search, Dark Hole to destroy it, and
// DoomZ Break - Diactorus in the Extra Deck to attach it to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (40619825, HAND),
    (40619825, HAND),
    (53129443, HAND),
    (54265980, DECK),
    (54265980, DECK),
    (92472273, DECK),
    (31010081, EXTRA),
    (31010081, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_target(duel, thtg);
    e1a.set_operation(duel, thop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
    let e1c = e1a.clone_effect(duel);
    e1c.set_code(duel, EVENT_DESTROYED);
    e1c.set_condition(duel, |_, a| a.r & REASON_EFFECT > 0);
    c.register_effect(duel, e1c);
    add_xyz_effect(duel, c, ID);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DOOMZ]) && c.is_monster(duel) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
