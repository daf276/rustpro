// Rampaging Smashtank Rhynosaber
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 82197831;

// On the field to battle the opponent's weak monsters (Key Mace), with Level 3 and 4 monsters (Battle Warrior, Celtic Guardian)
// in the Graveyard for the Level 7 total.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (55550921, GRAVE),
    (55550921, GRAVE),
    (91152256, GRAVE),
    (91152256, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
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
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, ID);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, atcost);
    e1.set_operation(duel, atop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battled_group_count(duel) > 0);
    e2.set_cost(duel, Cost::self_to_grave);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn atcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    let ct = duel.discard_hand(tp, CardId::is_discardable, 1, 60, REASON_COST | REASON_DISCARD)?;
    a.e.set_label(duel, ct.into());
    Ok(false)
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let ct = a.e.get_label(duel);
    if ct > 0 && c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        e1.set_value(duel, (700 * ct) as i32);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let lv = c.get_level(duel);
    lv > 0 && lv < 7 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if e.get_handler(duel).get_sequence(duel) < 5 {
        ft += 1;
    }
    let mut ct = duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff);
    let g = duel.get_matching_group(|c, duel| spfilter1(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ct = 1;
    }
    if chk == 0 {
        return Ok(ft > 0 && g.check_with_sum_equal(duel, CardId::get_level, 7, 1, ct.max(0) as usize));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut ct = duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff);
    if ct <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ct = 1;
    }
    let g = duel.get_matching_group(|c, duel| spfilter1(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
    if g.check_with_sum_equal(duel, CardId::get_level, 7, 1, ct as usize) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select_with_sum_equal(duel, tp, CardId::get_level, 7, 1, ct)?;
        for tc in sg {
            duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_DISABLE);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_code(duel, EFFECT_DISABLE_EFFECT);
            e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e2);
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}
