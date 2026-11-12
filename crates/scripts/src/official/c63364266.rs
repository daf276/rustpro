// Goyo Chaser
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 63364266;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// On the field to attack the opponent's monsters, with room to Special Summon the destroyed one.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, Value::Card(val));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_condition(duel, aux::bdogcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_WARRIOR) && c.is_attribute(duel, ATTRIBUTE_EARTH) && c.is_type(duel, TYPE_SYNCHRO)
}

fn val(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    duel.get_matching_group_count(atkfilter, 0, LOCATION_MZONE, LOCATION_MZONE, Some(c.into())) as i32 * 300
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(bc) = e.get_handler(duel).get_battle_target(duel) else {
        return Ok(false);
    };
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && bc.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_target_card(bc);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(bc.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        let atk = tc.get_attack(duel);
        if duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)? {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
            e1.set_code(duel, EFFECT_SET_ATTACK);
            // Lua halves with a float, which the core rounds.
            e1.set_value(duel, (f64::from(atk) / 2.0).round() as i32);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect_full(duel, e1, true);
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}
