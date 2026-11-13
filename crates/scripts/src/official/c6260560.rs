// Laevateinn the Burning Blade
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 6260560;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_FIRE, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned from FIRE Pyros, with lower-Level Pyros in the Graveyard and Trap Holes set for the opponent's
// Mystical Space Typhoon to hit.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (5464695, MZONE),
    (5464695, MZONE),
    (71407486, MZONE),
    (71407486, MZONE),
    (13179332, GRAVE),
    (33112041, GRAVE),
    (4206964, SZONE),
    (4206964, SZONE),
    (5318639, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Replaced];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP_DEFENSE>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, reptg);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e2.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        duel.remove(c, POS_FACEUP, REASON_EFFECT | REASON_REPLACE).map(drop)
    });
    c.register_effect(duel, e2);
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE)
        && c.has_level(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.is_existing_target(
            |sc, duel| spfilter(sc, duel, e, tp, c.get_original_race(duel), c.get_original_level(duel)),
            tp,
            LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, race: u64, lv: u32) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE)
        && c.has_level(duel)
        && c.is_original_race(duel, race)
        && c.get_original_level(duel) < lv
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, e, tp) = (a.e.get_handler(duel), a.e, a.tp);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |sc, duel| spcostfilter(sc, duel, e, tp), (1, 1), false, None, Some(c.into())));
    }
    let g = duel.select_release_group_cost(tp, |sc, duel| spcostfilter(sc, duel, e, tp), (1, 1), false, None, Some(c.into()))?;
    if let Some(sc) = g.get_first() {
        e.set_labels(duel, &[sc.get_original_race(duel) as i64, i64::from(sc.get_original_level(duel))]);
        duel.release(sc, REASON_COST)?;
    }
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let &[race, lv] = e.get_labels(duel) else { return Ok(false) };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(
        tp,
        |c, duel| spfilter(c, duel, e, tp, race as u64, lv as u32),
        tp,
        LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_facedown(duel)
        && c.is_location(duel, LOCATION_STZONE)
        && c.is_controler(duel, tp)
        && c.is_reason_player(duel, 1 - tp)
        && c.is_reason(duel, REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_status(duel, STATUS_DESTROY_CONFIRMED)
            && c.is_able_to_remove(duel)
            && a.eg
                .unwrap_or_default()
                .is_exists(duel, |sc, duel| repfilter(sc, duel, tp), 1, None));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}
