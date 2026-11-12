// Number 21: Frozen Lady Justice
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 57707471;
pub const XYZ_NUMBER: u32 = 21;

// In the Extra Deck, Xyz Summoned with two Level 6 Normal Monsters (Frostosaurus), with the opponent's monsters to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (6631034, MZONE),
    (6631034, MZONE),
    (6631034, MZONE),
    (6631034, MZONE),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
];

pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, mc: CardId) -> bool {
        mc.check_remove_overlay_card(duel, tp, 1, REASON_COST)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, mc: CardId) -> Script<bool> {
        mc.remove_overlay_card(duel, tp, 1, 1, REASON_COST)?;
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(|duel, _, c| c.get_overlay_count(duel) as i32 * 1000));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    c.is_faceup(duel) && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), u64::from(SUMMON_TYPE_XYZ), tp) && c.get_rank(duel) == 5
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_defense_pos(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(desfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(desfilter, tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(desfilter, a.tp, 0, LOCATION_MZONE, None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
