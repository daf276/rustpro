// Tenyi Spirit - Mula Adhara
use super::c44035031::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 73121813;

// In the Extra Deck, Synchro Summoned with Labradorite Dragon (a Level 6 Tuner) and Doron (Level 2), with Wyrms (Phantasm Spiral
// Dragon) in the Graveyard and a Field Spell (Umi) in the Deck to add; Doron on the field is a non-Effect Monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (62514770, MZONE),
    (62514770, MZONE),
    (756652, MZONE),
    (756652, MZONE),
    (756652, MZONE),
    (56649609, GRAVE),
    (56649609, GRAVE),
    (22702055, DECK),
    (22702055, DECK),
    (22702055, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, _| {
        duel.is_existing_matching_card(
            aux::faceup_filter(CardId::is_non_effect_monster),
            0,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            None,
        )
    });
    e2.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e3);
    let e4 = e2.clone_effect(duel);
    e4.set_code(duel, EFFECT_EXTRA_ATTACK);
    e4.set_value(duel, 1);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_field_spell(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None)
            && duel.is_existing_matching_card(
                aux::faceup_filter(|c, duel| c.is_race(duel, RACE_WYRM)),
                tp,
                LOCATION_GRAVE | LOCATION_REMOVED,
                0,
                2,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
