// Dark Ceruvim Irisfil
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 64626565;

pub const LISTED_NAMES: &[u32] = &[ID];
// In the Extra Deck, Xyz Summoned on Level 8 Normal Monsters or Norito; the opponent's Raigeki and Dark Hole test the replacement.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (47484352, MZONE),
    (10949074, MZONE),
    (14152862, MZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        !duel.has_flag_effect(tp, ID)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_CHAIN_SOLVED);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, reptg);
    c.register_effect(duel, e2);
}

fn ovfilter(c: CardId, duel: &Duel, _: u8, _: CardId) -> bool {
    c.is_xyz_monster(duel) && !c.has_flag_effect(duel, ID) && c.is_faceup(duel)
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if re.is_monster_effect(duel)
        && rc.is_xyz_monster(duel)
        && rc.is_relate_to_effect(duel, re)
        && duel.get_chain_info::<u32>(a.ev as u8, CHAININFO_TRIGGERING_LOCATION) == LOCATION_MZONE
    {
        rc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    }
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_target_card(duel, |duel, _, c| c.is_xyz_monster(duel));
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_condition(duel, |duel, _| duel.is_battle_phase());
    e1.set_value(duel, Value::Card(|duel, _, c| c.get_rank(duel) as i32 * 100));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
            && !c.is_reason(duel, REASON_REPLACE)
            && duel.check_remove_overlay_card(tp, 1, 0, 1, REASON_EFFECT));
    }
    Ok(duel.select_effect_yes_no_full(tp, c, 96)? && duel.remove_overlay_card(tp, 1, 0, 1, 1, REASON_EFFECT)? > 0)
}
