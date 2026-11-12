// Number C1: Numeron Chaos Gate Sunya
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 79747096;
const NUMERON_GATE_EKAM: u32 = 15232745;
pub const XYZ_NUMBER: u32 = 1;
pub const LISTED_NAMES: &[u32] = &[NUMERON_GATE_EKAM, CARD_NUMERON_NETWORK];

// In the Extra Deck, Xyz Summoned on Number 1: Numeron Gate Ekam or with four Level 2 monsters, with Ekam in the Extra Deck for the opponent's
// monsters to be banished and to come back.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (NUMERON_GATE_EKAM, MZONE),
    (NUMERON_GATE_EKAM, EXTRA),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 4;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, rmcon);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_operation(duel, op);
    c.register_effect(duel, e2);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    c.is_faceup(duel) && c.is_summon_code(duel, Some(lc), SUMMON_TYPE_XYZ as u64, tp, &[NUMERON_GATE_EKAM])
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_xyz_summoned(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_field_group(a.tp, LOCATION_MZONE, LOCATION_MZONE);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.tp, LOCATION_MZONE, LOCATION_MZONE);
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let turn_player = duel.is_turn_player(tp);
    let res = if duel.is_phase(PHASE_STANDBY) && turn_player { 2 } else { 1 };
    let turn_asc = if duel.get_current_phase() < PHASE_STANDBY && turn_player {
        0
    } else if turn_player {
        2
    } else {
        1
    };
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_range(duel, LOCATION_REMOVED);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop);
    let turn = duel.get_turn_count();
    e1.set_label(duel, i64::from(turn_asc + turn));
    e1.set_reset_full(
        duel,
        RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN,
        res,
    );
    c.register_effect(duel, e1);
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    i64::from(duel.get_turn_count()) == a.e.get_label(duel) && duel.is_turn_player(a.tp)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let xyz_faceup = aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_XYZ));
    let atk = duel
        .get_matching_group(xyz_faceup, tp, LOCATION_REMOVED, LOCATION_REMOVED, Some(c.into()))
        .get_sum(duel, |c, duel| i64::from(c.get_attack(duel)));
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_NUMERON_NETWORK)),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
        && atk > 0
    {
        duel.break_effect()?;
        duel.damage(1 - tp, atk as u32, REASON_EFFECT)?;
    }
    Ok(())
}
