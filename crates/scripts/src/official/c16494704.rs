// Odd-Eyes Advent
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 16494704;
const PENDULUMGRAPH: u32 = 66425726;

pub const LISTED_SERIES: &[u16] = &[SET_ODD_EYES];
// Odd-Eyes Pendulumgraph Dragon in the hand, with Odd-Eyes Arc Pendulum Dragon (Level 7 Pendulum) in the hand and "Odd-Eyes"
// monsters in the Extra Deck as materials.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (PENDULUMGRAPH, HAND),
    (14105623, HAND),
    (16691074, EXTRA),
    (80696379, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(PENDULUMGRAPH)];

struct Params;

impl RitualParams for Params {
    const LOCATION: Option<u32> = Some(LOCATION_HAND | LOCATION_GRAVE);

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_DRAGON)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8) -> Option<CardSet> {
        if duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0 && duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 1 {
            return Some(duel.get_matching_group(exfilter0, tp, LOCATION_EXTRA, 0, None));
        }
        None
    }

    fn extraop(duel: &mut Duel, mg: CardSet, _: EffectId, _: u8, _: CardId) -> Script<()> {
        let mat2 = mg.filter(duel, |c, duel| c.is_location(duel, LOCATION_EXTRA), None);
        duel.release_ritual_material(mg.difference(mat2))?;
        duel.send_to_grave(mat2, REASON_EFFECT + REASON_MATERIAL + REASON_RITUAL)?;
        Ok(())
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        (c.is_type(duel, TYPE_PENDULUM) && c.is_location(duel, LOCATION_HAND | LOCATION_ONFIELD))
            || (c.is_set_card(duel, &[SET_ODD_EYES]) && c.is_location(duel, LOCATION_EXTRA))
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Ritual::create_proc::<Params, RITPROC_GREATER>(duel, c);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    c.register_effect(duel, e1);
}

fn exfilter0(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ODD_EYES]) && c.is_level_above(duel, 1) && c.is_able_to_grave(duel)
}
