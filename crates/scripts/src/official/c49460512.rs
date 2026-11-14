// Blackwing - Kogarashi the Wanderer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 49460512;

pub const LISTED_SERIES: &[u16] = &[SET_BLACKWING];
// In hand and on the field with Gale and Dark Hole to destroy Blackwings, and with Oroshi (Level 1 Tuner) beside it on the
// field for Blackwing Armor Master (Level 7) in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (2009101, MZONE),
    (73652465, MZONE),
    (53129443, HAND),
    (69031175, EXTRA),
    (69031175, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::MaterialFrom(MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_property(duel, EFFECT_FLAG_EVENT_PLAYER);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_condition(duel, limitcon);
    e2.set_operation(duel, limitop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_BLACKWING])
        && c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.get_reason(duel) & (REASON_DESTROY | REASON_EFFECT) == REASON_DESTROY | REASON_EFFECT
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None))
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}

fn limitcon(_: &Duel, a: &Args) -> bool {
    a.r == REASON_SYNCHRO
}

fn limitop(duel: &mut Duel, _: &Args) -> Script<()> {
    duel.set_chain_limit_till_chain_end(|_, _, rp, tp, _| tp == rp, 0);
    Ok(())
}
