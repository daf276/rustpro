// The Phantom Knights of Cursed Javelin
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 12219047;

pub const LISTED_SERIES: &[u16] = &[SET_THE_PHANTOM_KNIGHTS];
// In the Extra Deck, Xyz Summoned from Level 2 Normal Monsters (Pharaoh's Servant, Fiend's Hand), with Sangan on the opponent's side to
// negate. No Level 2 Phantom Knight exists, so the Quick Effect that needs one as material shares the same description.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (52550973, MZONE),
    (52550973, MZONE),
    (52800428, MZONE),
    (52800428, MZONE),
    (26202165, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition1);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMINGS_CHECK_MONSTER_E);
    e2.set_condition(duel, condition2);
    c.register_effect(duel, e2);
}

fn has_phantom_knight_material(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_overlay_group(duel)
        .is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_THE_PHANTOM_KNIGHTS]), 1, None)
}

fn condition1(duel: &Duel, a: &Args) -> bool {
    !has_phantom_knight_material(duel, a)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && !(c.get_attack(duel) == 0 && c.is_disabled(duel))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        duel.negate_related_chain(tc, RESET_TURN_SET);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_value(duel, RESET_TURN_SET as i32);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e2);
        let e3 = duel.create_effect(c);
        e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e3.set_value(duel, 0);
        tc.register_effect(duel, e3);
        if tc.is_type(duel, TYPE_TRAPMONSTER) {
            let e4 = duel.create_effect(c);
            e4.set_type(duel, EFFECT_TYPE_SINGLE);
            e4.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
            e4.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e4);
        }
    }
    Ok(())
}

fn condition2(duel: &Duel, a: &Args) -> bool {
    has_phantom_knight_material(duel, a) && aux::stat_change_damage_step_condition(duel, a)
}
