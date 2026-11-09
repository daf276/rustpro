// Frontier Wiseman
use engine::constants::MSG_CHAIN_DISABLED;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 38742075;

// With Axe of Despair in hand to equip Swordsman of Landstar, a Warrior, and Mystical Space Typhoon to chain to it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (40619825, HAND),
    (40619825, HAND),
    (3573512, MZONE),
    (3573512, MZONE),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_CHAIN_DISABLED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_SZONE, LOCATION_SZONE);
    e1.set_target_card(duel, distg);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_SELF_DESTROY);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_SZONE, LOCATION_SZONE);
    e3.set_target_card(duel, distg);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_WARRIOR) && c.is_controler(duel, tp) && c.is_location(duel, LOCATION_MZONE)
}

fn distg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.get_card_target_count(duel) > 0
        && c.is_spell(duel)
        && c.get_card_target(duel)
            .is_exists(duel, |c, duel| cfilter(c, duel, e.get_handler_player(duel)), 1, None)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, ev) = (a.tp, a.ev as u8);
    let Some(re) =
        a.re.filter(|re| re.is_spell_effect(duel) && re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET))
    else {
        return Ok(());
    };
    let Some(g) = duel.get_chain_info::<Option<CardSet>>(ev, CHAININFO_TARGET_CARDS) else {
        return Ok(());
    };
    if g.is_exists(duel, |c, duel| cfilter(c, duel, tp), 1, None)
        && duel.negate_effect(ev)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(re.get_handler(duel), REASON_EFFECT)?;
    }
    Ok(())
}
