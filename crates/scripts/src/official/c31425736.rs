// Cupsy☆Yummy
use super::c4215180::{add_special_summon_procedure, is_summoned_by_synchro_effect};
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 31425736;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand and the Graveyard, with Cooky★Yummy Way (a Level 2 Synchro) on the field to Special Summon it from the hand and
// to Special Summon it from the Graveyard, the opponent's Raigeki to respond to, and Yummy monsters in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (67098897, MZONE),
    (67098897, MZONE),
    (10966439, DECK),
    (10966439, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_special_summon_procedure::<ID>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DRAW);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_YUMMY]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let sp_chk = is_summoned_by_synchro_effect(duel, a);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None) || (sp_chk && duel.is_player_can_draw(tp, 1)));
    }
    e.set_label(duel, i64::from(sp_chk));
    if sp_chk {
        duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
        duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    } else {
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    }
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let sp_chk = e.get_label(duel) == 1;
    let b1 = duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = sp_chk && duel.is_player_can_draw(tp, 1);
    if !(b1 || b2) {
        return Ok(());
    }
    let op = if sp_chk {
        duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?
    } else {
        Some(1)
    };
    if op == Some(1) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    } else if op == Some(2) {
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
