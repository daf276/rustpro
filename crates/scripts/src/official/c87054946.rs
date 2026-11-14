// Nephthys, the Sacred Flame
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 87054946;

pub const LISTED_SERIES: &[u16] = &[SET_NEPHTHYS];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(99);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_RITUAL, Some(lc), sumtype, tp), 1, None)
    }
}

// Link Summoned with Ritual Monsters (Zera the Mant) among the materials. Its hints go on it in the Extra Deck, which CardHint cannot count.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (69123138, MZONE),
    (69123138, MZONE),
    (69123138, MZONE),
    (98446407, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_sequence(duel) > 4);
    e2.set_value(
        duel,
        Value::Card(|duel, _, c| i32::from(c.is_faceup(duel) && c.is_set_card(duel, &[SET_NEPHTHYS]) && c.get_sequence(duel) < 5)),
    );
    c.register_effect(duel, e2);
}

fn grant(duel: &mut Duel, c: CardId, code: u32, property: u32, value: Option<i32>) {
    let effect = duel.create_effect(c);
    effect.set_type(duel, EFFECT_TYPE_SINGLE);
    if property != 0 {
        effect.set_property(duel, property);
        effect.set_range(duel, LOCATION_MZONE);
    }
    effect.set_code(duel, code);
    if let Some(value) = value {
        effect.set_value(duel, value);
    }
    effect.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
    c.register_effect(duel, effect);
}

fn hint(duel: &mut Duel, c: CardId, n: u64) {
    c.register_flag_effect_full(
        duel,
        0,
        RESET_EVENT | RESETS_STANDARD,
        EFFECT_FLAG_CLIENT_HINT,
        1,
        0,
        aux::stringid(ID, n),
    );
}

fn matcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let ct = c
        .get_material(duel)
        .filter_count(duel, |mc, duel| mc.is_type(duel, TYPE_RITUAL), None);
    if ct >= 1 {
        grant(duel, c, EFFECT_INDESTRUCTABLE_BATTLE, 0, Some(1));
        hint(duel, c, 0);
    }
    if ct >= 2 {
        grant(duel, c, EFFECT_UPDATE_ATTACK, EFFECT_FLAG_SINGLE_RANGE, Some(1200));
        grant(duel, c, EFFECT_INDESTRUCTABLE_EFFECT, 0, Some(1));
        hint(duel, c, 1);
    }
    if ct == 3 {
        grant(duel, c, EFFECT_UPDATE_ATTACK, EFFECT_FLAG_SINGLE_RANGE, Some(1200));
        grant(duel, c, EFFECT_CANNOT_BE_EFFECT_TARGET, EFFECT_FLAG_SINGLE_RANGE, Some(1));
        hint(duel, c, 2);
    }
}
