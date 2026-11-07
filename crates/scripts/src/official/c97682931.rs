// Revolution Synchron
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 97682931;

// In the hand, Synchro material for Kuibelt the Blade Dragon with a Level 4 monster, Kojikocy.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (87837090, EXTRA), (1184620, MZONE)];
// EFFECT_SYNCHRO_MAT_FROM_HAND.
pub const CHECKS: &[Check] = &[Check::MaterialFrom(HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_SYNCHRO_MAT_FROM_HAND);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_value(
        duel,
        Value::MaterialFor(|duel, _, _, sc| {
            sc.is_set_card(duel, &[SET_POWER_TOOL]) || (sc.is_race(duel, RACE_DRAGON) && sc.is_level(duel, &[7, 8]))
        }),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DECKDES + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e2.set_condition(duel, |duel, a| {
        let filter = |c: CardId, duel: &Duel| c.is_faceup(duel) && c.is_type(duel, TYPE_SYNCHRO) && c.is_level_above(duel, 7);
        duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None)
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, 1)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, tp, 1);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.discard_deck(tp, 1, REASON_EFFECT)? > 0
        && duel
            .get_operated_group()
            .get_first()
            .is_some_and(|tc| tc.is_location(duel, LOCATION_GRAVE))
    {
        let c = e.get_handler(duel);
        if c.is_relate_to_effect(duel, e) && duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)? {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CHANGE_LEVEL);
            e1.set_value(duel, 1);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            c.register_effect_full(duel, e1, true);
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}
