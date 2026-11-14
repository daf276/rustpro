// Lollipo☆Yummy
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 4215180;

pub const LISTED_SERIES: &[u16] = &[SET_YUMMY];
// In the hand and the Graveyard, with Cooky★Yummy Way (a Level 2 Synchro) on the field to Special Summon it from the hand and
// to Special Summon it from the Graveyard, and the opponent's Raigeki to respond to and monsters in the opponent's Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (67098897, MZONE),
    (67098897, MZONE),
    (74677422, GRAVE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_special_summon_procedure::<ID>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_category(duel, CATEGORY_TODECK + CATEGORY_REMOVE);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

// The Special Summon from the hand the Level 1 "Yummy" monsters share.
pub fn add_special_summon_procedure<const ID: u32>(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
}

// Whether the card was Special Summoned by the effect of a Synchro Monster.
pub fn is_summoned_by_synchro_effect(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| {
        a.e.get_handler(duel).is_special_summoned(duel)
            && re.is_monster_effect(duel)
            && re.get_handler(duel).is_original_type(duel, TYPE_SYNCHRO)
    })
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_link(duel, &[1]) || (c.is_type(duel, TYPE_SYNCHRO) && c.is_level(duel, &[2]))) && c.is_faceup(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(spconfilter, tp, LOCATION_MZONE, 0, 1, None)
}

fn tgfilter(c: CardId, duel: &Duel, sp_chk: bool) -> bool {
    c.is_able_to_deck(duel) || (sp_chk && c.is_able_to_remove(duel))
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let sp_chk = is_summoned_by_synchro_effect(duel, a);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp)
            && chkc.is_location(duel, LOCATION_GRAVE)
            && tgfilter(chkc, duel, e.get_label(duel) == 1));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| tgfilter(c, duel, sp_chk), tp, 0, LOCATION_GRAVE, 1, None));
    }
    e.set_label(duel, i64::from(sp_chk));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, |c, duel| tgfilter(c, duel, sp_chk), tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    if sp_chk {
        duel.set_possible_operation_info(0, CATEGORY_TODECK, g, 1, tp, 0);
        duel.set_possible_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    } else {
        duel.set_operation_info(0, CATEGORY_TODECK, g, 1, tp, 0);
    }
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    let sp_chk = e.get_label(duel) == 1;
    let (b1, b2) = (tc.is_able_to_deck(duel), sp_chk && tc.is_able_to_remove(duel));
    if !(b1 || b2) {
        return Ok(());
    }
    let op = if sp_chk {
        duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?
    } else {
        Some(1)
    };
    if op == Some(1) {
        duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    } else if op == Some(2) {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
