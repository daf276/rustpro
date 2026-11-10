// Code of Soul
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 74652966;
const SALAMANGREAT_RAGING_PHOENIX: u32 = 57134592;
const LINK_APPLE: u32 = 7925734;

// In the hand, on the field and in the Graveyard with Salamangreat Raging Phoenix on the field and in the Extra Deck, for the
// Reincarnation Link Summon and with three Link Apples for the Link Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (ID, GRAVE),
    (SALAMANGREAT_RAGING_PHOENIX, MZONE),
    (SALAMANGREAT_RAGING_PHOENIX, EXTRA),
    (SALAMANGREAT_RAGING_PHOENIX, EXTRA),
    (LINK_APPLE, MZONE),
    (LINK_APPLE, MZONE),
    (LINK_APPLE, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| {
        let filter = aux::faceup_filter(CardId::is_link_monster);
        duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None)
    });
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp) && duel.is_main_phase());
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, linktg);
    e3.set_operation(duel, linkop);
    c.register_effect(duel, e3);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_EXTRA);
    e1.set_condition(duel, reinclinkcon);
    e1.set_target(duel, reinclinktg);
    e1.set_operation(duel, reinclinkop);
    e1.set_value(duel, SUMMON_TYPE_LINK as i32);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_GRANT);
    e2.set_target_range(duel, LOCATION_EXTRA, 0);
    e2.set_target_card(duel, |duel, _, c| {
        c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_link_monster(duel)
    });
    e2.set_label_object(duel, Some(e1));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 4)));
    Ok(())
}

fn reincmatfilter(c: CardId, duel: &Duel, lc: CardId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_link_monster(duel)
        && c.is_summon_code(duel, Some(lc), SUMMON_TYPE_LINK.into(), tp, &[lc.get_code(duel)])
        && c.is_can_be_link_material(duel, Some(lc), Some(tp))
        && duel.get_location_count_from_ex(tp, tp, Some(c.into()), lc) > 0
}

fn reinc_materials(duel: &Duel, a: &Args, c: CardId, tp: u8) -> (CardSet, CardSet) {
    let g = duel.get_matching_group(|mc, duel| reincmatfilter(mc, duel, c, tp), tp, LOCATION_MZONE, 0, None);
    let mut mustg = aux::get_must_be_material_group(duel, tp, Some(g), tp, Some(c), Some(g), REASON_LINK);
    if let Some(must) = a.must {
        mustg = mustg.union(must);
    }
    (g, mustg)
}

fn reinclinkcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let (g, mustg) = reinc_materials(duel, a, c, tp);
    let valid = match mustg.len() {
        0 => !g.is_empty(),
        1 => mustg.get_first().is_some_and(|mc| reincmatfilter(mc, duel, c, tp)),
        _ => false,
    };
    valid && !duel.has_flag_effect(tp, ID)
}

fn reinclinktg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(c) = a.c else { return Ok(false) };
    let (g, mustg) = reinc_materials(duel, a, c, tp);
    if !mustg.is_empty() {
        if mustg.len() > 1 {
            return Ok(false);
        }
        e.set_label_object_group(duel, Some(mustg));
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_LMATERIAL);
    let Some(tc) = g.select_unselect(duel, Some(CardSet::default()), tp, false, true, 1, 1)? else {
        return Ok(false);
    };
    e.set_label_object_group(duel, Some(tc.into()));
    Ok(true)
}

fn reinclinkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_CARD, 0, ID.into());
    let (Some(c), Some(mg)) = (a.c, e.get_label_object_group(duel)) else {
        return Ok(());
    };
    c.set_material(duel, Some(mg));
    duel.send_to_grave(mg, REASON_MATERIAL | REASON_LINK)?;
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    Ok(())
}

fn linkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_CYBERSE) && c.is_link_above(duel, 3) && c.is_link_summonable(duel, None, None)
}

fn linktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(linkfilter, tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn linkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, linkfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.link_summon(tp, sc, None, None)?;
    }
    Ok(())
}
