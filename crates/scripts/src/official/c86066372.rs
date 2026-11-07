// Accesscode Talker
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 86066372;
// s.attr_list[tp], at this key plus tp.
const ATTR_LIST: u64 = (ID as u64) << 8;

// A copy in the Extra Deck with Code Talker (a Link Monster as material, then a target) and Code Generator on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (53413628, MZONE), (30114823, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, atkcon);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    e1.set_label_object(duel, Some(e0));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, descost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        duel.set_script_state(ATTR_LIST, 0);
        duel.set_script_state(ATTR_LIST + 1, 0);
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_PHASE + PHASE_END);
        ge1.set_count_limit(duel, 1, 0);
        ge1.set_condition_mut(duel, resetop);
        duel.register_effect(ge1, 0);
    });
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let g = c.get_material(duel);
    e.set_label(duel, 0);
    if g.is_exists(duel, |c, duel| c.is_link_monster(duel), 1, None) {
        e.set_label(duel, 1);
    }
}

fn resetop(duel: &mut Duel, _: &Args) -> bool {
    duel.set_script_state(ATTR_LIST, 0);
    duel.set_script_state(ATTR_LIST + 1, 0);
    false
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_link_summoned(duel) && a.e.get_label_object(duel).is_some_and(|e0| e0.get_label(duel) == 1)
}

fn atkfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_type(duel, TYPE_LINK)
        && (c.is_location(duel, LOCATION_GRAVE) || (c.is_location(duel, LOCATION_REMOVED) && c.is_faceup(duel)))
        && c.is_can_be_effect_target(duel, e)
}

fn chlimit(_: &Duel, _: EffectId, ep: u8, tp: u8, _: i64) -> bool {
    tp == ep
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = c.get_material(duel).filter(duel, |c, duel| atkfilter(c, duel, e), None);
    if let Some(chkc) = chkc {
        return Ok(c.get_material(duel).contains(chkc) && atkfilter(chkc, duel, e));
    }
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let tg = c
        .get_material(duel)
        .filter_select(duel, tp, |c, duel| atkfilter(c, duel, e), 1, 1, None)?;
    duel.set_target_card(tg);
    duel.set_chain_limit(chlimit, 0);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) && tc.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, tc.get_link(duel) as i32 * 1000);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn costfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let attr = c.get_attribute(duel);
    c.is_link_monster(duel) && c.is_able_to_remove_as_cost(duel) && duel.get_script_state(ATTR_LIST + u64::from(tp)) & i64::from(attr) == 0
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let filter = |c, duel: &Duel| costfilter(c, duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
    let attribute = g.get_first().map_or(0, |c| c.get_attribute(duel));
    a.e.set_label(duel, attribute.into());
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    duel.set_chain_limit(chlimit, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let att = a.e.get_label(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.destroy(g, REASON_EFFECT)?;
    }
    let key = ATTR_LIST + u64::from(tp);
    duel.set_script_state(key, duel.get_script_state(key) | att);
    for string in aux::get_attribute_strings(att as u32) {
        c.register_flag_effect_full(duel, 0, RESETS_STANDARD_PHASE_END, EFFECT_FLAG_CLIENT_HINT, 1, 0, string);
    }
    Ok(())
}
