// The Stars of Raise Moon
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 100458024;
const CHOICE: u8 = 0;
// s.name_list[tp]: the count of placed names at `NAME_LIST | tp << 8`, the codes in the slots after it.
const NAME_LIST: u64 = (ID as u64) << 16;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_RAISE_MOON];

// In the hand with Raise Moon cards in the Deck, in the Graveyard, and a Spellcaster Raise Moon Xyz Monster on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (100458017, DECK),
    (100458018, DECK),
    (100458019, DECK),
    (100458022, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        aux::add_values_reset(duel, |duel| {
            duel.set_script_state(NAME_LIST, 0);
            duel.set_script_state(NAME_LIST | 1 << 8, 0);
        });
    });
}

fn is_listed(duel: &Duel, tp: u8, code: u32) -> bool {
    let list = NAME_LIST | u64::from(tp) << 8;
    (1..=duel.get_script_state(list) as u64).any(|slot| duel.get_script_state(list | slot) == i64::from(code))
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RAISE_MOON]) && !c.is_code(duel, ID) && !is_listed(duel, tp, c.get_code(duel))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_DECK, 0) > 1
            && duel.is_existing_matching_card(|c, duel| plfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_field_group_count(tp, LOCATION_DECK, 0) > 1 {
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
        let g = duel.select_matching_card(tp, |c, duel| plfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first() {
            duel.shuffle_deck(tp)?;
            duel.move_to_deck_top(sc.into())?;
            duel.confirm_decktop(tp, 1)?;
            let list = NAME_LIST | u64::from(tp) << 8;
            let count = duel.get_script_state(list) + 1;
            duel.set_script_state(list, count);
            duel.set_script_state(list | count as u64, i64::from(sc.get_code(duel)));
        }
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_TO_HAND);
    e1.set_target_range(duel, 1, 1);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_DECK));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

// Also read by "Raise Moon" the City that Never Sleeps.
pub(crate) fn drconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_set_card(duel, &[SET_RAISE_MOON]) && c.is_xyz_monster(duel) && c.is_faceup(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let option_1 = duel.is_player_can_draw(tp, 1) && duel.is_player_can_draw(1 - tp, 1);
    let option_2 = duel.is_player_can_draw(tp, 1) && duel.is_existing_matching_card(drconfilter, tp, LOCATION_MZONE, 0, 1, None);
    if chk == 0 {
        return Ok(option_1 || option_2);
    }
    let choice = duel.select_effect(tp, &[(option_1, aux::stringid(ID, 4)), (option_2, aux::stringid(ID, 5))])?;
    e.set_chain_data(duel, CHOICE, choice.map_or(0, |choice| choice as i64));
    let player = if choice == Some(1) { PLAYER_ALL } else { tp };
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, player, 1);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let choice = a.e.get_chain_data(duel, CHOICE).unwrap_or(0);
    if choice == 1 {
        duel.draw(tp, 1, REASON_EFFECT)?;
        duel.draw(1 - tp, 1, REASON_EFFECT)?;
    } else if choice == 2 {
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
