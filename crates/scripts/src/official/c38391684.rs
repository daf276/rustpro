// Atlantis, City of the Sea Dragon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 38391684;

pub const LISTED_NAMES: &[u32] = &[CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON, CARD_UMI];

// Copies in the hand and the Graveyard, with Levia-Dragon of Atlantis - Daedalus on the field, which mentions Atlantis and is named Umi.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, GRAVE), (64603351, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, LOCATION_HAND | LOCATION_MZONE);
    e1.set_condition(duel, |duel, _| {
        duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.lists_code(duel, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON)),
            0,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            None,
        )
    });
    e1.set_value(duel, -1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, 1, 0);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_UMI)),
            a.tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
    });
    e3.set_target(duel, |duel, a, chk, _| {
        let c = a.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.check_unique_on_field(duel, a.tp));
        }
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, a.tp, 0);
        Ok(false)
    });
    e3.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        if c.is_relate_to_effect(duel, a.e) {
            duel.place_field_spell(c, a.tp)?;
        }
        Ok(())
    });
    c.register_effect(duel, e3);
}
