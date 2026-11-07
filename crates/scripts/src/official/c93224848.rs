// Phantasmal Martyrs
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 93224848;
const URIA: u32 = 6007213;
const HAMON: u32 = 32491822;

pub const LISTED_NAMES: &[u32] = &[URIA, HAMON, ID + 1];

// Uria and Hamon on the field.
pub const SUPPORT: &[(u32, u16)] = &[(URIA, MZONE), (HAMON, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(ID + 1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && (c.is_code(duel, URIA) || c.is_code(duel, HAMON))
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_existing_matching_card(aux::TRUE, a.tp, LOCATION_HAND, 0, 2, Some(c.into()))
        && duel.is_existing_matching_card(filter, a.tp, LOCATION_ONFIELD, 0, 1, None)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(!duel.is_existing_matching_card(|c, duel| !c.is_able_to_grave_as_cost(duel), tp, LOCATION_HAND, 0, 1, None));
    }
    let g = duel.get_field_group(tp, LOCATION_HAND, 0);
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn can_summon_tokens(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(0, TYPES_TOKEN, 0, 0, 1, RACE_FIEND, ATTRIBUTE_DARK);
    duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.get_location_count(tp, LOCATION_MZONE) > 2
        && duel.is_player_can_special_summon_monster(tp, ID + 1, stats)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(can_summon_tokens(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 3, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 3, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if can_summon_tokens(duel, tp) {
        for _ in 0..3 {
            if let Some(token) = duel.create_token(tp, ID + 1) {
                duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP_ATTACK)?;
            }
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}
