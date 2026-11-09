// Magical Android
use crate::prelude::*;
use crate::testing::MZONE;

const ID: u32 = 43385557;

pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE)];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_F + EFFECT_TYPE_FIELD);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_target(duel, rectg);
    e1.set_operation(duel, recop);
    c.register_effect(duel, e1);
}

fn psychic_count(duel: &Duel, tp: u8) -> u32 {
    duel.get_matching_group_count(
        aux::faceup_filter(|c, duel| c.is_race(duel, RACE_PSYCHIC)),
        tp,
        LOCATION_MZONE,
        0,
        None,
    ) as u32
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let ct = psychic_count(duel, tp);
    duel.set_target_player(tp);
    duel.set_target_param(ct * 600);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, (ct * 600) as i32);
    Ok(false)
}

fn recop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = psychic_count(duel, tp);
    duel.recover(tp, ct * 600, REASON_EFFECT)?;
    Ok(())
}
