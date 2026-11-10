// The Undying Legion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 43355214;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Xyz Summoned from two Level 7 monsters (Dark Magician), with Dark Magician on the opponent's field to attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (36996508, MZONE),
    (36996508, MZONE),
    (46986414, MZONE),
    (46986414, MZONE),
    (36996508, MZONE | OPPONENT),
    (46986414, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
    const MAXCT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        !duel.has_flag_effect(tp, ID)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.is_main_phase_full(Some(1 - a.tp)));
    e1.set_cost(duel, attachcost);
    e1.set_target(duel, attachtg);
    e1.set_operation(duel, attachop);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e1);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    c.is_rank(duel, &[6])
        && c.is_race_full(duel, RACE_ZOMBIE, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
        && c.is_xyz_monster(duel)
        && c.is_faceup(duel)
}

fn attachcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let g = a.e.get_handler(duel).get_overlay_group(duel);
    let min = if g.len() == g.filter_count(duel, CardId::is_xyz_monster, None) {
        1
    } else {
        2
    };
    Cost::detach_from_self_with(duel, a, chk, min, 2, |_, _, _| {})
}

fn attachfilter(c: CardId, duel: &Duel, xyzc: CardId, tp: u8) -> bool {
    c.is_attack_pos(duel) && c.is_monster(duel) && c.is_can_be_xyz_material(duel, Some(xyzc), tp, REASON_EFFECT)
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp)
            && chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE)
            && attachfilter(chkc, duel, c, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(
            |c2, duel| attachfilter(c2, duel, c, tp),
            tp,
            0,
            LOCATION_MZONE | LOCATION_GRAVE,
            1,
            None,
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(
        tp,
        |c2, duel| attachfilter(c2, duel, c, tp),
        tp,
        0,
        LOCATION_MZONE | LOCATION_GRAVE,
        1,
        1,
        None,
    )?;
    if c.is_location(duel, LOCATION_GRAVE) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, tp, 0);
    }
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, e) && !tc.is_immune_to_effect(duel, e) {
        duel.overlay_full(c, tc, true)?;
    }
    Ok(())
}
