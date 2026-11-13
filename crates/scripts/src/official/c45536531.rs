// Funny Dark Rabbit
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 45536531;

pub const LISTED_NAMES: &[u32] = &[CARD_TOON_WORLD];

// Copies in the hand, with Toon World and Toon World the Perfect World in the Deck to add or place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, 0),
    (CARD_TOON_WORLD, DECK),
    (CARD_TOON_WORLD, DECK),
    (7293697, DECK),
    (7293697, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_operation(duel, regop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_ADD_TYPE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, _| {
        duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_TOON_WORLD)),
            0,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            None,
        )
    });
    e2.set_value(duel, TYPE_TOON as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, thpltg);
    e3.set_operation(duel, thplop);
    c.register_effect(duel, e3);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if duel.has_flag_effect(tp, ID) {
        return Ok(());
    }
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e1.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.lists_code(duel, CARD_TOON_WORLD));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn thplfilter(c: CardId, duel: &Duel, tp: u8, szone_chk: bool) -> bool {
    c.is_set_card(duel, &[SET_TOON])
        && (c.is_continuous_spell(duel) || c.is_field_spell(duel))
        && (c.is_able_to_hand(duel)
            || (c.check_unique_on_field(duel, tp) && !c.is_forbidden(duel) && (c.is_field_spell(duel) || szone_chk)))
}

fn thpltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let szone_chk = duel.get_location_count(tp, LOCATION_SZONE) > 0;
        return Ok(duel.is_existing_matching_card(|c, duel| thplfilter(c, duel, tp, szone_chk), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thplop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let szone_chk = duel.get_location_count(tp, LOCATION_SZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let g = duel.select_matching_card(tp, |c, duel| thplfilter(c, duel, tp, szone_chk), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        aux::to_hand_or_else(
            duel,
            sc,
            tp,
            |sc, duel| szone_chk || sc.is_field_spell(duel),
            |duel| {
                if sc.is_field_spell(duel) {
                    duel.place_field_spell(sc, tp)?;
                } else {
                    duel.move_to_field(sc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
                }
                Ok(())
            },
            aux::stringid(ID, 4),
        )?;
    }
    Ok(())
}
