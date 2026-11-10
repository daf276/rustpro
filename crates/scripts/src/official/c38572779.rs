// Miscellaneousaurus
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 38572779;

// A copy in the hand and one in the Graveyard, with Dinosaur Normal Monsters (Tomozaurus, Mammoth Graveyard) there to banish
// and (Tomozaurus, Trakodon) in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (46457856, GRAVE),
    (40374923, GRAVE),
    (46457856, DECK),
    (46457856, DECK),
    (42348802, DECK),
    (42348802, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, immcost);
    e1.set_operation(duel, immop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, ID);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_cost(duel, |duel, a, _| {
        a.e.set_label(duel, 100);
        Ok(true)
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn immcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_grave_as_cost(duel) && duel.get_flag_effect(tp, ID) == 0);
    }
    duel.send_to_grave(c, REASON_COST)?;
    Ok(false)
}

fn immop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let phase = if duel.is_phase(PHASE_MAIN1) { PHASE_MAIN1 } else { PHASE_MAIN2 };
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_DINOSAUR));
    e1.set_value(duel, Value::Effect(efilter));
    e1.set_reset(duel, RESET_PHASE | phase);
    duel.register_flag_effect(tp, ID, RESET_PHASE | phase, 0, 1);
    duel.register_effect(e1, tp);
    Ok(())
}

fn efilter(duel: &Duel, e: EffectId, re: EffectId) -> bool {
    e.get_owner_player(duel) != re.get_owner_player(duel) && re.is_activated(duel)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DINOSAUR) && c.is_able_to_remove_as_cost(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32) -> bool {
    c.is_race(duel, RACE_DINOSAUR) && c.is_level_below(duel, lv) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        if e.get_label(duel) != 100 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        let cg = duel.get_matching_group(cfilter, tp, LOCATION_GRAVE, 0, None);
        return Ok(Cost::self_banish(duel, a, 0)?
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, cg.len() as u32), tp, LOCATION_DECK, 0, 1, None));
    }
    let cg = duel.get_matching_group(cfilter, tp, LOCATION_GRAVE, 0, None);
    let tg = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp, cg.len() as u32), tp, LOCATION_DECK, 0, None);
    let mut levels: Vec<u64> = tg
        .iter()
        .map(|tc| u64::from(tc.get_level(duel)))
        .filter(|lv| (1..=12).contains(lv))
        .collect();
    levels.sort_unstable();
    levels.dedup();
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let lv = duel.announce_number(tp, &levels)?;
    let mut rg1 = CardSet::default();
    if lv > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        rg1.merge(cg.select(duel, tp, (lv - 1) as u8, (lv - 1) as u8, Some(c.into()))?);
    }
    rg1.insert(c);
    duel.remove(rg1, POS_FACEUP, REASON_COST)?;
    e.set_label(duel, lv as i64);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn sfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32) -> bool {
    c.is_race(duel, RACE_DINOSAUR) && c.get_level(duel) == lv && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let lv = e.get_label(duel) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| sfilter(c, duel, e, tp, lv), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? != 0
    {
        let fid = i64::from(e.get_handler(duel).get_field_id(duel));
        tc.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1, fid, 0);
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_count_limit(duel, 1, 0);
        e1.set_label(duel, fid);
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_condition_mut(duel, descon);
        e1.set_operation(duel, desop);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    if tc.is_some_and(|tc| tc.get_flag_effect_label(duel, ID) == Some(a.e.get_label(duel))) {
        return true;
    }
    a.e.reset(duel);
    false
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
