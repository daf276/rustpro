// Kurikara Divincarnate
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 35405755;

// In the hand and on the field, with the opponent's Sanga of the Thunder to activate on our attacks (Elfnote Lucina) so it can be
// Tributed, and a Sangan of the opponent's in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (13597785, MZONE),
    (13597785, MZONE),
    (25955164, MZONE | OPPONENT),
    (25955164, MZONE | OPPONENT),
    (25955164, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(|_, _, _, _, _, _, _| false));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SPSUMMON_PROC);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_condition(duel, spproccon);
    e2.set_operation(duel, spprocop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE | PHASE_END);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_CHAIN_SOLVED);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    let p: u8 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_CONTROLER);
    let tc = re.get_handler(duel);
    if re.is_monster_effect(duel) && tc.is_relate_to_effect(duel, re) && loc == LOCATION_MZONE && p == 1 - duel.get_turn_player() {
        tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    }
    Ok(())
}

fn tfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_flag_effect(duel, ID) > 0
}

fn spproccon(duel: &Duel, a: &Args) -> bool {
    let Some(_) = a.c else { return true };
    let tp = a.e.get_handler_player(duel);
    let rg = duel.get_matching_group(tfilter, 0, LOCATION_MZONE, LOCATION_MZONE, None);
    !rg.is_empty() && rg.filter_count(duel, CardId::is_releasable, None) == rg.len() && duel.get_mzone_count(tp, Some(rg)) > 0
}

fn spprocop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(c) = a.c else { return Ok(()) };
    let g = duel
        .get_matching_group(tfilter, 0, LOCATION_MZONE, LOCATION_MZONE, None)
        .filter(duel, CardId::is_releasable, None);
    duel.release(g, REASON_COST)?;
    let atk = g.len() as i32 * 1500;
    if atk == 0 {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, atk);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD));
    c.register_effect(duel, e1);
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, 1 - tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(
            duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0
                && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.special_summon(tc, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
