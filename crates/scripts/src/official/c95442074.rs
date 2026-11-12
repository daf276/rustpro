// Number 31: Embodiment of Punishment
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 95442074;
const EMBODIMENT_OF_CRIME: u32 = 69058960;
pub const XYZ_NUMBER: u32 = 31;

pub const LISTED_NAMES: &[u32] = &[EMBODIMENT_OF_CRIME];

// Xyz Summoned with two Level 1 monsters, with an opponent's monster to change the position of. Neither a second copy nor Number 13:
// Embodiment of Crime is placed, since two of them in play reach the unported choice among several EFFECT_MUST_ATTACK_MONSTER effects.
pub const SUPPORT: &[(u32, u16)] = &[(1929294, MZONE), (3557275, MZONE), (13193642, MZONE), (26202165, MZONE | OPPONENT)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(1);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register::<ID, EMBODIMENT_OF_CRIME>(duel, c);
}

// Number 13: Embodiment of Crime has the same script with the two codes swapped.
pub fn register<const CODE: u32, const PARTNER: u32>(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(CODE, 0));
    e1.set_category(duel, CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_condition(duel, indcon::<PARTNER>);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_REFLECT_BATTLE_DAMAGE);
    e4.set_condition(duel, refcon::<PARTNER>);
    e4.set_value(duel, 1);
    c.register_effect(duel, e4);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > 0);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let g = duel.get_field_group(tp, 0, LOCATION_MZONE);
    if g.is_empty() {
        return Ok(());
    }
    duel.change_position(g, POS_FACEUP_ATTACK)?;
    let fid = c.get_real_field_id(duel);
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_MUST_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_MUST_ATTACK_MONSTER);
        e2.set_value(duel, Value::Card(atklimit));
        e2.set_label(duel, i64::from(fid));
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn atklimit(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    (i64::from(c.get_real_field_id(duel)) == e.get_label(duel)).into()
}

fn partner_on_field<const PARTNER: u32>(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        |c, duel| c.is_faceup(duel) && c.is_code(duel, PARTNER),
        a.e.get_handler_player(duel),
        LOCATION_MZONE,
        0,
        1,
        None,
    )
}

fn indcon<const PARTNER: u32>(duel: &Duel, a: &Args) -> bool {
    partner_on_field::<PARTNER>(duel, a) && a.e.get_handler(duel).get_overlay_count(duel) != 0
}

fn refcon<const PARTNER: u32>(duel: &Duel, a: &Args) -> bool {
    partner_on_field::<PARTNER>(duel, a) && duel.get_attack_target() == Some(a.e.get_handler(duel))
}
