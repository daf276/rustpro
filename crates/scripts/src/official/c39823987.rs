// Sun Dragon Inti
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 39823987;

pub const MATERIAL: &[u32] = &[78275321];
pub const LISTED_NAMES: &[u32] = &[78275321, 66818682];

// In the Extra Deck, Synchro Summoned from Fire Ant Ascator (Level 3 Tuner) and Shapesnatch (Level 5), with Moon Dragon Quilla
// in the Graveyard to revive and stronger attackers on the opponent's side to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (78275321, MZONE),
    (78275321, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (66818682, GRAVE),
    (66818682, GRAVE),
    (5559570, MZONE | OPPONENT),
    (5559570, MZONE | OPPONENT),
    (5559570, MZONE | OPPONENT),
    (5559570, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[78275321])
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_condition(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE) && c.is_reason(duel, REASON_BATTLE)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let Some(tc) = a.e.get_handler(duel).get_reason_card(duel) else {
        return Ok(false);
    };
    if tc.is_relate_to_battle(duel) {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(tc.into()), 1, 0, 0);
        let atk = tc.get_attack(duel);
        duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, (atk + 1) / 2);
    }
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = a.e.get_handler(duel).get_reason_card(duel) else {
        return Ok(());
    };
    if !tc.is_relate_to_battle(duel) {
        return Ok(());
    }
    let atk = ((tc.get_attack(duel) + 1) / 2).max(0);
    if duel.destroy(tc, REASON_EFFECT)? != 0 {
        duel.damage(1 - a.tp, atk as u32, REASON_EFFECT)?;
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| duel.get_turn_count() as i64 != a.e.get_label(duel));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
    let turn = duel.get_turn_count();
    e1.set_label(duel, i64::from(turn));
    duel.register_effect(e1, tp);
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, 66818682) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
