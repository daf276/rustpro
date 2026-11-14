// Dragunity Darkspear
use crate::prelude::*;
use crate::testing::{GRAVE, MZONE};

const ID: u32 = 13361027;

pub const LISTED_SERIES: &[u16] = &[SET_DRAGUNITY];
// Copies on the field to release, and Harpie Girl (Level 2 Winged Beast) in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (34100324, GRAVE), (34100324, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DRAGUNITY]) && c.is_race(duel, RACE_DRAGON)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        return Ok(ft > -1 && duel.check_release_group_cost(tp, cfilter, (1, 1), false, Some(&aux::release_check_mmz), None));
    }
    let rg = duel.select_release_group_cost(tp, cfilter, (1, 1), false, Some(&aux::release_check_mmz), None)?;
    duel.release(rg, REASON_COST)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, sp: u8) -> bool {
    c.is_level_below(duel, 4) && c.is_race(duel, RACE_WINGEDBEAST) && c.is_can_be_special_summoned(duel, e, 0, sp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
