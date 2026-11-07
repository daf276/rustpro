// Number 65: Djinn Buster
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 3790062;

// In the Extra Deck, Xyz Summoned with two Level 2 DARK monsters.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (4920010, MZONE), (17238333, MZONE), (22026707, MZONE)];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.rp != a.tp
        && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, 500);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8) {
        duel.damage(1 - a.tp, 500, REASON_EFFECT)?;
    }
    Ok(())
}
