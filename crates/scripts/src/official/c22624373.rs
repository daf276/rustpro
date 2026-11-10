// Lyla, Lightsworn Sorceress
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT, SZONE};

const ID: u32 = 22624373;

// Two on the field to switch to Defense Position against the opponent's Spells and Traps, and cards in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (12580477, DECK),
    (12580477, DECK),
    (44095762, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_position(duel, POS_FACEUP_ATTACK));
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_category(duel, CATEGORY_DECKDES);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_code(duel, EVENT_PHASE | PHASE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| a.tp == duel.get_turn_player());
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_spell_trap(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let tc = duel.get_first_target();
    if c.is_relate_to_effect(duel, a.e) && c.is_position(duel, POS_FACEUP_ATTACK) {
        duel.change_position(c, POS_FACEUP_DEFENSE)?;
        if let Some(tc) = tc
            && tc.is_relate_to_effect(duel, a.e)
        {
            duel.destroy(tc, REASON_EFFECT)?;
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_COPY_INHERIT);
        e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END | RESET_SELF_TURN, 2);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, a.tp, 3);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.discard_deck(a.tp, 3, REASON_EFFECT)?;
    Ok(())
}
