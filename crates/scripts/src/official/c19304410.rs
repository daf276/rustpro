// Tri-Brigade Springans Kitt
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 19304410;

pub const LISTED_SERIES: &[u16] = &[SET_TRI_BRIGADE, SET_SPRINGANS];
pub const LISTED_NAMES: &[u32] = &[ID, CARD_ALBAZ];

// In the hand and on the field with Tri-Brigade Mercourier and Fallen of Albaz in the Graveyard to Special Summon, and
// the opponent's Dark Hole to send it there.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (19096726, MZONE),
    (19096726, GRAVE),
    (68468459, GRAVE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, selfspcon);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, gysptg);
    e2.set_operation(duel, gyspop);
    c.register_effect(duel, e2);
}

fn selfspconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TRI_BRIGADE, SET_SPRINGANS]) && c.is_faceup(duel) && !c.is_code(duel, ID)
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(selfspconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_set_card(duel, &[SET_TRI_BRIGADE, SET_SPRINGANS]) || (c.is_code(duel, CARD_ALBAZ) || c.lists_code(duel, CARD_ALBAZ)))
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| gyspfilter(c, duel, a.e, a.tp), LOCATION_GRAVE)
}

fn gyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(
        duel,
        a,
        |c, duel| gyspfilter(c, duel, a.e, a.tp),
        LOCATION_GRAVE,
        POS_FACEUP_DEFENSE,
    )
}
