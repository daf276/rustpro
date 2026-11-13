// Predaplant Banksiogre
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 22138839;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_PREDATOR];

// On the field and in the hand, destroyed in battle by the opponent's Elfnote Lucinas, which then get Predator Counters to release for the copies in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (ID, HAND),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, hspcon);
    e1.set_target(duel, hsptg);
    e1.set_operation(duel, hspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_COUNTER);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_operation(duel, cop);
    c.register_effect(duel, e2);
}

fn rfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.get_counter(duel, COUNTER_PREDATOR) > 0 && c.is_controler(duel, 1 - tp)
}

fn release_options(c: CardId, tp: u8) -> ReleaseOptions {
    ReleaseOptions {
        check_field: true,
        to_check: Some(c),
        use_oppo: true,
        ..ReleaseOptions::new(tp)
    }
}

fn hspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = a.e.get_handler_player(duel);
    duel.check_release_group_full(tp, |rc, duel| rfilter(rc, duel, tp), 1, release_options(c, tp))
}

fn hsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(c) = a.c else { return Ok(false) };
    let g = duel.select_release_group_full(tp, |rc, duel| rfilter(rc, duel, tp), (1, 1), true, release_options(c, tp))?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn hspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.release(g, REASON_COST)?;
    Ok(())
}

fn cop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, 0, LOCATION_MZONE, None);
    for tc in g {
        tc.add_counter(duel, COUNTER_PREDATOR, 1);
        if tc.get_level(duel) > 1 {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CHANGE_LEVEL);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_counter(duel, COUNTER_PREDATOR) > 0);
            e1.set_value(duel, 1);
            tc.register_effect(duel, e1);
        }
    }
    Ok(())
}
