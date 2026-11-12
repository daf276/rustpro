// Jormungandr, Generaider Boss of Eternity
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 2665273;

// In the Extra Deck, Xyz Summoned with Level 9 monsters (Wandering Titan of Tartarus, Thunder King, the Lightningstrike Kaiju).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (43363035, MZONE),
    (43363035, MZONE),
    (48770333, MZONE),
    (48770333, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(9);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_BASE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SET_BASE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    c.get_overlay_count(duel) as i32 * 1000
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1) && duel.is_player_can_draw(1 - tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, PLAYER_ALL, 1);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut ps = Vec::new();
    if duel.draw(tp, 1, REASON_EFFECT)? > 0 {
        ps.push(tp);
    }
    if duel.draw(1 - tp, 1, REASON_EFFECT)? > 0 {
        ps.push(1 - tp);
    }
    if ps.is_empty() || !c.is_relate_to_effect(duel, e) || !c.is_faceup(duel) {
        return Ok(());
    }
    duel.break_effect()?;
    for p in ps {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
        let g = duel.select_matching_card(
            p,
            |tc, duel| tc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT),
            p,
            LOCATION_HAND | LOCATION_ONFIELD,
            0,
            1,
            1,
            Some(c.into()),
        )?;
        if let Some(tc) = g.get_first() {
            tc.cancel_to_grave(duel);
            duel.overlay_full(c, tc, true)?;
        }
    }
    Ok(())
}
