// Void Imagination
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, SZONE};

const ID: u32 = 31444249;

pub const LISTED_SERIES: &[u16] = &[SET_INFERNOID];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_INFERNOID])
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        if dmcon(duel, tp) {
            let g = duel.get_matching_group(
                |c, duel| Fusion::is_monster_filter(c, duel, |c, duel| c.is_able_to_grave(duel)),
                tp,
                LOCATION_DECK,
                0,
                None,
            );
            return (Some(g), Some((fcheck, e.get_handler(duel))));
        }
        (None, None)
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, a.tp, LOCATION_DECK as i32);
        Ok(false)
    }
}

// Face-up in the Spell & Trap Zone with Infernoid Decatron and Pirmais in hand for Infernoid Evil in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (97051536, EXTRA),
    (97051536, EXTRA),
    (58446973, HAND),
    (60551528, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(97051536)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CHANGE_LEVEL);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_value(duel, 1);
    e2.set_target_card(duel, lvtg);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_card(duel, lvtg);
    e3.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, HALF_DAMAGE>));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_cost(duel, Cost::self_to_grave);
    e4.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e4.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e4);
}

// s.lvtg and s.rdtg, which are identical.
fn lvtg(duel: &Duel, _: EffectId, c: CardId) -> bool {
    c.is_set_card(duel, &[SET_INFERNOID]) && c.get_original_level(duel) >= 2
}

fn fcheck(duel: &Duel, _: CardId, _: u8, sg: CardSet, _: CardId) -> bool {
    sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None) <= 6
}

fn dmcon(duel: &Duel, tp: u8) -> bool {
    let from_extra = |c: CardId, duel: &Duel| c.is_summon_location(duel, LOCATION_EXTRA);
    !duel.is_existing_matching_card(from_extra, tp, LOCATION_MZONE, 0, 1, None)
        && duel.is_existing_matching_card(from_extra, tp, 0, LOCATION_MZONE, 1, None)
}
