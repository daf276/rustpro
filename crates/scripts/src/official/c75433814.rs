// Number 40: Gimmick Puppet of Strings
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 75433814;
pub const XYZ_NUMBER: u32 = 40;
const COUNTER_STRING: u16 = 0x1024;

// On the field with Xyz Import to attach Sangan, which Strings detaches to place String Counters on the monsters, the opponent's Sangan then
// destroyed at the end of the opponent's turn.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, cttg);
    e1.set_operation(duel, ctop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    Ok(chk == 0 && duel.is_existing_matching_card(CardId::is_faceup, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())))
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    let g = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    for tc in g {
        tc.add_counter(duel, COUNTER_STRING, 1);
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 2);
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    duel.get_flag_effect(a.tp, ID) != 0 && !duel.is_turn_player(a.tp)
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.get_counter(duel, COUNTER_STRING) != 0
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(desfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(desfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, g.len() as i32 * 500);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(desfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let ct = duel.destroy(g, REASON_EFFECT)?;
    duel.damage(1 - tp, ct as u32 * 500, REASON_EFFECT)?;
    Ok(())
}
