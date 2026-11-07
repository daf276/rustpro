// Sacred Beasts Combined Assault
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 50147815;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_PHANTASM])
    }
}

// Level 10 Sacred Beast monsters in the hand, on the field and in the Graveyard, to Special Summon and to be the materials
// of The Chaotic Phantasmal Sacred Beasts in the Extra Deck, and a copy of the Trap in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, 0),
    (ID, GRAVE),
    (23856331, HAND),
    (50251045, MZONE),
    (96345184, MZONE),
    (96345184, GRAVE),
    (7894706, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DISABLE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SACRED_BEAST]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, true, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    let dg = duel.get_matching_group(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, None);
    duel.set_possible_operation_info(0, CATEGORY_DISABLE, Some(dg), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, Some(dg), 1, tp, 0);
    Ok(false)
}

fn sacredbeastfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SACRED_BEAST]) && c.is_original_level(duel, 10) && c.is_faceup(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty()
        && duel.special_summon(g, 0, tp, tp, false, true, POS_FACEUP_DEFENSE)? > 0
        && duel.is_existing_matching_card(sacredbeastfilter, tp, LOCATION_MZONE, 0, 2, None)
        && duel.is_existing_matching_card(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
        let g = duel.select_matching_card(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
        let Some(sc) = g.get_first() else {
            return Ok(());
        };
        duel.hint_selection(sc);
        if sc.is_can_be_disabled_by_effect(duel, e, true) {
            duel.break_effect()?;
            sc.negate_effects(duel, e.get_handler(duel), RESET_EVENT | RESETS_STANDARD, true);
            duel.adjust_instantly(Some(sc));
            if sc.is_disabled(duel) {
                duel.destroy(sc, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}
