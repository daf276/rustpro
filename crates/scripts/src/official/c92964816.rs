// Glow-Up Bloom
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 92964816;
pub const LISTED_NAMES: &[u32] = &[4064256];

// On the field and destroyed by Dark Hole, with Zombie World to Special Summon Level 5 Zombies (Shadow Ghoul) instead of adding.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (53129443, HAND),
    (4064256, HAND),
    (30778711, DECK),
    (30778711, DECK),
    (30778711, DECK),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(30778711)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_banish);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, chk: bool) -> bool {
    c.is_level_above(duel, 5)
        && c.is_race(duel, RACE_ZOMBIE)
        && (c.is_able_to_hand(duel)
            || (chk && duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn fieldcond(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, 4064256)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let ck = duel.is_existing_matching_card(fieldcond, tp, LOCATION_FZONE, LOCATION_FZONE, 1, None);
        return Ok(duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp, ck), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_race(duel, RACE_ZOMBIE));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    let ck = duel.is_existing_matching_card(fieldcond, tp, LOCATION_FZONE, LOCATION_FZONE, 1, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp, ck), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        if ck
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && tc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
            && (!tc.is_able_to_hand(duel) || duel.select_yes_no(tp, aux::stringid(ID, 1))?)
        {
            duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        } else {
            duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, tc)?;
        }
    }
    Ok(())
}
