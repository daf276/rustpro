// Number C105: Battlin' Boxer Comet Cestus
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 85121942;
pub const XYZ_NUMBER: u32 = 105;
const STAR_CESTUS: u32 = 59627393;

pub const LISTED_NAMES: &[u32] = &[STAR_CESTUS];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 4;
}

// Number 105: Battlin' Boxer Star Cestus on the opponent's side, attached by Xyz Import, and Sangan to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (STAR_CESTUS, MZONE | OPPONENT),
    (STAR_CESTUS, MZONE | OPPONENT),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_condition(duel, damcon);
    e1.set_target(duel, damtg);
    e1.set_operation(duel, stock::damage);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, descon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_relate_to_battle(duel)
        && c.get_battle_target(duel)
            .is_some_and(|bc| bc.is_location(duel, LOCATION_GRAVE) && bc.is_monster(duel))
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let Some(bc) = a.e.get_handler(duel).get_battle_target(duel) else {
        return Ok(false);
    };
    let dam = (bc.get_text_attack(duel) / 2).max(0);
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_overlay_group(duel)
        .is_exists(duel, |c, duel| c.is_code(duel, STAR_CESTUS), 1, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    let atk = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_attack(duel));
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, atk);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_controler(duel, 1 - tp) {
        let mut atk = tc.get_attack(duel);
        if atk < 0 || tc.is_facedown(duel) {
            atk = 0;
        }
        if duel.destroy(tc, REASON_EFFECT)? != 0 {
            duel.damage(1 - tp, atk as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
