// Illusion Gate
use crate::prelude::*;
use crate::testing::{GRAVE, MZONE, OPPONENT};

const ID: u32 = 33017964;

// Sangan on the opponent's field to destroy and another in their Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, 0), (26202165, MZONE | OPPONENT), (26202165, GRAVE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, Cost::pay_lp_half);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(!duel.has_flag_effect(tp, ID) && !g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 1 - tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_monster(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.has_flag_effect(tp, ID) {
        return Ok(());
    }
    duel.register_flag_effect(tp, ID, 0, 0, 1);
    let g = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_MZONE, None);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    if !g.is_empty()
        && duel.destroy(g, REASON_EFFECT)? > 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(&filter, tp, 0, LOCATION_GRAVE, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_target(tp, &filter, tp, 0, LOCATION_GRAVE, 1, 1, None)?;
        if let Some(sg) = sg.filter(|sg| !sg.is_empty()) {
            duel.break_effect()?;
            duel.special_summon(sg, 0, tp, tp, true, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
