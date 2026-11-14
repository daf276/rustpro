// Bloom Prima the Melodious Choir
use super::c56208713::melodious;
use super::c62895219::grave_to_hand_target;
use super::c84988419::maestra;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 24672164;

pub const LISTED_SERIES: &[u16] = &[SET_MELODIOUS, SET_MELODIOUS_MAESTRA];
pub const MATERIAL_SETCODE: &[u16] = &[SET_MELODIOUS];

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial = FusionMaterial::Filter(melodious);
    const MINC: i32 = 1;
    const MAXC: i32 = 99;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(maestra)];
}

// In the Extra Deck, Fusion Summoned by Polymerization from Bacha and Aria, with Melodious monsters in the GY to add and the
// opponent's Dark Hole to send it there.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (56208713, MZONE),
    (40502912, MZONE),
    (79514956, MZONE),
    (40502912, GRAVE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_MATERIAL_CHECK);
    e2.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_EXTRA_ATTACK);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_TOHAND);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e4.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_previous_location(duel, LOCATION_MZONE) && c.is_fusion_summoned(duel)
    });
    e4.set_target(duel, thtg);
    e4.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e4);
}

fn matcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let ct = c.get_material_count(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, ct as i32 * 300);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE & !RESET_TOFIELD);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MELODIOUS]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    grave_to_hand_target(duel, a, chk, chkc, filter)
}
