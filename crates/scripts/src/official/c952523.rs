// Traptrix Cularia
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};
use engine::constants::MSG_POS_CHANGE;

const ID: u32 = 952523;

pub const LISTED_SERIES: &[u16] = &[SET_HOLE, SET_TRAP_HOLE, SET_TRAPTRIX];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_INSECT | RACE_PLANT, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with two Traptrix Atrax (Insect); Trap Hole (a "Hole" Normal Trap) in hand to set again, and
// Traptrix Atrax in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (55428242, MZONE),
    (55428242, MZONE),
    (55428242, MZONE),
    (55428242, GRAVE),
    (4206964, HAND),
    (4206964, HAND),
];
// The re-Set is reported: its position change is about the Trap, a card no check counts for this one.
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::Message(MSG_POS_CHANGE)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_value(duel, Value::Effect(|duel, _, te| te.is_trap_effect(duel)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVED);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, setcon);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP_DEFENSE>);
    c.register_effect(duel, e3);
}

fn setcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let Some(re) = a.re else { return false };
    if !(a.rp == tp && re.is_trap_effect(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && !duel.has_flag_effect(tp, ID)) {
        return false;
    }
    let rc = re.get_handler(duel);
    rc.is_normal_trap(duel)
        && rc.is_set_card(duel, &[SET_HOLE, SET_TRAP_HOLE])
        && rc.is_can_turn_set(duel)
        && rc.is_relate_to_effect(duel, re)
        && rc.is_location(duel, LOCATION_STZONE)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        if let Some(rc) = a.re.map(|re| re.get_handler(duel)) {
            rc.cancel_to_grave(duel);
            duel.change_position(rc, POS_FACEDOWN)?;
        }
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_TRAPTRIX]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}
