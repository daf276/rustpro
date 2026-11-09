// Arc Rebellion Xyz Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 64276752;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
}

// In the Extra Deck, Xyz Summoned with three Level 5 monsters, and on the field against Sangan of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (4035199, MZONE),
    (7562372, MZONE),
    (19737320, MZONE),
    (4035199, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_value(duel, 1);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_NEGATE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, ID);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_base_attack(duel) > 0
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let handler = a.e.get_handler(duel);
    Ok(chk == 0 && duel.is_existing_matching_card(atkfilter, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(handler.into())))
}

fn ovfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_type(duel, TYPE_XYZ)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_code(duel, EFFECT_CANNOT_ATTACK);
    e0.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e0.set_target_range(duel, LOCATION_MZONE, 0);
    e0.set_target_card(duel, |duel, e, c| e.get_label(duel) != i64::from(c.get_field_id(duel)));
    e0.set_label(duel, i64::from(c.get_field_id(duel)));
    e0.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e0, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
    if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel)) {
        return Ok(());
    }
    let g = duel.get_matching_group(atkfilter, tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    let atk = g.get_sum(duel, |c, duel| i64::from(c.get_base_attack(duel))) as i32;
    if c.update_attack(duel, atk, None, None, None) == atk && c.get_overlay_group(duel).is_exists(duel, ovfilter, 1, None) {
        let g1 = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
        if !g1.is_empty() {
            duel.break_effect()?;
        }
        for nc in g1.filter(duel, CardId::is_negatable_monster, None) {
            for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
                let e = duel.create_effect(c);
                e.set_type(duel, EFFECT_TYPE_SINGLE);
                e.set_code(duel, code);
                e.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                nc.register_effect(duel, e);
            }
            if nc.is_type(duel, TYPE_TRAPMONSTER) {
                let e3 = duel.create_effect(c);
                e3.set_type(duel, EFFECT_TYPE_SINGLE);
                e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
                e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                nc.register_effect(duel, e3);
            }
        }
    }
    Ok(())
}
