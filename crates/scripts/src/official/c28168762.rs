// Sunvine Gardna
use super::c65563871::{Procedure, descon, desop, destg};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 28168762;

// On the field and in the Extra Deck, with Sunavalon Dryas in the Extra Deck to be Link Summoned pointing at it, and strong
// monsters of the opponent's (Blue-Eyes White Dragon) to attack it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (27520594, MZONE),
    (27520594, MZONE),
    (93896655, EXTRA),
    (93896655, EXTRA),
    (93896655, MZONE),
    (93896655, MZONE),
    (93896655, MZONE),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
];
// The halved damage needs a Link Summoned Sunavalon Dryas pointing at Gardna, which random play rarely reaches.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_LEAVE_FIELD);
    e1.set_condition(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, dmgcon);
    e2.set_operation(duel, dmgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_DESTROYED);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn dmgfilter(c: CardId, duel: &Duel, cc: CardId) -> bool {
    c.is_set_card(duel, &[SET_SUNAVALON]) && c.is_type(duel, TYPE_LINK) && c.get_linked_group(duel).is_contains(cc)
}

fn dmgcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    c.is_faceup(duel)
        && c.is_relate_to_battle(duel)
        && c.get_battle_target(duel).is_some()
        && duel.is_existing_matching_card(|cc, duel| dmgfilter(cc, duel, c), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
}

fn dmgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, HALF_DAMAGE as i32);
    e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e2 = duel.create_effect(a.e.get_handler(duel));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_ADJUST);
    e2.set_operation(duel, skop);
    e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
    duel.register_effect(e2, a.tp);
    Ok(())
}

fn skop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.get_current_phase() != PHASE_DAMAGE {
        let p = duel.get_turn_player();
        duel.skip_phase(p, PHASE_BATTLE, RESET_PHASE | PHASE_BATTLE_STEP, 1);
        a.e.reset(duel);
    }
    Ok(())
}
