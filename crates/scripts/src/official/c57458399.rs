// Satellite Synchron
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 57458399;

pub const LISTED_SERIES: &[u16] = &[SET_WARRIOR, SET_STARDUST, SET_SYNCHRON];
// In hand and on the field (Level 2) with Junk Warrior in the Graveyard, and Monster Reborn to Special Summon from there.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (CARD_JUNK_WARRIOR, GRAVE), (83764718, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, condition);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn gfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.get_previous_location(duel) == LOCATION_GRAVE && c.get_previous_controler(duel) == tp
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| gfilter(c, duel, tp), 1, None)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel)
        && (c.is_original_set_card(duel, &[SET_WARRIOR])
            || c.is_original_set_card(duel, &[SET_STARDUST])
            || c.is_original_set_card(duel, &[SET_SYNCHRON]))
        && c.is_type(duel, TYPE_SYNCHRO)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_level(duel) != 4 && duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, 4);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
