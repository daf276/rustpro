// Recovery Sorcerer
use super::c49847524::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 76232522;

// In the Extra Deck, Link Summoned from Cyberse Normal Monsters, with a Cyberse Link Monster of ours destroyed that turn by Raigeki
// or Dark Hole in the opponent's hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (24154052, MZONE),
    (32295838, MZONE),
    (ID, MZONE),
    (49847524, MZONE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (47226949, GRAVE),
    (47226949, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, tid: i32) -> bool {
    c.get_turn_id(duel) == tid
        && c.get_reason(duel) & REASON_DESTROY != 0
        && c.is_race(duel, RACE_CYBERSE)
        && c.is_link_monster(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let tid = duel.get_turn_count();
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp, tid));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp, tid), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp, tid), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)? {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect_full(duel, e1, true);
        }
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e3.set_range(duel, LOCATION_MZONE);
        e3.set_code(duel, EVENT_PHASE + PHASE_END);
        e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e3.set_count_limit(duel, 1, 0);
        e3.set_condition_mut(duel, descon);
        e3.set_operation(duel, desop);
        e3.set_label_object_group(duel, Some(tc.into()));
        duel.register_effect(e3, tp);
        tc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    unported!(duel, tc.is_none());
    if tc.is_some_and(|tc| tc.get_flag_effect(duel, ID) == 0) {
        a.e.reset(duel);
        return false;
    }
    true
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
