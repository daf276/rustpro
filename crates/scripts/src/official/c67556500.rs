// Phonon Pulse Dragon
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 67556500;

// In the Extra Deck, Synchro Summoned with Water Spirit (Level 1 Tuner) and Swordsman of Landstar (Level 3).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (487395, MZONE),
    (487395, MZONE),
    (3573512, MZONE),
    (3573512, MZONE),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, lvtg);
    e1.set_operation(duel, lvop);
    c.register_effect(duel, e1);
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_LVRANK);
    let level = u64::from(a.e.get_handler(duel).get_level(duel));
    let lv = duel.announce_number(tp, &(1..=3).filter(|&n| n != level).collect::<Vec<_>>())?;
    duel.set_target_param(lv as u32);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut fid = 0;
    let lv: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, lv as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
        fid = c.get_real_field_id(duel);
    }
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_target_range(duel, 1, 0);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_target_spsummon(duel, splimit);
    e2.set_label(duel, i64::from(fid));
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_MUST_BE_MATERIAL);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_target_range(duel, 1, 0);
    e3.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_PHASE_END & !RESET_TURN_SET));
    e3.set_value(duel, REASON_SYNCHRO as i32);
    c.register_effect(duel, e3);
    Ok(())
}

fn splimit(duel: &Duel, e: EffectId, _: CardId, _: u8, sumtype: u32, _: u8, _: u8, _: Option<EffectId>) -> bool {
    let owner = e.get_owner(duel);
    sumtype != SUMMON_TYPE_SYNCHRO || i64::from(owner.get_real_field_id(duel)) != e.get_label(duel) || owner.is_facedown(duel)
}
