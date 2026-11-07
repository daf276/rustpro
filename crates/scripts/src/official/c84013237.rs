// Number 39: Utopia
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 84013237;

// In the Extra Deck, Xyz Summoned with Mystical Elf and Flying Kamakiri #2.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (15025844, MZONE),
    (3134241, MZONE),
    (79629370, MZONE),
];

// Being attacked without materials is too rare to assert.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, |duel, _| duel.negate_attack().map(drop));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.get_attack_target() == Some(c) && c.get_overlay_count(duel) == 0
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.destroy(c, REASON_EFFECT)?;
    }
    Ok(())
}
