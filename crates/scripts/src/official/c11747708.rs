// Spore
use crate::prelude::*;
use crate::testing::{GRAVE, MZONE};

const ID: u32 = 11747708;

// In the Graveyard with Plant Normal Monsters (Dark Plant) on the field and in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, GRAVE), (13193642, MZONE), (13193642, GRAVE), (13193642, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

// `c:IsInMainZone()` does not exist, so the Lua filter errors and rejects a monster when there is no free zone.
fn costfilter(c: CardId, duel: &Duel, hz: bool) -> bool {
    c.is_race(duel, RACE_PLANT) && c.has_level(duel) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false) && hz
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let hz = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let handler = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| costfilter(c, duel, hz),
            tp,
            LOCATION_MZONE | LOCATION_GRAVE,
            0,
            1,
            Some(handler.into()),
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| costfilter(c, duel, hz),
        tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        1,
        1,
        Some(handler.into()),
    )?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    if let Some(tc) = g.get_first() {
        e.set_label(duel, tc.get_level(duel) as i64);
    }
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 1 {
        let level = e.get_label(duel) as i32;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, level);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
