// S-Force Edge Razor
use crate::prelude::*;
use crate::testing::{DECK, EXTRA, HAND, MZONE};

const ID: u32 = 91864689;

pub const LISTED_SERIES: &[u16] = &[SET_S_FORCE];
// S-Force monsters in the hand to Special Summon, and Link Monsters to Link Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (22180094, HAND),
    (22180094, HAND),
    (22180094, HAND),
    (ID, DECK),
    (ID, MZONE),
    (20515672, EXTRA),
    (4611341, EXTRA),
    (35334193, EXTRA),
];

// SForce.ColumnTarget
pub fn column_target(duel: &Duel, e: EffectId, cc: CardId) -> bool {
    let tp = e.get_handler_player(duel);
    cc.get_column_group(duel).is_exists(
        duel,
        |c, duel| c.is_set_card(duel, &[SET_S_FORCE]) && c.is_monster(duel) && c.is_controler(duel, tp) && c.is_faceup(duel),
        1,
        Some(cc.into()),
    )
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
    e3.set_value(duel, Value::Card(|duel, _, sc| i32::from(sc.is_link_above(duel, 3))));
    e3.set_target_range(duel, 0, LOCATION_MZONE);
    e3.set_target_card(duel, column_target);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_S_FORCE])
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_ATTACK, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP_ATTACK)
}
