// Coach King Giantrainer
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 30741334;

// Twice in the Extra Deck, Xyz Summoned with Level 8 Normal Monsters (Black Luster Soldier, Tri-Horned Dragon, Gogiga Gagagigo).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (10000100, MZONE),
    (39111158, MZONE),
    (39674352, MZONE),
    (10000100, MZONE),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 3, ID);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<1, 1>, drcost])
    });
    e1.set_target(duel, drtg);
    e1.set_operation(duel, drop);
    c.register_effect(duel, e1);
}

fn drcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_phase(PHASE_MAIN1));
    }
    let c = a.e.get_handler(duel);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e1.set_code(duel, EFFECT_CANNOT_BP);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 1, 1 - tp, 800);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.draw(tp, 1, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let Some(dc) = duel.get_operated_group().get_first() else {
        return Ok(());
    };
    duel.confirm_cards(1 - tp, dc)?;
    if dc.is_monster(duel) {
        duel.break_effect()?;
        duel.damage(1 - tp, 800, REASON_EFFECT)?;
    }
    duel.shuffle_hand(tp)?;
    Ok(())
}
