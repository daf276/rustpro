// Power Tool Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 2403771;

// In the Extra Deck, Synchro Summoned with Water Spirit and Binding Chain (Level 1 and 3), with Equip Spells in the Deck to search and
// Dragon Treasure in the hand to equip it, and the opponent's Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (487395, MZONE),
    (487395, MZONE),
    (8058240, MZONE),
    (8058240, MZONE),
    (1435851, HAND),
    (1435851, DECK),
    (1557499, DECK),
    (15052462, DECK),
    (37820550, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Message(MSG_SELECT_EFFECTYN),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |_, _| true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_target(duel, desreptg);
    e2.set_operation(duel, desrepop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_EQUIP) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK, 0, 3, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, 0, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if g.len() >= 3 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 3, 3, None)?;
        duel.confirm_cards(1 - tp, sg)?;
        duel.shuffle_deck(tp)?;
        let tg = sg.random_select(duel, 1 - tp, 1);
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_location(duel, LOCATION_SZONE) && !c.is_status(duel, STATUS_DESTROY_CONFIRMED)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        let g = c.get_equip_group(duel);
        return Ok(!c.is_reason(duel, REASON_REPLACE) && g.is_exists(duel, repfilter, 1, None));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    let g = c.get_equip_group(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let sg = g.filter_select(duel, tp, repfilter, 1, 1, None)?;
    duel.set_target_card(sg);
    Ok(true)
}

fn desrepop(duel: &mut Duel, _: &Args) -> Script<()> {
    if let Some(tg) = duel.get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS) {
        duel.send_to_grave(tg, REASON_EFFECT | REASON_REPLACE)?;
    }
    Ok(())
}
