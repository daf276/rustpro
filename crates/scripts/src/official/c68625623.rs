// Unchained Malevolent Magistrate Kamura
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 68625623;

// In the hand, with a monster and a Set card on the field to destroy, and on the field with Dark Hole in the opponent's hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (26236560, MZONE),
    (27412542, SZONE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON + CATEGORY_DRAW + CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_on_field(duel) && duel.get_mzone_count(tp, Some(chkc.into())) > 0);
    }
    let c = e.get_handler(duel);
    let mmz_rescon = aux::chkf_mmz(tp, 1);
    let g = duel.get_target_group(aux::TRUE, tp, LOCATION_ONFIELD, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty()
            && aux::select_unselect_group_check(duel, g, 1, 3, &mmz_rescon)
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    let tg = aux::select_unselect_group_full(duel, g, 1, 3, &mmz_rescon, tp, HINTMSG_DESTROY, Some(&mmz_rescon), false)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(tg), tg.len() as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    if tg.is_exists(duel, CardId::is_monster_card, 1, None) {
        duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    }
    if tg.is_exists(duel, CardId::is_spell_card, 1, None) {
        duel.set_possible_operation_info(0, CATEGORY_POSITION, None, 1, PLAYER_EITHER, LOCATION_MZONE as i32);
    }
    if tg.is_exists(duel, CardId::is_trap_card, 1, None) {
        duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if tg.is_empty() || duel.destroy(tg, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let og = duel.get_operated_group();
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        if og.is_exists(duel, CardId::is_monster_card, 1, None)
            && duel.is_player_can_draw(tp, 1)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.break_effect()?;
            duel.draw(tp, 1, REASON_EFFECT)?;
        }
        if og.is_exists(duel, CardId::is_spell_card, 1, None)
            && duel.is_existing_matching_card(CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 3))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
            let g = duel.select_matching_card(tp, CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.break_effect()?;
                duel.change_position_full(
                    g,
                    [POS_FACEUP_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
                    false,
                )?;
            }
        }
        if og.is_exists(duel, CardId::is_trap_card, 1, None)
            && duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into()))
            && duel.select_yes_no(tp, aux::stringid(ID, 4))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.break_effect()?;
                duel.destroy(g, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_ONFIELD) && c.is_reason(duel, REASON_EFFECT)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_MZONE, LOCATION_MZONE) > 0);
    }
    let g = duel.get_field_group(tp, LOCATION_MZONE, LOCATION_MZONE);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.tp, LOCATION_MZONE, LOCATION_MZONE);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
