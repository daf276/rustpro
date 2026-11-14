// Crystal Beast Sapphire Pegasus
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, OPPONENT};

const ID: u32 = 7093411;

// Copies in the hand to summon, Crystal Beasts in the Deck to place; the opponent's Raigeki to place it as a Spell.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (68215963, DECK), (21698716, DECK), (12580477, OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c68215963::add_redirect(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    for code in [EVENT_SPSUMMON_SUCCESS, EVENT_FLIP_SUMMON_SUCCESS] {
        let e = e2.clone_effect(duel);
        e.set_code(duel, code);
        c.register_effect(duel, e);
    }
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CRYSTAL_BEAST]) && !c.is_forbidden(duel)
}

const LOCATIONS: u32 = LOCATION_DECK | LOCATION_GRAVE | LOCATION_HAND;

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATIONS, 0, 1, None) && duel.get_location_count(a.tp, LOCATION_SZONE) > 0);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, aux::necro_valley_filter(filter), tp, LOCATIONS, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
        super::c68215963::become_continuous_spell(duel, a.e.get_handler(duel), tc);
        duel.raise_event(tc, EVENT_CUSTOM + CARD_CRYSTAL_TREE, Some(a.e), 0, tp, 0, 0)?;
    }
    Ok(())
}
