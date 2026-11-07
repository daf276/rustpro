// The Phantom Knights of Umbrage Veil
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 85257384;

// Doomed Soleret (the "The Phantom Knights" monster to control and the Xyz material), Break Sword and Malevolent Scythe
// (the DARK Xyz Monsters), and an opponent's monster for the position change.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, GRAVE),
    (83566725, MZONE),
    (83566725, MZONE),
    (62709239, EXTRA),
    (78449284, EXTRA),
    (83566725, DECK),
    (84192580, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

const HINT_TIMING: u32 = TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e0.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e0.set_condition(duel, actcon);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, HINT_TIMING);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, xyztg);
    e2.set_operation(duel, xyzop);
    e2.set_hint_timing(duel, 0, HINT_TIMING);
    c.register_effect(duel, e2);
}

fn actconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_THE_PHANTOM_KNIGHTS]) && c.is_monster(duel) && c.is_faceup(duel)
}

fn actcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        actconfilter,
        a.e.get_handler_player(duel),
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        1,
        None,
    )
}

fn can_summon(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(
        SET_THE_PHANTOM_KNIGHTS,
        TYPE_MONSTER | TYPE_NORMAL,
        0,
        300,
        3,
        RACE_WARRIOR,
        ATTRIBUTE_DARK,
    );
    duel.is_player_can_special_summon_monster_full(tp, ID, stats, POS_FACEUP_DEFENSE, tp, 0)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && a.e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && can_summon(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(a.e.get_handler(duel).into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_POSITION, None, 1, 1 - tp, 0);
    Ok(false)
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attack_pos(duel) && c.is_can_change_position(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) && can_summon(duel, tp) {
        c.add_monster_attribute(duel, TYPE_NORMAL);
        duel.special_summon_step(c, 0, tp, tp, true, false, POS_FACEUP_DEFENSE)?;
        c.add_monster_attribute_complete(duel);
        if duel.special_summon_complete()? > 0
            && duel.is_existing_matching_card(posfilter, tp, 0, LOCATION_MZONE, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
            let posg = duel.select_matching_card(tp, posfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
            if posg.is_empty() {
                return Ok(());
            }
            duel.hint_selection(posg);
            duel.break_effect()?;
            duel.change_position_full(
                posg,
                [POS_FACEUP_DEFENSE, POS_FACEDOWN_DEFENSE, POS_FACEUP_DEFENSE, POS_FACEUP_DEFENSE],
                false,
            )?;
        }
    }
    Ok(())
}

fn xyzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_xyz_summonable(duel, None, None)
}

fn xyztg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(xyzfilter, a.tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn xyzop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, xyzfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.xyz_summon(tp, sc, None, None)?;
    }
    Ok(())
}
