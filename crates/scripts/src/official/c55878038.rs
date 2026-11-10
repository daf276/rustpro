// Chaos Dragon Levianeer
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, OPPONENT};

const ID: u32 = 55878038;

// In the hand, with LIGHT (Mokey Mokey, Key Mace) and DARK (Summoned Skull) monsters in the Graveyard to banish and the
// opponent's hand and field to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (27288416, GRAVE),
    (27288416, GRAVE),
    (1929294, GRAVE),
    (70781052, GRAVE),
    (70781052, GRAVE),
    (14851496, GRAVE),
    (26976414, HAND | OPPONENT),
    (14851496, HAND | OPPONENT),
];

pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_value(duel, 1);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_summon_type(duel, &[SUMMON_TYPE_SPECIAL + 1])
    });
    e2.set_cost(duel, cost);
    e2.set_target(duel, target);
    c.register_effect(duel, e2);
    e1.set_label_object(duel, Some(e2));
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT + ATTRIBUTE_DARK) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    let rg = duel.get_matching_group(spfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    duel.get_location_count(tp, LOCATION_MZONE) > -3
        && rg.len() > 2
        && aux::select_unselect_group_check(duel, rg, 3, 3, &aux::chkf_mmz(tp, 1))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(spfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    let g = aux::select_unselect_group_full(duel, rg, 3, 3, &aux::chkf_mmz(tp, 1), tp, HINTMSG_REMOVE, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    let ids: Vec<i64> = g.iter().map(|card| i64::from(card.get_card_id(duel))).collect();
    e.set_labels(duel, &ids);
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let mut g = CardSet::default();
    for &id in a.e.get_labels(duel) {
        let Some(card) = u32::try_from(id).ok().and_then(|id| duel.get_card_from_card_id(id)) else {
            return Ok(());
        };
        g.insert(card);
    }
    let lt = g.filter_count(duel, |c, duel| c.is_attribute(duel, ATTRIBUTE_LIGHT), None);
    let dt = g.filter_count(duel, |c, duel| c.is_attribute(duel, ATTRIBUTE_DARK), None);
    let lbl = match (lt, dt) {
        (_, 0) => ATTRIBUTE_LIGHT,
        (0, _) => ATTRIBUTE_DARK,
        _ => 1,
    };
    if let Some(e2) = a.e.get_label_object(duel) {
        e2.set_label(duel, i64::from(lbl));
    }
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(())
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    Ok(chk != 0 || a.e.get_handler(duel).get_attack_announced_count(duel) == 0)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if chk == 0 {
        if op == 0 {
            return Ok(false);
        }
        let res = if op == i64::from(ATTRIBUTE_LIGHT) {
            duel.get_location_count(tp, LOCATION_MZONE) > 0
                && duel.is_existing_matching_card(|c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None)
        } else if op == i64::from(ATTRIBUTE_DARK) {
            duel.is_existing_matching_card(CardId::is_able_to_deck, tp, 0, LOCATION_HAND, 1, None)
        } else {
            duel.get_field_group_count(tp, LOCATION_ONFIELD, LOCATION_ONFIELD) > 0
        };
        if !res {
            e.set_label(duel, 0);
        }
        return Ok(res);
    }
    if op == 0 {
        e.set_category(duel, 0);
        e.clear_operation(duel);
    } else if op == i64::from(ATTRIBUTE_LIGHT) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        e.set_operation(duel, spop2);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    } else if op == i64::from(ATTRIBUTE_DARK) {
        e.set_category(duel, CATEGORY_TODECK);
        e.set_operation(duel, tdop);
        duel.set_operation_info(0, CATEGORY_TODECK, None, 1, 1 - tp, LOCATION_HAND as i32);
    } else {
        e.set_category(duel, CATEGORY_DESTROY);
        e.set_operation(duel, desop);
        let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, PLAYER_ALL, LOCATION_ONFIELD as i32);
    }
    e.set_label(duel, 0);
    Ok(false)
}

fn cannot_attack(duel: &mut Duel, e: EffectId) {
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3206);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    cannot_attack(duel, e);
    if duel.get_location_count(tp, LOCATION_MZONE) < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter2(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    cannot_attack(duel, a.e);
    let hg = duel.get_matching_group(CardId::is_able_to_deck, tp, 0, LOCATION_HAND, None);
    if !hg.is_empty() {
        let sg = hg.random_select(duel, tp, 1);
        duel.send_to_deck(sg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    cannot_attack(duel, a.e);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 2, None)?;
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
