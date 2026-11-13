// Magical Musketeer Starfire
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, SZONE};

const ID: u32 = 31629407;

pub const LISTED_NAMES: &[u32] = &[ID];

// On the field with a Spell/Trap Card in its column, and in the Deck. No other Level 4 or lower "Magical Musket" monster is ported to
// Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, DECK),
    (5318639, SZONE),
    (5318639, SZONE),
    (5318639, SZONE),
];
pub const CHECKS: &[Check] = &[Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1)))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_code(duel, EFFECT_QP_ACT_IN_NTPHAND);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_target_range(duel, LOCATION_HAND, 0);
    e1a.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_MAGICAL_MUSKET]));
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| {
        re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && a.e.get_handler(duel).get_column_group(duel).contains(re.get_handler(duel))
    })
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4)
        && c.is_set_card(duel, &[SET_MAGICAL_MUSKET])
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP_DEFENSE)
}
