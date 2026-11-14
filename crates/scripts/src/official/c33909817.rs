// Sylvan Princessprite
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 33909817;

pub const LISTED_SERIES: &[u16] = &[SET_SYLVAN];
// In the Extra Deck, Xyz Summoned with two Level 1 monsters (Dark Plant), with Sylvan Snapdrassinagon in the Graveyard to Special Summon and a
// Plant in the hand to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (13193642, MZONE),
    (13193642, MZONE),
    (13193642, HAND),
    (47741109, GRAVE),
    (47741109, GRAVE),
];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(1);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DECKDES);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0
        && duel.is_player_can_discard_deck(tp, 1)
        && duel.is_existing_matching_card(CardId::is_able_to_hand, tp, LOCATION_DECK, 0, 1, None))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.is_player_can_discard_deck(tp, 1) {
        return Ok(());
    }
    duel.confirm_decktop(tp, 1)?;
    let Some(tc) = duel.get_decktop_group(tp, 1).get_first() else {
        return Ok(());
    };
    duel.disable_shuffle_check(true);
    if tc.is_spell_trap(duel) && tc.is_able_to_hand(duel) {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.shuffle_hand(tp)?;
    } else {
        duel.send_to_grave(tc, REASON_EFFECT | REASON_EXCAVATE)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
        && c.is_race(duel, RACE_PLANT)
        && c.is_able_to_grave_as_cost(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |tc, duel| cfilter(tc, duel, tp),
            tp,
            LOCATION_HAND | LOCATION_MZONE,
            0,
            1,
            Some(c.into()),
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |tc, duel| cfilter(tc, duel, tp),
        tp,
        LOCATION_HAND | LOCATION_MZONE,
        0,
        1,
        1,
        Some(c.into()),
    )?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SYLVAN]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
