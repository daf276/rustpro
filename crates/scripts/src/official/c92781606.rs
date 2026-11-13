// X-Krawler Qualiark
use super::c39998992::{spcon, spop, sptg};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 92781606;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        c.is_set_card(duel, &[SET_KRAWLER])
    }
}

// Link Summoned from Krawler Spine and Axon, destroyed by Raigeki, with Krawlers of different names in the Graveyard to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (88316955, MZONE),
    (88316955, MZONE),
    (88316955, MZONE),
    (15710054, MZONE),
    (15710054, MZONE),
    (88316955, GRAVE),
    (15710054, GRAVE),
    (13597785, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_LEAVE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_value(duel, 300);
    e2.set_condition(duel, effcon);
    e2.set_label(duel, 2);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e4.set_target_range(duel, 0, 1);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_label(duel, 4);
    e4.set_condition(duel, |duel, a| effcon(duel, a) && duel.is_battle_phase());
    e4.set_value(duel, 1);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD);
    e5.set_code(duel, EFFECT_DIRECT_ATTACK);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_target_range(duel, LOCATION_MZONE, 0);
    e5.set_label(duel, 6);
    e5.set_condition(duel, effcon);
    c.register_effect(duel, e5);
}

fn effilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_KRAWLER])
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    duel.get_matching_group_count(effilter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, None) as i64 >= a.e.get_label(duel)
}
