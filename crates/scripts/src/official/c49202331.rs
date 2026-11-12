// CXyz Skypalace Babylon
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 49202331;

pub const LISTED_NAMES: &[u32] = &[3814632];

// In the Extra Deck, Xyz Summoned with three Level 11 monsters (Flower Cardian Willow), or on the field with Skypalace Gangaridai attached by
// Xyz Import, attacking the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (54135423, MZONE),
    (54135423, MZONE),
    (54135423, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (3814632, MZONE | OPPONENT),
    (3814632, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(11);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_condition(duel, aux::bdgcon);
    e1.set_target(duel, damtg);
    e1.set_operation(duel, damop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, atcon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_operation(duel, atop);
    c.register_effect(duel, e2);
}

// The core rounds the Lua's `atk/2` where it reads it as an integer.
fn half_attack(duel: &Duel, c: CardId) -> i32 {
    ((c.get_attack(duel) + 1) / 2).max(0)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let Some(bc) = a.e.get_handler(duel).get_battle_target(duel) else {
        return Ok(false);
    };
    duel.set_target_card(bc);
    let dam = half_attack(duel, bc);
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
        let dam = half_attack(duel, tc);
        duel.damage(p, dam as u32, REASON_EFFECT)?;
    }
    Ok(())
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    aux::bdcon(duel, a)
        && duel.check_mut(|duel| c.can_chain_attack(duel))
        && c.get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_code(duel, 3814632), 1, None)
}

fn atop(duel: &mut Duel, _: &Args) -> Script<()> {
    duel.chain_attack();
    Ok(())
}
