// Fallen of Albaz
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 68468459;

struct Params;

impl FusionSummonParams for Params {
    fn matfilter(c: CardId, duel: &Duel, e: EffectId, _: u8) -> bool {
        Fusion::check_with_handler(c, duel, e, |_, _| false)
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, |c, duel| c.is_faceup(duel));
        let g = duel.get_matching_group(filter, tp, 0, LOCATION_ONFIELD, None);
        (Some(g), Some((fcheck, e.get_handler(duel))))
    }

    fn gc(duel: &Duel, e: EffectId, _: u8) -> Option<CardSet> {
        Some(Fusion::forced_handler(duel, e))
    }
}

// In the hand to Normal Summon, and the opponent's LIGHT monster for Albion the Branded Dragon: Elfnote Fortuna.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (85976588, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_condition(duel, |duel, _| !duel.is_damage_step());
    e1a.set_cost(duel, Cost::discard);
    e1a.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e1a.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
}

// s.fcheck(c)(tp, sg, fc).
fn fcheck(duel: &Duel, c: CardId, tp: u8, sg: CardSet, _: CardId) -> bool {
    !sg.is_exists(
        duel,
        |tc, duel| tc.is_controler(duel, tp) && tc.is_on_field(duel),
        1,
        Some(c.into()),
    )
}
