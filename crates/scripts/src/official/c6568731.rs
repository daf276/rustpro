// Glacial Beast Iceberg Narwhal
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, SZONE};

const ID: u32 = 6568731;

// In the Extra Deck, Synchro Summoned from Water Spirit (Level 1 Tuner) and Boulder Tortoise (Level 6 WATER), with Doron to be
// destroyed by Pesanta of the opponent's and Mirror Force Set to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (487395, MZONE),
    (487395, MZONE),
    (487395, MZONE),
    (9540040, MZONE),
    (9540040, MZONE),
    (9540040, MZONE),
    (756652, MZONE),
    (756652, MZONE),
    (66429798, MZONE | OPPONENT),
    (66429798, MZONE | OPPONENT),
    (44095762, SZONE | OPPONENT),
    (44095762, SZONE | OPPONENT),
];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| {
            c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(sc), sumtype, tp)
        })
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, damcon);
    e1.set_target(duel, damtg);
    e1.set_operation(duel, stock::damage);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, discon);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_reason(duel, REASON_DESTROY)
        && ((c.is_reason(duel, REASON_EFFECT) && c.get_reason_player(duel) == 1 - tp)
            || (c.is_reason(duel, REASON_BATTLE) && duel.get_attacker().is_some_and(|attacker| attacker.is_controler(duel, 1 - tp))))
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, tp), 1, None))
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(600);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 600);
    Ok(false)
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.is_battle_phase()
        && a.ep == 1 - tp
        && duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_MZONE, 0, 1, Some(a.e.get_handler(duel).into()))
        && duel.is_chain_disablable(a.ev as u8)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_effect(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
