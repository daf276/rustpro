// Armityle the Chaos Phantasm
use crate::prelude::*;
use crate::testing::{Check, MZONE};

pub const LISTED_NAMES: &[u32] = &[6007213, 32491822, 69890967];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(6007213),
        FusionMaterial::Code(32491822),
        FusionMaterial::Code(69890967),
    ];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::SpSummonCondition(splimit));

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(CardId::is_able_to_remove_as_cost, tp, LOCATION_ONFIELD, 0, None)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, _: CardId) -> Script<()> {
        duel.remove(g, POS_FACEUP, REASON_COST | REASON_MATERIAL).map(|_| ())
    }
}

// The three materials on the field.
pub const SUPPORT: &[(u32, u16)] = &[(6007213, MZONE), (32491822, MZONE), (69890967, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| duel.get_turn_player() == a.e.get_handler_player(duel));
    e2.set_value(duel, 10000);
    c.register_effect(duel, e2);
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, _: u32, _: u8, _: u8) -> bool {
    !e.get_handler(duel).is_location(duel, LOCATION_EXTRA)
}
