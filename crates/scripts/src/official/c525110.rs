// Cherry Inmato
use crate::prelude::*;
use crate::testing::{DECK, MZONE, OPPONENT};

const ID: u32 = 525110;

// On the field with Inmato in the Deck to summon, and Elfnote Lucina on the opponent's side to destroy it in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (39703254, DECK),
    (39703254, DECK),
    (39703254, DECK),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE) && c.is_reason(duel, REASON_BATTLE)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_INMATO]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE).min(2);
    if ft <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, ft as u8, None)?;
    let mut cards = g.into_iter();
    let Some(t1) = cards.next() else { return Ok(()) };
    duel.special_summon_step(t1, 0, tp, tp, false, false, POS_FACEUP)?;
    // The code adds a flag to the effect code, so the effect never matches anything.
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, 3310);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL + EFFECT_FLAG_CLIENT_HINT);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    t1.register_effect(duel, e1);
    if let Some(t2) = cards.next() {
        duel.special_summon_step(t2, 0, tp, tp, false, false, POS_FACEUP)?;
        let e2 = e1.clone_effect(duel);
        t2.register_effect(duel, e2);
    }
    duel.special_summon_complete()?;
    Ok(())
}
