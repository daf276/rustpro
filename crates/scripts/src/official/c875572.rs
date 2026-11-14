// Crowley, the Gifted Magistus
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 875572;

pub const LISTED_SERIES: &[u16] = &[SET_MAGISTUS, SET_INVOKED];

const ALEISTER_THE_INVOKER: u32 = 86120751;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_MAGISTUS, SET_INVOKED])
    }
}

pub const LISTED_NAMES: &[u32] = &[ALEISTER_THE_INVOKER];

// In hand to Normal Summon, with Invoked Magellanica in the Extra Deck to Fusion Summon using Crowley as Aleister the
// Invoker and EARTH Tao the Chanter, and a copy in the Deck to be searched by Spenta.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (48791583, EXTRA),
    (46247516, MZONE),
    (ID, DECK),
    (42544773, HAND),
    (42544773, HAND),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_ADD_CODE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, ALEISTER_THE_INVOKER as i32);
    e1.set_code_condition(duel, |_, _, sumtype, _| {
        sumtype & MATERIAL_FUSION > 0 || sumtype & u64::from(SUMMON_TYPE_FUSION) > 0
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_reason(duel, REASON_DRAW));
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, _| !duel.is_phase(PHASE_DAMAGE));
    e3.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e3.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
}
