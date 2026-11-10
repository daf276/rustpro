// Rescue Cat
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 14878871;

// On the field with Level 3 or lower Beasts (Silver Fang, Dark Gray) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (90357090, DECK), (90357090, DECK), (9159938, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 3) && c.is_race(duel, RACE_BEAST) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let handler = Some(e.get_handler(duel).into());
        return Ok(duel.get_mzone_count(tp, handler) > 1
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 2, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) < 2 || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 2, 2, None)?;
    if sg.len() != 2 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let fid = c.get_field_id(duel);
    for tc in sg {
        duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        tc.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1, fid.into(), 0);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_value(duel, RESET_TURN_SET as i32);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e2);
    }
    duel.special_summon_complete()?;
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_count_limit(duel, 1, 0);
    e3.set_label(duel, fid.into());
    e3.set_label_object_group(duel, Some(sg));
    e3.set_condition_mut(duel, descon);
    e3.set_operation(duel, desop);
    duel.register_effect(e3, tp);
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel, fid: i64) -> bool {
    c.get_flag_effect_label(duel, ID) == Some(fid)
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let fid = a.e.get_label(duel);
    let g = a.e.get_label_object_group(duel).unwrap_or_default();
    if !g.is_exists(duel, |c, duel| desfilter(c, duel, fid), 1, None) {
        a.e.reset(duel);
        return false;
    }
    true
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let fid = a.e.get_label(duel);
    let g =
        a.e.get_label_object_group(duel)
            .unwrap_or_default()
            .filter(duel, |c, duel| desfilter(c, duel, fid), None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
