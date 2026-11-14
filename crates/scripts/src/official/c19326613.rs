// Once Upon a Fairy Tail
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, REMOVED};

const ID: u32 = 19326613;

pub const LISTED_SERIES: &[u16] = &[SET_FAIRY_TAIL];
// Fairy Tail - Luna in the hand, Matchgiru in the Graveyard and Snow in banishment to Special Summon, and a copy in the Graveyard
// and banishment to Set, with Luna on the field to activate it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, REMOVED),
    (86937530, HAND),
    (19144622, GRAVE),
    (55623480, REMOVED),
    (86937530, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_SET);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_REMOVE);
    e2a.set_count_limit(duel, 1, (ID, 1));
    e2a.set_target(duel, settg);
    e2a.set_operation(duel, setop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_TO_GRAVE);
    e2b.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    c.register_effect(duel, e2b);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.is_set_card(duel, &[SET_FAIRY_TAIL])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
}

fn spcheck(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let ct1 = sg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
    let ct2 = sg.get_class_count(duel, |c, duel| i64::from(c.get_location(duel)));
    let ct3 = sg.len();
    (ct1 == ct3 && ct2 == ct3, ct1 != ct3 || ct2 != ct3)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let locations = LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, locations, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, locations as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft > 0 {
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            ft = 1;
        }
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
        let g = duel.get_matching_group(filter, tp, LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED, 0, None);
        if !g.is_empty() {
            ft = ft.min(3);
            let sg = aux::select_unselect_group(duel, g, 1, ft as usize, &spcheck, tp, HINTMSG_SPSUMMON)?;
            if !sg.is_empty() {
                duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    super::c19144622::register_spellcaster_only(duel, e.get_handler(duel), tp, aux::stringid(ID, 2));
    Ok(())
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_ssetable(duel));
    }
    if c.is_location(duel, LOCATION_GRAVE) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, a.tp, 0);
    }
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.sset(tp, c)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 3));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
        e1.set_label(duel, i64::from(tp));
        e1.set_condition(duel, |duel, a| {
            let tp = a.e.get_label(duel) as u8;
            !duel.is_existing_matching_card(
                aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_FAIRY_TAIL])),
                tp,
                LOCATION_MZONE,
                0,
                1,
                None,
            )
        });
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    Ok(())
}
