// Kewl Tune Clip
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 43904702;

// Kewl Tune Mix on the field and in the hand and Track Maker in the Extra Deck to Synchro Summon with this card as material.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (16509007, MZONE),
    (16509007, HAND),
    (42781164, EXTRA),
    (42781164, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct TunerFromHand;

impl HandMaterial for TunerFromHand {
    const ID: u32 = ID;

    fn material_filter(c: CardId, duel: &Duel) -> bool {
        c.is_type(duel, TYPE_TUNER)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_hand_material_effect::<TunerFromHand>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp) && duel.is_main_phase());
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO
    });
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn scfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TUNER) && c.is_synchro_summonable(duel, None, None)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(scfilter, tp, LOCATION_EXTRA, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
            duel.synchro_summon(tp, sc, None, None)?;
        }
    }
    Ok(())
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_able_to_remove(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(rmfilter, tp, 0, LOCATION_EXTRA, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(rmfilter, tp, 0, LOCATION_EXTRA, None);
    if g.is_empty() {
        return Ok(());
    }
    let rg = g.random_select(duel, tp, 1);
    if !rg.is_empty() {
        duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
