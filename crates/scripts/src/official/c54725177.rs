// Galaxy-Eyes Photon Change Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 54725177;

pub const LISTED_NAMES: &[u32] = &[CARD_GALAXYEYES_P_DRAGON];

// In the hand and on the field, Special Summoned by Tributing two Blue-Eyes White Dragons (3000 ATK) on either field, with Galaxy-Eyes
// Photon Dragon in the Deck and Graveyard to Special Summon, and the opponent's monsters (Special Summoned from the Extra Deck by
// Polymerization) to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (89631139, MZONE),
    (89631139, MZONE),
    (89631139, MZONE | OPPONENT),
    (CARD_GALAXYEYES_P_DRAGON, DECK),
    (CARD_GALAXYEYES_P_DRAGON, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(CARD_GALAXYEYES_P_DRAGON),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, (ID, 0), EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, selfspcon);
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, gepdsptg);
    e2.set_operation(duel, gepdspop);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
}

fn selfspcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attack_above(duel, 2000) && c.is_faceup(duel) && c.is_releasable(duel)
}

fn rescon(tp: u8) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, _, _| {
        (
            duel.get_mzone_count(tp, Some(sg)) > 0 && sg.is_exists(duel, |c, duel| c.is_controler(duel, tp), 1, None),
            false,
        )
    }
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let g = duel.get_matching_group(selfspcostfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    !g.is_empty() && aux::select_unselect_group_check(duel, g, 2, 2, &rescon(tp))
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(selfspcostfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let sg = aux::select_unselect_group_full(duel, g, 2, 2, &rescon(tp), tp, HINTMSG_RELEASE, None, true)?;
    if sg.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(sg));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.release(g, REASON_COST)?;
    }
    Ok(())
}

fn gepdspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_GALAXYEYES_P_DRAGON) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gepdsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, Some(e.get_handler(duel).into())) > 0
            && duel.is_existing_matching_card(
                |c, duel| gepdspfilter(c, duel, e, tp),
                tp,
                LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn rmvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_summon_location(duel, LOCATION_EXTRA) && c.is_able_to_remove(duel)
}

fn gepdspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| gepdspfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty()
        && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.is_existing_matching_card(rmvfilter, tp, 0, LOCATION_MZONE, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let rg = duel.select_matching_card(tp, rmvfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        if !rg.is_empty() {
            duel.hint_selection(rg);
            duel.break_effect()?;
            aux::remove_until::<aux::DefaultFieldReturnOp>(duel, rg, 0, REASON_EFFECT, PHASE_END, ID, e, tp)?;
        }
    }
    Ok(())
}
