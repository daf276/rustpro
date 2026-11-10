// Ice Dragon's Prison
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 20899496;

// In the hand, with Kojikocy (a Normal Monster) in the opponent's Graveyard to Special Summon and others on their field to
// banish together with it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (1184620, GRAVE | OPPONENT),
    (1184620, MZONE | OPPONENT),
    (1184620, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(1184620), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_remove(duel)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (
        sg.get_class_count(duel, |c, duel| c.get_race(duel) as i64) == 1
            && sg.get_class_count(duel, |c, duel| i64::from(c.get_controler(duel))) == sg.len(),
        false,
    )
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
            tc.register_effect(duel, e1);
        }
        duel.special_summon_complete()?;
        let g = duel.get_matching_group(rmfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
        if aux::select_unselect_group_check(duel, g, 2, 2, &rescon) && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.break_effect()?;
            let rg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_REMOVE)?;
            duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
        }
    }
    Ok(())
}
