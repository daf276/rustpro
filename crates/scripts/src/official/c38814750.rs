// PSY-Framegear Gamma
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 38814750;

pub const LISTED_NAMES: &[u32] = &[CARD_PSYFRAME_DRIVER];

// In the hand with PSY-Frame Driver in the Deck, against monsters on the opponent's field that activate effects.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_PSYFRAME_DRIVER, DECK),
    (CARD_PSYFRAME_DRIVER, DECK),
    (ID, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_unsummonable(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(
        duel,
        Value::SpSummonCondition(|duel, _, se, _, _, _, _| se.is_some_and(|se| se.is_has_type(duel, EFFECT_TYPE_ACTIONS))),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.ep != tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
        && (duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0 || duel.is_player_affected_by_effect(tp, CARD_PSYFRAME_LAMBDA).is_some())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_PSYFRAME_DRIVER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(!c.is_status(duel, STATUS_CHAINING)
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp),
                tp,
                LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        2,
        tp,
        (LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE) as i32,
    );
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
        }
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    if duel.get_location_count(tp, LOCATION_MZONE) < 2 || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let mut g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    let fid = c.get_field_id(duel);
    duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)?;
    for sc in [tc, c] {
        sc.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1, fid.into(), 0);
    }
    duel.special_summon_complete()?;
    g.merge(c.into());
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_label(duel, fid.into());
    e1.set_label_object_group(duel, Some(g));
    e1.set_condition_mut(duel, rmcon::<ID>);
    e1.set_operation(duel, rmop::<ID>);
    duel.register_effect(e1, tp);
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn rmfilter<const CODE: u32>(c: CardId, duel: &Duel, fid: i64) -> bool {
    c.get_flag_effect_label(duel, CODE) == Some(fid)
}

pub(super) fn rmcon<const CODE: u32>(duel: &mut Duel, a: &Args) -> bool {
    let fid = a.e.get_label(duel);
    let g = a.e.get_label_object_group(duel).unwrap_or_default();
    if g.is_exists(duel, |c, duel| rmfilter::<CODE>(c, duel, fid), 1, None) {
        return true;
    }
    a.e.reset(duel);
    false
}

pub(super) fn rmop<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let fid = a.e.get_label(duel);
    let g = a.e.get_label_object_group(duel).unwrap_or_default();
    let tg = g.filter(duel, |c, duel| rmfilter::<CODE>(c, duel, fid), None);
    duel.remove(tg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
