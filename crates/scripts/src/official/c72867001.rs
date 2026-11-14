// Commander Souza, Swordmaster
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, REMOVED};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 72867001;

pub const LISTED_SERIES: &[u16] = &[SET_X_SABER, SET_SABER];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Synchro Summoned with X-Saber Airbellum (Level 3 Tuner), Anu Piranha (Level 4) and Hard Armor (Level 3), with
// X-Saber monsters in the Graveyard and banished monsters, the opponent's Dark Hole to chain to, and weak opponent monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (90508760, MZONE),
    (23115241, MZONE),
    (20060230, MZONE),
    (23115241, GRAVE),
    (23115241, GRAVE),
    (1929294, REMOVED),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    // The destruction replacement moves another card, so only its prompt is counted.
    Check::Message(MSG_SELECT_EFFECTYN),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_EARTH, Some(sc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(super::c4891376::valcheck));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, _, _| {
            200 * duel.get_matching_group_count(aux::faceup_filter(CardId::is_monster), 0, LOCATION_REMOVED, LOCATION_REMOVED, None) as i32
        }),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_condition(duel, aux::bdocon);
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |_, a| a.ep == 1 - a.tp);
    e3.set_cost(duel, descost);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_code(duel, EFFECT_DESTROY_REPLACE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target(duel, desreptg);
    c.register_effect(duel, e4);
}

fn descostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_X_SABER]) && duel.is_existing_matching_card(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, Some(c.into()))
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |c, duel| descostfilter(c, duel, tp), (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, |c, duel| descostfilter(c, duel, tp), (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn desrepfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SABER]) && c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE)
            && duel.is_existing_matching_card(desrepfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    if duel.select_effect_yes_no(tp, c)? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
        let g = duel.select_matching_card(tp, desrepfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
        return Ok(duel.remove(g, POS_FACEUP, REASON_EFFECT)? > 0);
    }
    Ok(false)
}
