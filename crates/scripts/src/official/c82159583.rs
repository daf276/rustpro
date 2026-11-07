// Clown Crew Biancaviso
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 82159583;

struct Procedure;

impl NormalProcedure for Procedure {
    const NS: bool = true;
    const OPT: bool = true;
    const MIN: i32 = 1;
    const MAX: i32 = 1;
    const DESC: Option<u64> = Some(aux::stringid(ID, 0));

    fn f(c: CardId, duel: &Duel, tp: u8) -> bool {
        c.is_type(duel, TYPE_RITUAL | TYPE_PENDULUM | TYPE_EXTRA) && (c.is_controler(duel, tp) || c.is_faceup(duel))
    }
}

// In the hand twice and on the field, with two Clown Crew Flair on the field to Tribute Summon it with.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, MZONE), (42759961, MZONE), (42759961, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::TributeSummoned(1),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Normal::add_summon_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_tribute_summoned(duel));
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_SUMMON + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| duel.is_main_phase_full(Some(1 - a.tp)));
    e2.set_target(duel, tribsumtg);
    e2.set_operation(duel, tribsumop);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ct = e.get_handler(duel).get_material_count(duel);
    let b1 = !duel.has_flag_effect(tp, ID) && duel.is_player_can_draw(tp, ct);
    let b2 = !duel.has_flag_effect(tp, ID + 1) && duel.is_existing_matching_card(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, ct, None);
    if chk == 0 {
        return Ok(ct > 0 && (b1 || b2));
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])?;
    e.set_labels(duel, &[op.map_or(0, |op| op as i64), ct as i64]);
    if op == Some(1) {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_DRAW);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct as i32);
    } else if op == Some(2) {
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_DISABLE);
        duel.set_operation_info(0, CATEGORY_DISABLE, None, ct as u8, 1 - tp, LOCATION_ONFIELD as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let &[op, ct] = e.get_labels(duel) else { return Ok(()) };
    let ct = ct as usize;
    if op == 1 {
        duel.draw(tp, ct as u32, REASON_EFFECT)?;
    } else if op == 2 {
        let g = duel.get_matching_group(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, None);
        if g.len() >= ct {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
            let g = g.select(duel, tp, ct as u8, ct as u8, None)?;
            duel.hint_selection(g);
            let c = e.get_handler(duel);
            for sc in g {
                sc.negate_effects(duel, c, 0, true);
            }
        }
    }
    Ok(())
}

fn tribsumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| c.can_summon_or_set(duel, true, None, 1), a.tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, 0, 0);
    Ok(false)
}

fn tribsumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| c.can_summon_or_set(duel, true, None, 1),
        tp,
        LOCATION_HAND,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first() {
        duel.summon_or_set(tp, sc, true, None, 1)?;
    }
    Ok(())
}
