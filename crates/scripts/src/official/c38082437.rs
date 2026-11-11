// Morphtronic Telefon
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE};

const ID: u32 = 38082437;

// On the field in both positions, with Level 1 to 4 Morphtronics in the Graveyard to Special Summon and in the Deck to excavate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, GRAVE),
    (84592800, GRAVE),
    (84592800, DECK),
    (84592800, DECK),
    (55119278, DECK),
    (55119278, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

// s.roll_dice, which only cards that are not ported read, is left out.
pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_attack_pos(duel));
    e1.set_target(duel, target_a);
    e1.set_operation(duel, operation_a);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_defense_pos(duel));
    e2.set_target(duel, target_d);
    e2.set_operation(duel, operation_d);
    c.register_effect(duel, e2);
}

fn target_a(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DICE, None, 0, tp, 1);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, lv: u32, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MORPHTRONIC]) && c.is_level_below(duel, lv) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn operation_a(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(&res) = duel.toss_dice(tp, 1)?.first() else {
        return Ok(());
    };
    let res = u32::from(res);
    if duel.recover(tp, res * 100, REASON_EFFECT)? != res * 100 || duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, res, e, tp));
    let g = duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
            duel.break_effect()?;
            duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

fn target_d(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_DECK, 0) > 0);
    }
    duel.set_operation_info(0, CATEGORY_DICE, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MORPHTRONIC]) && c.is_able_to_grave(duel)
}

fn operation_d(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_field_group_count(tp, LOCATION_DECK, 0) == 0 {
        return Ok(());
    }
    let Some(&res) = duel.toss_dice(tp, 1)?.first() else {
        return Ok(());
    };
    duel.confirm_decktop(tp, usize::from(res))?;
    let mut dg = duel.get_decktop_group(tp, usize::from(res));
    let mut ct = 0;
    if dg.is_exists(duel, tgfilter, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let tg = dg.filter_select(duel, tp, tgfilter, 1, 1, None)?;
        dg = dg.difference(tg);
        duel.disable_shuffle_check(true);
        duel.send_to_grave(tg, REASON_EFFECT | REASON_EXCAVATE)?;
        ct = 1;
    }
    let ac = u16::from(res) - ct;
    if ac == 0 {
        return Ok(());
    }
    if duel.select_option(tp, &[aux::stringid(ID, 4), aux::stringid(ID, 5)])? == 0 {
        duel.move_to_deck_top(dg)?;
        duel.sort_decktop(tp, tp, ac)?;
    } else {
        duel.move_to_deck_bottom(dg)?;
        duel.sort_deckbottom(tp, tp, ac)?;
    }
    Ok(())
}
