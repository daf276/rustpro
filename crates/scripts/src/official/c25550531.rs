// Purrely
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 25550531;

pub const LISTED_SERIES: &[u16] = &[SET_PURRELY];

// Purrely Happy Memory in the Deck to excavate and in the hand to reveal for Epurrely Happiness, which lists it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (82105704, HAND),
    (82105704, HAND),
    (82105704, DECK),
    (82105704, DECK),
    (52645235, EXTRA),
    (52645235, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_DECK, 0) >= 3);
    }
    duel.set_target_player(tp);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PURRELY]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let mut ac = 3;
    duel.confirm_decktop(p, ac)?;
    let g = duel.get_decktop_group(p, ac);
    if !g.is_empty() && g.is_exists(duel, thfilter, 1, None) && duel.select_yes_no(p, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, p, HINTMSG_ATOHAND);
        let sg = g.filter_select(duel, p, thfilter, 1, 1, None)?;
        duel.disable_shuffle_check(true);
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - p, sg)?;
        duel.shuffle_hand(p)?;
        ac -= 1;
    }
    if ac > 0 {
        duel.move_to_deck_bottom_count(ac, tp)?;
        duel.sort_deckbottom(tp, tp, ac as u16)?;
    }
    Ok(())
}

fn spcfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_set_card(duel, &[SET_PURRELY])
        && c.is_type(duel, TYPE_QUICKPLAY)
        && !c.is_public(duel)
        && duel.is_existing_matching_card(|xc, duel| spfilter(xc, duel, e, tp, mc, c), tp, LOCATION_EXTRA, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId, sc: CardId) -> bool {
    c.is_type_full(duel, TYPE_XYZ, Some(c), SUMMON_TYPE_XYZ as u64, tp)
        && c.lists_code(duel, sc.get_code(duel))
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(super::c79933029::must_material_ok(duel, tp, c)
            && duel.is_existing_matching_card(|rc, duel| spcfilter(rc, duel, e, tp, c), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) || c.is_controler(duel, 1 - tp) || c.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    if !super::c79933029::must_material_ok(duel, tp, c) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let Some(rc) = duel
        .select_matching_card(tp, |rc, duel| spcfilter(rc, duel, e, tp, c), tp, LOCATION_HAND, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.confirm_cards(1 - tp, rc)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    if let Some(sc) = duel
        .select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp, c, rc), tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    {
        sc.set_material(duel, Some(c.into()));
        duel.overlay(sc, c)?;
        if duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
            sc.complete_procedure(duel);
            duel.overlay(sc, rc)?;
        }
    }
    Ok(())
}
