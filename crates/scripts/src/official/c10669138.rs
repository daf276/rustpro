// Five-Headed Link Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 10669138;
const FIVE_ATTRIBUTES: u32 = ATTRIBUTE_DARK | ATTRIBUTE_WIND | ATTRIBUTE_FIRE | ATTRIBUTE_WATER | ATTRIBUTE_EARTH;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 5;
    const MAX: Option<usize> = Some(5);
}

// In the Extra Deck and on the field, Link Summoned from one Normal Monster of each attribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (13179332, MZONE),
    (14181608, MZONE),
    (38982356, MZONE),
    (21844576, MZONE),
    (13193642, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(aux::lnklimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_MATERIAL_CHECK);
    e3.set_value(duel, Value::MaterialCheck(valcheck));
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_value(duel, Value::Effect(|duel, e, te| te.get_owner(duel) != e.get_owner(duel)));
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e5.set_value(duel, Value::Card(|duel, _, c| i32::from(c.is_attribute(duel, FIVE_ATTRIBUTES))));
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_category(duel, CATEGORY_REMOVE);
    e6.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e6.set_code(duel, EVENT_PHASE + PHASE_END);
    e6.set_range(duel, LOCATION_MZONE);
    e6.set_count_limit(duel, 1, 0);
    e6.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e6.set_target(duel, rmtg);
    e6.set_operation(duel, rmop);
    c.register_effect(duel, e6);
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let g = c
        .get_material(duel)
        .filter(duel, |mc, duel| mc.is_attribute(duel, FIVE_ATTRIBUTES), None);
    let has = |attribute| g.is_exists(duel, |mc, duel| mc.is_attribute(duel, attribute), 1, None);
    if g.len() == 5
        && has(ATTRIBUTE_EARTH)
        && has(ATTRIBUTE_WIND)
        && has(ATTRIBUTE_FIRE)
        && has(ATTRIBUTE_WATER)
        && has(ATTRIBUTE_DARK)
        && let Some(e2) = e.get_label_object(duel)
    {
        e2.set_label(duel, 1);
    }
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_link_summoned(duel) && a.e.get_label(duel) == 1
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.tp, 0, LOCATION_ONFIELD);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, LOCATION_GRAVE, 0, None);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len().min(5) as u8, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, LOCATION_GRAVE, 0, None);
    if g.len() < 5 {
        duel.send_to_grave(c, REASON_EFFECT)?;
    }
    if g.len() >= 5 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let sg = g.select(duel, tp, 5, 5, None)?;
        if sg.len() != 5 || duel.remove(sg, POS_FACEDOWN, REASON_EFFECT)? != 5 {
            duel.send_to_grave(c, REASON_EFFECT)?;
        }
    }
    Ok(())
}
