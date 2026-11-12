// Number 87: Queen of the Night
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, SZONE};

const ID: u32 = 89516305;
pub const XYZ_NUMBER: u32 = 87;

// In the Extra Deck, Xyz Summoned with three Level 8 Normal Monsters (Blue-Eyes White Dragon), with a Plant monster (Queen of Autumn
// Leaves) and the opponent's set Spell/Trap Cards to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (89631139, MZONE),
    (89631139, MZONE),
    (89631139, MZONE),
    (89631139, MZONE),
    (4179849, MZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
];

pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_hint_timing(
        duel,
        TIMING_DAMAGE_STEP | TIMING_SSET,
        TIMING_DAMAGE_STEP | TIMING_SSET | TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E,
    );
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_PLANT) && c.is_can_turn_set(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        match e.get_label(duel) {
            1 => return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_SZONE) && chkc.is_facedown(duel)),
            2 => return Ok(chkc.is_location(duel, LOCATION_MZONE) && setfilter(chkc, duel)),
            3 => return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel)),
            _ => {}
        }
    }
    let not_dmg_step = !duel.is_phase(PHASE_DAMAGE);
    let b1 = not_dmg_step && duel.is_existing_target(CardId::is_facedown, tp, 0, LOCATION_SZONE, 1, None);
    let b2 = not_dmg_step && duel.is_existing_target(setfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None);
    let b3 = aux::stat_change_damage_step_condition(duel, a)
        && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None);
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let op = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    e.set_category(duel, 0);
    if op == Some(1) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEDOWN);
        duel.select_target(tp, CardId::is_facedown, tp, 0, LOCATION_SZONE, 1, 1, None)?;
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
        let g = duel.select_target(tp, setfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_POSITION, g, 1, tp, POS_FACEDOWN_DEFENSE as i32);
    } else if op == Some(3) {
        e.set_category(duel, CATEGORY_ATKCHANGE);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
        duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    match e.get_label(duel) {
        1 if c.is_relate_to_effect(duel, e) && tc.is_facedown(duel) => {
            c.set_card_target(duel, tc);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_OWNER_RELATE);
            e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
            e1.set_condition(duel, |duel, a| {
                let (c, tc) = (a.e.get_owner(duel), a.e.get_handler(duel));
                c.is_has_card_target(duel, tc) && tc.is_facedown(duel)
            });
            e1.set_value(duel, 1);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
        2 if tc.is_race(duel, RACE_PLANT) => {
            duel.change_position(tc, POS_FACEDOWN_DEFENSE)?;
        }
        3 if tc.is_faceup(duel) => {
            tc.update_attack(duel, 300, None, Some(c), None);
        }
        _ => {}
    }
    Ok(())
}
