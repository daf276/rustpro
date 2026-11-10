// Primite Roar
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 92501449;

// In the hand and Graveyard with Mystical Elf (Normal Monster) in the Deck to summon and on the field to banish with, against the
// opponent's monsters (Sangan) summoned or on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (15025844, DECK),
    (15025844, DECK),
    (15025844, GRAVE),
    (63198739, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_BATTLE_START | TIMINGS_CHECK_MONSTER_E);
    e1.set_cost(duel, Cost::pay_lp::<2000>);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_condition(duel, |duel, a| {
        let tp = a.tp;
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - tp), 1, None)
    });
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, rmvtg);
    e2.set_operation(duel, rmvop);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let code = duel.announce_card(tp, &[u64::from(TYPE_NORMAL), OPCODE_ISTYPE])?;
    duel.set_target_param(code);
    duel.set_operation_info(0, CATEGORY_ANNOUNCE, None, 0, tp, ANNOUNCE_CARD_FILTER as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_type(duel, TYPE_NORMAL)
        && c.is_code(duel, code)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let code: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_label(duel, i64::from(code));
    e1.set_target_card(duel, |duel, e, c| {
        c.is_set_card(duel, &[SET_PRIMITE]) || (c.is_code(duel, e.get_label(duel) as u32) && c.is_type(duel, TYPE_NORMAL))
    });
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END | RESET_OPPO_TURN);
    duel.register_effect(e1, tp);
    aux::register_client_hint_full(
        duel,
        c,
        0,
        tp,
        1,
        0,
        Some(aux::stringid(ID, 2)),
        RESET_PHASE | PHASE_END | RESET_OPPO_TURN,
        0,
    );
    if duel.get_field_group_count(tp, LOCATION_MZONE, 0) > 0 || duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp, code), tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.break_effect()?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

fn rmvfilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_able_to_remove(duel) && c.is_faceup(duel) && c.get_attack(duel) < atk
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_type(duel, TYPE_NORMAL)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(
            |rc, duel| rmvfilter(rc, duel, c.get_attack(duel)),
            tp,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            None,
        )
}

fn rmvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && chkc.is_controler(duel, tp) && tgfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| tgfilter(c, duel, tp), tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(
        tp,
        |c, duel| tgfilter(c, duel, tp),
        tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn rmvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel)) {
        return Ok(());
    }
    let atk = tc.get_attack(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| rmvfilter(c, duel, atk),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.remove(g, 0, REASON_EFFECT)?;
    }
    Ok(())
}
