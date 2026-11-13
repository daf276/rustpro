// Beat Cop from the Underworld
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 99011763;
const COUNTER: u16 = 0x1049;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);
}

// In the Extra Deck, Link Summoned from two DARK monsters with different names, with monsters on the field to Tribute and to counter.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (13193642, MZONE),
    (77581312, MZONE),
    (80600490, MZONE),
    (30325729, MZONE),
    (26202165, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e1);
}

fn matcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let g = c.get_material(duel);
    if g.len() == 2
        && g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) == g.len()
        && !g.is_exists(duel, |c, duel| !c.is_attribute(duel, ATTRIBUTE_DARK), 1, None)
    {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 0));
        e1.set_category(duel, CATEGORY_COUNTER);
        e1.set_type(duel, EFFECT_TYPE_IGNITION);
        e1.set_count_limit(duel, 1, ID);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        e1.set_cost(duel, addcost);
        e1.set_target(duel, addtg);
        e1.set_operation(duel, addop);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
        c.register_effect(duel, e1);
    }
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    duel.is_existing_target(
        |tc, duel| tc.is_can_add_counter(duel, COUNTER, 1),
        0,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        Some(c.into()),
    )
}

fn addcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, cfilter, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, cfilter, (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn addtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_COUNTER);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    Ok(false)
}

fn addop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        tc.add_counter(duel, COUNTER, 1);
        if tc.get_flag_effect(duel, ID) != 0 {
            return Ok(());
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EFFECT_DESTROY_REPLACE);
        e1.set_target(duel, reptg);
        e1.set_operation(duel, repop);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        tc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 0);
    }
    Ok(())
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE + REASON_RULE) && c.get_counter(duel, COUNTER) > 0);
    }
    Ok(true)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel).remove_counter(duel, a.tp, COUNTER, 1, REASON_EFFECT)?;
    Ok(())
}
