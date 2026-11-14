// Ursarctic Big Dipper
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 89264428;

pub const LISTED_SERIES: &[u16] = &[SET_URSARCTIC];
pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_BIG_DIPPER];
const COUNTER_BIG_DIPPER: u16 = 0x204;

// In the hand to activate, with Polari on the field to replace its Tribute with Ursarctic Megabilis in the Graveyard, and Sangan on the
// other side; the control effect needs seven counters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (27693363, MZONE),
    (81108658, GRAVE),
    (81108658, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(0),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_BIG_DIPPER);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_COST_REPLACE);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, repcon);
    e1.set_value(duel, Value::CostReplace(repval));
    e1.set_cost_replace_operation(duel, repop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_operation(duel, |duel, a| {
        a.e.get_handler(duel).add_counter(duel, COUNTER_BIG_DIPPER, 1);
        Ok(())
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_CONTROL);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, ctrlcost);
    e3.set_target(duel, ctrltg);
    e3.set_operation(duel, ctrlop);
    c.register_effect(duel, e3);
}

fn repconfilter(c: CardId, duel: &Duel, extra: Option<(EffectId, CostReplaceCall)>) -> bool {
    c.is_level_above(duel, 7)
        && c.is_set_card(duel, &[SET_URSARCTIC])
        && c.is_able_to_remove_as_cost(duel)
        && extra.is_none_or(|(base, call)| (call.extracon)(duel, base, &Args { c: Some(c), ..*call.args }))
}

fn repcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        |c, duel| repconfilter(c, duel, None),
        a.e.get_handler_player(duel),
        LOCATION_GRAVE,
        0,
        1,
        None,
    )
}

fn repval(duel: &Duel, base: EffectId, call: CostReplaceCall) -> bool {
    let e = call.args.e;
    let c = e.get_handler(duel);
    c.is_set_card(duel, &[SET_URSARCTIC])
        && c.is_monster(duel)
        && duel.is_existing_matching_card(
            |gc, duel| repconfilter(gc, duel, Some((base, call))),
            e.get_handler_player(duel),
            LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn repop(duel: &mut Duel, base: EffectId, call: CostReplaceCall) -> Script<()> {
    let tp = call.args.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| repconfilter(c, duel, Some((base, call))),
        tp,
        LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    duel.remove(g, POS_FACEUP, REASON_COST | REASON_REPLACE)?;
    Ok(())
}

fn ctrlcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_can_remove_counter(duel, tp, COUNTER_BIG_DIPPER, 7, REASON_COST));
    }
    let count = c.get_counter(duel, COUNTER_BIG_DIPPER);
    c.remove_counter(duel, tp, COUNTER_BIG_DIPPER, count, REASON_COST)?;
    Ok(false)
}

fn ctrlconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_URSARCTIC]) && c.is_type(duel, TYPE_SYNCHRO) && c.is_faceup(duel)
}

fn ctrltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_controler_can_be_changed(duel));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(ctrlconfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
                && duel.is_existing_target(CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(tp, CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, tp, 0);
    Ok(false)
}

fn ctrlop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.get_control(tc, a.tp)?;
    }
    Ok(())
}
