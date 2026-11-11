// Kewl Tune Crackle
use super::c15665977::Procedure;
use super::c65961304::valcheck;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 39576656;

pub const LISTED_NAMES: &[u32] = &[43904702];
pub const MATERIAL: &[u32] = &[43904702];

struct ReturnOp;

impl aux::DelayedOperation for ReturnOp {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        let (facedown_g, faceup_g) = ag.split(duel, |c, duel| c.is_previous_position(duel, POS_FACEDOWN), None);
        if let Some(tc) = facedown_g.get_first() {
            duel.send_to_deck(facedown_g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
            let player = tc.get_controler(duel);
            duel.shuffle_extra(player)?;
        }
        if !faceup_g.is_empty() {
            duel.send_to_extra_p(faceup_g, PLAYER_NONE, REASON_EFFECT)?;
        }
        Ok(())
    }
}

// Synchro Summoned with Kewl Tune Clip and Cue on the field, and Raigeki in the opponent's hand to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (43904702, MZONE),
    (43904702, MZONE),
    (16387555, MZONE),
    (16387555, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure<43904702>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, bantg);
    e1.set_operation(duel, banop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_synchro_summoned(duel) && c.is_previous_location(duel, LOCATION_MZONE)
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_MATERIAL_CHECK);
    e3.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e3);
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, 0, LOCATION_EXTRA) > 0 && duel.is_player_can_remove(tp, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, LOCATION_EXTRA as i32);
    duel.set_possible_operation_info(0, CATEGORY_ATKCHANGE, Some(a.e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_field_group(tp, 0, LOCATION_EXTRA);
    if g.is_empty() {
        return Ok(());
    }
    duel.confirm_cards(tp, g)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let rc = g.filter_select(duel, tp, CardId::is_able_to_remove, 1, 1, None)?.get_first();
    if let Some(rc) = rc
        && aux::remove_until_desc::<ReturnOp>(
            duel,
            rc,
            POS_FACEUP,
            REASON_EFFECT | REASON_TEMPORARY,
            PHASE_END,
            ID,
            e,
            tp,
            aux::stringid(ID, 2),
        )?
        .is_some()
        && rc.is_location(duel, LOCATION_REMOVED)
        && rc.is_faceup(duel)
    {
        let c = e.get_handler(duel);
        let atk = rc.get_attack(duel);
        if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && atk > 0 && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
            duel.break_effect()?;
            c.update_attack(duel, atk, None, None, None);
        }
    }
    duel.shuffle_extra(1 - tp)?;
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 2, 1 - tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let g = duel.get_field_group(tp, 0, LOCATION_EXTRA);
    if g.len() < 2 || !duel.is_player_can_remove(tp, None) || !duel.select_yes_no(tp, aux::stringid(ID, 4))? {
        return Ok(());
    }
    duel.break_effect()?;
    duel.confirm_cards(tp, g)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let rg = g.filter_select(duel, tp, CardId::is_able_to_remove, 2, 2, None)?;
    if rg.len() == 2 {
        aux::remove_until_desc::<ReturnOp>(
            duel,
            rg,
            POS_FACEUP,
            REASON_EFFECT | REASON_TEMPORARY,
            PHASE_END,
            ID,
            e,
            tp,
            aux::stringid(ID, 5),
        )?;
    }
    duel.shuffle_extra(1 - tp)?;
    Ok(())
}
