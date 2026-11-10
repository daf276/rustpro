// Tearlaments Merrli
use super::c572850::Params;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND};

const ID: u32 = 74078255;

// In the hand to Normal Summon, with copies in the Deck for the mill to send, to Fusion Summon Rulkallos with Kitkallos in the
// Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, DECK), (ID, DECK), (84330567, EXTRA), (92731385, GRAVE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DECKDES);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| {
        duel.get_current_phase() != PHASE_DAMAGE && a.e.get_handler(duel).is_reason(duel, REASON_EFFECT)
    });
    e3.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e3.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e3);
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, 3));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, tp, 3);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.discard_deck(a.tp, 3, REASON_EFFECT)?;
    Ok(())
}
