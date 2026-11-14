// Reeshaddoll Wendikurhu
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 8852158;

pub const LISTED_SERIES: &[u16] = &[SET_SHADDOLL];
pub const MATERIAL_SETCODE: &[u16] = &[SET_SHADDOLL];

// In the Extra Deck, Fusion Summoned by Polymerization with Shaddoll Squamata and Dancing Elf (WIND), with face-down Shaddoll
// Dragons (Flip) to change.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (30328508, HAND),
    (59983499, HAND),
    (77723643, MZONE),
    (77723643, MZONE),
    (77723643, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_first_be_fusion_summoned(duel);
    Fusion::add_proc_mix::<super::c94977269::Materials<ATTRIBUTE_WIND>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e2);
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_position(duel, POS_FACEDOWN_DEFENSE) && c.is_can_change_position(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(posfilter, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let positions = (POS_FACEUP_DEFENSE | POS_FACEDOWN_DEFENSE) as i32;
    duel.set_operation_info(0, CATEGORY_POSITION, None, 1, PLAYER_EITHER, positions);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let facedown_ct = duel.get_matching_group_count(posfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if facedown_ct == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_matching_card(tp, posfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, facedown_ct as u8, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    if duel.change_position(g, POS_FACEUP_DEFENSE)? == 0 {
        return Ok(());
    }
    let og = duel.get_operated_group();
    let flip_count = og.filter_count(duel, |c, duel| c.is_type(duel, TYPE_FLIP), None);
    if flip_count > 0
        && duel.is_existing_matching_card(CardId::is_can_turn_set, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(og))
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
        let pos_g = duel.select_matching_card(
            tp,
            CardId::is_can_turn_set,
            tp,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            flip_count as u8,
            Some(og),
        )?;
        if !pos_g.is_empty() {
            duel.hint_selection(pos_g);
            duel.break_effect()?;
            duel.change_position(pos_g, POS_FACEDOWN_DEFENSE)?;
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SHADDOLL]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}
