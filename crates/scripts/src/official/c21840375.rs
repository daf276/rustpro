// Hidden Spellbook
use crate::prelude::*;
use crate::testing::{Check, GRAVE, SZONE};

const ID: u32 = 21840375;

// Set on the field with Spells in the Graveyard to return to the Deck: Raigeki and Pot of Greed.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (12580477, GRAVE), (12580477, GRAVE), (55144522, GRAVE)];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_able_to_deck(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 2, 2, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 2, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let sg = tg.filter(duel, |c, duel| c.is_relate_to_effect(duel, a.e), None);
    duel.send_to_deck(sg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    Ok(())
}
