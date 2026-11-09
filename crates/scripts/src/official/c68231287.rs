// Jupiter the Power Patron of Destruction
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 68231287;

pub const LISTED_NAMES: &[u32] = &[ID];

// Power Patron DoomZ and Junora the Power Patron of Tuning on the field to Xyz Summon it, Axe of Despair and another
// DoomZ in the GY to equip and Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (81650695, MZONE),
    (81650695, GRAVE),
    (5914858, MZONE),
    (40619825, GRAVE),
    (40619825, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(10);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(|c, duel, _, _| c.get_equip_count(duel) >= 3);
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
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_type(duel, TYPE_XYZ), 1, None)
    });
    e2.set_value(duel, 3000);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn eqfilter(c: CardId, duel: &Duel, tp: u8, ec: CardId) -> bool {
    c.is_equip_spell(duel) && c.check_equip_target(duel, ec) && c.check_unique_on_field(duel, tp)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let ec = a.e.get_handler(duel);
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|c, duel| eqfilter(c, duel, tp, ec), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, 0, LOCATION_GRAVE as i32);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let ft = duel.get_location_count(tp, LOCATION_SZONE);
    if ft == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(
        aux::necro_valley_filter(|tc, duel| eqfilter(tc, duel, tp, c)),
        tp,
        LOCATION_GRAVE,
        0,
        None,
    );
    if g.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let sg = g.select(duel, tp, 1, (ft as usize).min(g.len()) as u8, None)?;
    if !sg.is_empty() {
        for tc in sg {
            duel.equip_full(tp, tc, c, true, true)?;
        }
        duel.equip_complete()?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DOOMZ]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty()
        && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.get_field_group_count(tp, LOCATION_ONFIELD, LOCATION_ONFIELD) > 0
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let dg = duel.select_matching_card(tp, |_, _| true, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
        if !dg.is_empty() {
            duel.hint_selection(dg);
            duel.break_effect()?;
            duel.destroy(dg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
