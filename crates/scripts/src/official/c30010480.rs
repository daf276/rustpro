// Gouki Thunder Ogre
use super::c22510667::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 30010480;

// Link Summoned from Gouki Guts and Gouki Machine Suprex, with Gouki monsters in the hand to Normal Summon and the opponent's
// Smashing Ground (which leaves the Link Monster, with no Defense, alone) to destroy the monsters it points to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (7540107, MZONE),
    (29099860, MZONE),
    (7540107, MZONE),
    (7540107, HAND),
    (29099860, HAND),
    (97169186, HAND | OPPONENT),
    (97169186, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure<2, 0>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_HAND, LOCATION_HAND);
    e1.set_value(duel, Value::ExtraSummon(sumval));
    c.register_effect(duel, e1);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, atkcon);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn sumval(duel: &Duel, e: EffectId, c: CardId) -> (i32, i32, i32) {
    let (handler, tp) = (e.get_handler(duel), e.get_handler_player(duel));
    if c.is_controler(duel, tp) {
        (0, handler.get_linked_zone(duel) as i32, -(1 << handler.get_sequence(duel)))
    } else {
        (
            0,
            handler.get_linked_zone_full(duel, 1 - tp) as i32,
            -(1 << (handler.get_sequence(duel) + 16)),
        )
    }
}

fn cfilter(c: CardId, duel: &Duel, tp: u8, zone: u32) -> bool {
    let seq = c.get_previous_sequence(duel) + if c.is_previous_controler(duel, tp) { 0 } else { 16 };
    c.is_reason(duel, REASON_BATTLE | REASON_EFFECT) && c.is_previous_location(duel, LOCATION_MZONE) && zone >> seq & 1 != 0
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let zone = a.e.get_handler(duel).get_linked_zone(duel);
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| cfilter(c, duel, a.tp, zone), 1, None)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
        e1.set_value(duel, 400);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
