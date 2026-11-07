// Divine Arsenal AA-ZEUS - Sky Thunder
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 90448279;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(12);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> =
        Some(|c, duel, tp, xyzc| c.is_faceup(duel) && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), u64::from(SUMMON_TYPE_XYZ), tp));
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        duel.get_flag_effect(tp, ID) == 0 && duel.get_flag_effect(0, ID + 1) > 0
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        Ok(true)
    }
}

// On the field, where random Xyz materials are attached to it for the cost, and in the Extra Deck to be Xyz Summoned on Tornado Dragon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, EXTRA), (6983839, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_END_PHASE);
    e1.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e1.set_target(duel, gytg);
    e1.set_operation(duel, gyop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, attcon);
    e2.set_target(duel, atttg);
    e2.set_operation(duel, attop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_BATTLED);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn checkop(duel: &mut Duel, _: &Args) -> Script<()> {
    let is_xyz = |c: Option<CardId>, duel: &Duel| c.is_some_and(|c| c.is_type(duel, TYPE_XYZ));
    if is_xyz(duel.get_attacker(), duel) || is_xyz(duel.get_attack_target(), duel) {
        duel.register_flag_effect(0, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
    }
    Ok(())
}

fn gytg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let g = duel.get_matching_group(
        |c, duel| c.is_able_to_grave(duel),
        a.tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        Some(a.e.get_handler(duel).into()),
    );
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn gyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(
        |c, duel| c.is_able_to_grave(duel),
        a.tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        Some(a.e.get_handler(duel).into()),
    );
    duel.send_to_grave(g, REASON_EFFECT)?;
    Ok(())
}

fn attconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    (c.is_reason(duel, REASON_BATTLE) || (c.is_reason(duel, REASON_EFFECT) && c.get_reason_player(duel) == 1 - tp))
        && c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_ONFIELD)
}

fn attcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| attconfilter(c, duel, a.tp), 1, None)
}

fn attfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    !c.is_immune_to_effect(duel, e)
}

fn atttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let e = a.e;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| attfilter(c, duel, e),
            a.tp,
            LOCATION_HAND | LOCATION_DECK | LOCATION_EXTRA,
            0,
            1,
            None,
        ) && e.get_handler(duel).is_type(duel, TYPE_XYZ));
    }
    Ok(false)
}

fn attop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let g = duel.select_matching_card(
        tp,
        |c, duel| attfilter(c, duel, e),
        tp,
        LOCATION_HAND | LOCATION_DECK | LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if let Some(tc) = g.get_first() {
        duel.overlay_full(c, tc, true)?;
    }
    Ok(())
}
