// Number 104: Masquerade
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 2061963;
pub const XYZ_NUMBER: u32 = 104;

// In the Extra Deck and on the field, Xyz Summoned with Mystical Elf, Flying Kamakiri #2 and Maiden of the Moonlight.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, MZONE), (15025844, MZONE), (3134241, MZONE), (79629370, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
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
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DECKDES);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, decktg);
    e2.set_operation(duel, deckop);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.ep != a.tp
        && duel.is_battle_phase()
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, 800);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8) {
        duel.damage(1 - a.tp, 800, REASON_EFFECT)?;
    }
    Ok(())
}

fn decktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(1 - tp, 1));
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, 1 - tp, 1);
    Ok(false)
}

fn deckop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u16 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.discard_deck(p, d, REASON_EFFECT)?;
    Ok(())
}
