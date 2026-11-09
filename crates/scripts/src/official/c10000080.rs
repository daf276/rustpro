// The Winged Dragon of Ra - Sphere Mode
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 10000080;

pub const LISTED_NAMES: &[u32] = &[CARD_RA];

// In the hand to be Summoned with three of the opponent's Prevent Rats as Tributes, and The Winged Dragon of Ra in the
// Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (549481, MZONE | OPPONENT),
    (549481, MZONE | OPPONENT),
    (549481, MZONE | OPPONENT),
    (549481, MZONE | OPPONENT),
    (CARD_RA, DECK),
    (CARD_RA, DECK),
];
pub const CHECKS: &[Check] = &[Check::TributeSummoned(3), Check::ResolvesEffect(aux::stringid(ID, 2))];

struct SummonProcedure;

impl NormalProcedure for SummonProcedure {
    const NS: bool = true;
    const OPT: bool = false;
    const MIN: i32 = 3;
    const MAX: i32 = 3;
    const VAL: u32 = SUMMON_TYPE_TRIBUTE;
    const DESC: Option<u64> = Some(aux::stringid(ID, 0));
}

struct SetProcedure;

impl NormalProcedure for SetProcedure {
    const NS: bool = false;
    const OPT: bool = false;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Normal::add_summon_procedure::<SummonProcedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SPSUM_PARAM);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_LIMIT_SUMMON_PROC);
    e2.set_target_range(duel, POS_FACEUP_ATTACK, 1);
    e2.set_condition(duel, ttcon2);
    e2.set_target(duel, tttg2);
    e2.set_operation(duel, ttop2);
    e2.set_value(duel, SUMMON_TYPE_TRIBUTE as i32);
    c.register_effect(duel, e2);
    Normal::add_set_procedure::<SetProcedure>(duel, c);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e5.set_code(duel, EVENT_SUMMON_SUCCESS);
    e5.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e5.set_operation(duel, retreg);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_SINGLE);
    e6.set_code(duel, EFFECT_CANNOT_ATTACK);
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_type(duel, EFFECT_TYPE_SINGLE);
    e7.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e7.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e7.set_range(duel, LOCATION_MZONE);
    e7.set_value(duel, Value::Card(aux::imval2));
    c.register_effect(duel, e7);
    let e8 = e7.clone_effect(duel);
    e8.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e8.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e8);
    let e9 = duel.create_effect(c);
    e9.set_description(duel, aux::stringid(ID, 2));
    e9.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e9.set_type(duel, EFFECT_TYPE_IGNITION);
    e9.set_range(duel, LOCATION_MZONE);
    e9.set_cost(duel, Cost::self_tribute);
    e9.set_target(duel, sptg);
    e9.set_operation(duel, spop);
    c.register_effect(duel, e9);
}

fn opponent_tributes(duel: &Duel, relzone: u32, tp: u8) -> CardSet {
    duel.get_matching_group(|mc, duel| aux::is_zone(mc, duel, relzone, tp), tp, 0, LOCATION_MZONE, None)
}

fn ttcon2(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    if a.min > 3 {
        return false;
    }
    let tp = c.get_controler(duel);
    let mg = opponent_tributes(duel, a.releasable, tp);
    duel.check_tribute(c, 3, 3, Some(mg), 1 - tp, a.zone)
}

fn tttg2(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, Some(c)) = (a.tp, a.c) else { return Ok(false) };
    let mg = opponent_tributes(duel, a.releasable, tp);
    let g = duel.select_tribute(tp, c, 3, 3, Some(mg), 1 - tp, a.zone, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn ttop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(c) = a.c else { return Ok(()) };
    let g = a.e.get_label_object_group(duel).unwrap_or_default();
    c.set_material(duel, Some(g));
    duel.release(g, REASON_SUMMON + REASON_MATERIAL)?;
    Ok(())
}

fn retreg(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    c.register_flag_effect(duel, ID, RESET_EVENT | (RESETS_STANDARD_PHASE_END & !RESET_TURN_SET), 0, 2);
    let turn = i64::from(duel.get_turn_count() + 1);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_label(duel, turn);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| {
        i64::from(duel.get_turn_count()) == a.e.get_label(duel) && a.e.get_owner(duel).has_flag_effect(duel, ID)
    });
    e1.set_operation(duel, retop);
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_owner(duel);
    c.reset_effect(duel, EFFECT_SET_CONTROL, RESET_CODE);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_CONTROL);
    e1.set_value(duel, i32::from(c.get_owner(duel)));
    e1.set_reset(
        duel,
        RESET_EVENT | (RESETS_STANDARD & !(RESET_TOFIELD | RESET_TEMP_REMOVE | RESET_TURN_SET)),
    );
    c.register_effect(duel, e1);
    Ok(())
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_RA) && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if e.get_handler(duel).get_sequence(duel) < 5 {
        ft += 1;
    }
    if chk == 0 {
        return Ok(
            ft > 0 && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| filter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if let Some(tc) = g.get_first()
        && duel.special_summon_step(tc, 0, tp, tp, true, false, POS_FACEUP)?
    {
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK);
        e1.set_value(duel, 4000);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_SET_DEFENSE);
        tc.register_effect(duel, e2);
    }
    duel.special_summon_complete()?;
    Ok(())
}
