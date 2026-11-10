// Ghostrick Jiangshi
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 80885284;
const GHOSTRICK_FAIRY: u32 = 36239585;
const GHOSTRICK_SCARE: u32 = 86516889;
const ELFNOTE_LUCINA: u32 = 13597785;

pub const RESET_FLAG_STANDARD_PHASE_END: u32 = RESET_EVENT | (RESETS_STANDARD_PHASE_END & !RESET_TURN_SET);

// Copies face-down in the Monster Zone with an attacker of the opponent's to flip them, Ghostrick Scare to flip them, and
// Ghostrick Fairy in the Deck (Level 2) to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (GHOSTRICK_SCARE, DECK),
    (GHOSTRICK_FAIRY, DECK),
    (GHOSTRICK_FAIRY, DECK),
    (ELFNOTE_LUCINA, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_SUMMON);
    e1.set_condition(duel, sumcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, postg::<ID, RESET_FLAG_STANDARD_PHASE_END>);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e3.set_code(duel, EVENT_FLIP);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn sfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_GHOSTRICK])
}

// The Normal Summon limit and the turn-set effect the Ghostrick monsters share.
pub fn sumcon(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(sfilter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
}

pub fn postg<const CODE: u32, const RESET: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_turn_set(duel) && c.get_flag_effect(duel, CODE) == 0);
    }
    c.register_flag_effect(duel, CODE, RESET, 0, 1);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(c.into()), 1, a.tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

pub fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        duel.change_position(c, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_level_below(duel, lv) && c.is_set_card(duel, &[SET_GHOSTRICK]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let count = duel.get_matching_group_count(sfilter, tp, LOCATION_MZONE, 0, None) as u32;
        return Ok(duel.is_existing_matching_card(|c, duel| thfilter(c, duel, count), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let count = duel.get_matching_group_count(sfilter, tp, LOCATION_MZONE, 0, None) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, count), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
