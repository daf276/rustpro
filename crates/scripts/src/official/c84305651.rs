// Goyo King
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 84305651;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp))
    }
}

// Three on the field, with weak monsters (Key Mace) on the opponent's side to destroy in battle and take or Special Summon.
// s.synchro_nt_required, which only cards that are not ported read, is left out.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_condition(duel, |duel, _| duel.get_attack_target().is_some());
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_condition(duel, aux::bdogcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let ct = duel.get_matching_group_count(filter, tp, LOCATION_MZONE, 0, None);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, ct as i32 * 400);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE | RESET_PHASE | PHASE_DAMAGE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_WARRIOR) && c.is_attribute(duel, ATTRIBUTE_EARTH) && c.is_type(duel, TYPE_SYNCHRO)
}

fn ctfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_controler_can_be_changed(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let bc = e.get_handler(duel).get_battle_target(duel);
    let space = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    // The Lua errors on a nil battle target.
    unported!(duel, space && bc.is_none());
    let b1 = space && bc.is_some_and(|bc| bc.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    let b2 = duel.is_existing_matching_card(ctfilter, tp, 0, LOCATION_MZONE, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = if b1 && b2 {
        duel.select_option(tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2)])?
    } else if b1 {
        duel.select_option(tp, &[aux::stringid(ID, 1)])?
    } else {
        duel.select_option(tp, &[aux::stringid(ID, 2)])? + 1
    };
    if op == 0 {
        let Some(bc) = bc else { return Ok(false) };
        duel.set_target_card(bc);
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(bc.into()), 1, 0, 0);
    } else {
        e.set_category(duel, CATEGORY_CONTROL);
    }
    e.set_label(duel, i64::from(op));
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if e.get_label(duel) == 0 {
        if let Some(tc) = duel.get_first_target()
            && tc.is_relate_to_effect(duel, e)
        {
            duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
        let g = duel.select_matching_card(tp, ctfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        if let Some(tc) = g.get_first() {
            duel.get_control(tc, tp)?;
        }
    }
    Ok(())
}
