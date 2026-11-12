// Star Mine
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 49407319;

pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Three copies side by side on the field, destroyed by the opponent's Dark Hole or by battle with Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (53129443, HAND | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
// The first effect carries the second description and the second has none.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1)), Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_condition(duel, damcon1);
    e1.set_target(duel, damtg1);
    e1.set_operation(duel, damop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    // Mirrors the Lua, which sets these on e1 after registering it.
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_condition(duel, damcon2);
    e2.set_target(duel, damtg2);
    e2.set_operation(duel, damop2);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, tp: u8, rp: u8) -> bool {
    if c.is_reason(duel, REASON_BATTLE)
        && let Some(ac) = duel.get_attacker()
        && ac != c
    {
        if ac.is_location(duel, LOCATION_MZONE) {
            ac.is_controler(duel, 1 - tp)
        } else {
            ac.is_previous_controler(duel, 1 - tp)
        }
    } else {
        c.is_reason(duel, REASON_EFFECT) && rp != tp
    }
}

fn damcon1(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    filter(c, duel, a.tp, a.rp) && c.is_previous_controler(duel, a.tp)
}

fn damtg1(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, PLAYER_ALL, 2000);
    Ok(false)
}

fn damop1(duel: &mut Duel, a: &Args) -> Script<()> {
    damage_both(duel, a.tp)
}

fn damage_both(duel: &mut Duel, tp: u8) -> Script<()> {
    let val = duel.damage(tp, 2000, REASON_EFFECT)?;
    if val > 0 && duel.get_lp(tp) > 0 {
        duel.break_effect()?;
        duel.damage(1 - tp, 2000, REASON_EFFECT)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8, seq: u32) -> bool {
    if !c.is_previous_controler(duel, tp) || !c.is_previous_location(duel, LOCATION_MZONE) {
        return false;
    }
    let previous = c.get_previous_sequence(duel);
    (seq >= 1 && previous == seq - 1) || (seq < 4 && previous == seq + 1)
}

fn damcon2(duel: &Duel, a: &Args) -> bool {
    let (tp, seq) = (a.tp, a.e.get_handler(duel).get_sequence(duel));
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| filter(c, duel, tp, a.rp), None);
    g.is_exists(duel, |c, duel| cfilter(c, duel, tp, seq), 1, None)
}

fn damtg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, a.tp, LOCATION_MZONE as i32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, PLAYER_ALL, 2000);
    Ok(false)
}

fn damop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && duel.destroy(c, REASON_EFFECT)? > 0 {
        damage_both(duel, a.tp)?;
    }
    Ok(())
}
