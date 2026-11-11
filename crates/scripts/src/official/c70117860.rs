// Windwitch - Snow Bell
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 70117860;

// Windwitch - Glass Bell on the field and in the hand, with Battlewasp - Azusa in the Extra Deck to Synchro Summon with this card.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (71007216, MZONE),
    (71007216, MZONE),
    (71007216, HAND),
    (71007216, HAND),
    (27565379, EXTRA),
    (27565379, EXTRA),
];
// The hint of its other effect lands on the Synchro Monster summoned with it, which no check reads.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_condition(duel, efcon);
    e2.set_operation(duel, efop);
    c.register_effect(duel, e2);
}

fn nonwindfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) || c.is_attribute_except(duel, ATTRIBUTE_WIND)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_attribute(duel, ATTRIBUTE_WIND)),
        tp,
        LOCATION_MZONE,
        0,
        2,
        None,
    ) && !duel.is_existing_matching_card(nonwindfilter, tp, LOCATION_MZONE, 0, 1, None)
}

fn efcon(duel: &Duel, a: &Args) -> bool {
    a.r == REASON_SYNCHRO
        && a.e
            .get_handler(duel)
            .get_reason_card(duel)
            .is_some_and(|rc| rc.is_attribute(duel, ATTRIBUTE_WIND))
}

fn efop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(rc) = c.get_reason_card(duel) else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3060);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_label(duel, i64::from(a.ep));
    e1.set_value(duel, Value::EffectPlayer(|duel, e, _, rp| i64::from(rp) == 1 - e.get_label(duel)));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    rc.register_effect(duel, e1);
    Ok(())
}
