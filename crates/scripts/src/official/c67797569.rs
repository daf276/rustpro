// Lavalval Salamander
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 67797569;

pub const LISTED_SERIES: &[u16] = &[SET_LAVAL];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_attribute_full(duel, ATTRIBUTE_FIRE, Some(sc), sumtype, tp)
    }
}

// Laval Coatl (Level 2 Tuner) with Molten Behemoth for a Synchro Summon from the Extra Deck, more Laval monsters on the field and
// in the Graveyard, and monsters on the opponent's side to turn face-down.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (45439263, MZONE),
    (17192817, MZONE),
    (2407147, MZONE),
    (15169262, GRAVE),
    (45042329, MZONE | OPPONENT),
    (45042329, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, stock::draw_target::<2>);
    e1.set_operation(duel, drop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, poscost);
    e2.set_target(duel, postg);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (sg.is_exists(duel, |c, duel| c.is_attribute(duel, ATTRIBUTE_FIRE), 1, None), false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    duel.shuffle_hand(p)?;
    duel.break_effect()?;
    let cg = duel.get_field_group(p, LOCATION_HAND, 0);
    let og = aux::select_unselect_group_full(duel, cg, 2, 2, &rescon, tp, HINTMSG_TOGRAVE, Some(&rescon), false)?;
    if og.len() >= 2 {
        duel.send_to_grave(og, REASON_EFFECT)?;
    } else {
        duel.confirm_cards(1 - p, cg)?;
        duel.send_to_deck(cg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        duel.shuffle_deck(p)?;
    }
    Ok(())
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn poscost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, costfilter, LOCATION_MZONE | LOCATION_GRAVE)
}

fn laval_count(duel: &Duel, tp: u8) -> usize {
    duel.get_matching_group_count(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_LAVAL])),
        tp,
        LOCATION_MZONE,
        0,
        None,
    )
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = laval_count(duel, tp);
    let g = duel.get_matching_group(aux::faceup_filter(CardId::is_can_turn_set), tp, 0, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(ct > 0 && !g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g), 1, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = laval_count(duel, tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_matching_card(
        tp,
        aux::faceup_filter(CardId::is_can_turn_set),
        tp,
        0,
        LOCATION_MZONE,
        1,
        ct as u8,
        None,
    )?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.change_position(g, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}
