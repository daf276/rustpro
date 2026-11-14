// Metal Illusionist
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 29157292;

pub const LISTED_SERIES: &[u16] = &[SET_METALMORPH];
pub const LISTED_NAMES: &[u32] = &[CARD_MAX_METALMORPH];

// In the hand to Normal Summon and on the field for the Type change, with Max Metalmorph in the Deck to Set and in the Graveyard
// to draw.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_MAX_METALMORPH, DECK),
    (CARD_MAX_METALMORPH, DECK),
    (CARD_MAX_METALMORPH, DECK),
    (CARD_MAX_METALMORPH, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.e.get_handler_player(duel)));
    e1.set_value(duel, 4);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_target(duel, changetypetg);
    e4.set_operation(duel, changetypeop);
    c.register_effect(duel, e4);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_METALMORPH]) && c.is_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(setfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty()
        && duel.sset(tp, g)? > 0
        && duel.is_existing_matching_card(|c, duel| c.is_code(duel, CARD_MAX_METALMORPH), tp, LOCATION_GRAVE, 0, 1, None)
        && duel.is_player_can_draw(tp, 1)
    {
        duel.shuffle_deck(tp)?;
        if duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.break_effect()?;
            duel.draw(tp, 1, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn changetypetg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let race = a.e.get_handler(duel).announce_another_race(duel, a.tp)?;
    a.e.set_label(duel, race as i64);
    Ok(false)
}

fn changetypeop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_RACE);
        e1.set_value(duel, a.e.get_label(duel) as i32);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END | RESET_OPPO_TURN);
        c.register_effect(duel, e1);
    }
    Ok(())
}
