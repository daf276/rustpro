// Temtempo the Percussion Djinn
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 52558805;

// In the Extra Deck, Xyz Summoned with two Level 3 monsters (Necrolancer the Time-lord, Rock Ogre Grotto #1), and a copy in
// the opponent's Extra Deck and monsters, whose Xyz Material it detaches.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA | OPPONENT),
    (ID, EXTRA | OPPONENT),
    (61454890, MZONE),
    (61454890, MZONE),
    (68846917, MZONE),
    (68846917, MZONE),
    (61454890, MZONE | OPPONENT),
    (61454890, MZONE | OPPONENT),
    (68846917, MZONE | OPPONENT),
    (68846917, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(
        duel,
        TIMING_DAMAGE_STEP | TIMINGS_CHECK_MONSTER,
        TIMING_DAMAGE_STEP | TIMINGS_CHECK_MONSTER,
    );
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT)
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && c.is_set_card(duel, &[SET_DJINN])
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| filter(c, duel, tp), tp, 0, LOCATION_MZONE, 1, None)
                && duel.is_existing_matching_card(atkfilter, tp, LOCATION_MZONE, 0, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DEATTACHFROM);
    duel.select_target(tp, |c, duel| filter(c, duel, tp), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || tc.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(atkfilter, tp, LOCATION_MZONE, 0, None);
    for tc in g.iter() {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_value(duel, 500);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
