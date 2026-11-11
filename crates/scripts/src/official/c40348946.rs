// Hyper Synchron
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 40348946;

// On the field with Kojikocy (Level 4 Normal Monster) for Stardust Dragon in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (CARD_STARDUST_DRAGON, EXTRA),
    (CARD_STARDUST_DRAGON, EXTRA),
    (CARD_STARDUST_DRAGON, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BE_MATERIAL);
    e1.set_condition(duel, con);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
}

fn con(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE)
        && a.r == REASON_SYNCHRO
        && c.get_reason_card(duel).is_some_and(|sync| sync.is_race(duel, RACE_DRAGON))
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(sync) = c.get_reason_card(duel) else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 800);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    sync.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_operation(duel, rmop);
    e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
    sync.register_effect(duel, e2);
    Ok(())
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.remove(a.e.get_handler(duel), POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
