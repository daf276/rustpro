// Marincess Current
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, SZONE};

const ID: u32 = 84430165;

pub const LISTED_SERIES: &[u16] = &[SET_MARINCESS];
// Set with Marincess Marbled Rock on the field to destroy a weak opponent's monster in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (5524387, MZONE),
    (5524387, MZONE),
    (13597785, MZONE | OPPONENT),
    (7892180, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, damcon);
    e1.set_target(duel, damtg);
    e1.set_operation(duel, damop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    e2.set_condition(duel, actcon);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MARINCESS]) && c.is_link_monster(duel) && c.is_controler(duel, tp)
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    a.eg.and_then(|eg| eg.get_first()).is_some_and(|c| cfilter(c, duel, a.tp))
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ac = a.eg.and_then(|eg| eg.get_first());
    if chk == 0 {
        return Ok(ac.is_some_and(|ac| ac.is_relate_to_battle(duel)));
    }
    duel.set_target_player(1 - tp);
    duel.set_operation_info(
        0,
        CATEGORY_DAMAGE,
        None,
        0,
        1 - tp,
        ac.map_or(0, |ac| ac.get_link(duel) as i32 * 400),
    );
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let Some(ac) = a.eg.and_then(|eg| eg.get_first()) else {
        return Ok(());
    };
    duel.damage(p, ac.get_link(duel) * 400, REASON_EFFECT)?;
    let battle_target = ac.get_battle_target(duel);
    if duel
        .get_matching_group(|c, duel| cfilter(c, duel, tp), tp, LOCATION_MZONE, 0, None)
        .is_exists(duel, |c, duel| c.is_link_above(duel, 2), 1, None)
        && battle_target.is_some_and(|bt| bt.is_link_monster(duel))
        && ac.get_link(duel) > 0
    {
        duel.break_effect()?;
        if let Some(bt) = battle_target {
            duel.damage(p, bt.get_link(duel) * 500, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn actfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MARINCESS]) && c.is_link_monster(duel) && c.is_link_above(duel, 3)
}

fn actcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(actfilter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
}
