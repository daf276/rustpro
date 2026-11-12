// Performage Trapeze High Magician
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::{MSG_ATTACK, MSG_DAMAGE};

const ID: u32 = 6659193;

pub const LISTED_NAMES: &[u32] = &[17016362];
// Xyz Summoned from the Extra Deck with Level 5 Spellcasters (Leo Wizard, Illusionist Faceless Mage), with Performage Trick Clown on the
// field to be destroyed by the opponent's Raigeki and deal effect damage to be reflected. The second effect needs Trapeze Magician, an
// Xyz Monster, as material, which no Xyz Summon can use.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (4392470, MZONE),
    (4392470, MZONE),
    (28546905, MZONE),
    (67696066, MZONE),
    (67696066, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::Message(MSG_ATTACK),
    Check::Message(MSG_DAMAGE),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) > 0);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EFFECT_REFLECT_DAMAGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 1, 0);
    e3.set_condition(duel, reflectcond);
    e3.set_value(duel, Value::ReflectDamageMut(reflectvalue));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_condition(duel, atkcon);
    e4.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e4.set_operation(duel, atkop);
    c.register_effect(duel, e4);
}

fn reflectcond(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_overlay_count(duel) > c.get_flag_effect(duel, ID)
}

fn reflectvalue(duel: &mut Duel, e: EffectId, _: Option<EffectId>, _: u32, r: u32, rp: u8, _: Option<CardId>) -> bool {
    if r & REASON_EFFECT == 0 {
        return false;
    }
    duel.hint(HINT_CARD, rp, u64::from(ID));
    e.get_handler(duel).register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    true
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_able_to_enter_bp()
        && c.can_attack(duel)
        && c.get_effect_count(duel, EFFECT_EXTRA_ATTACK) < 2
        && c.get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_code(duel, 17016362), 1, None)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_value(duel, 2);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
