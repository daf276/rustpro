// G Golem Invalid Dolmen
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 24151924;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_EARTH, Some(lc), sumtype, tp)
    }
}

// Copies on the field to link each other and in the Extra Deck, Link Summoned from EARTH Normal Monsters, with Cyberse monsters (Digitron,
// Bitron) in the hand to discard and Raigeki from the opponent to destroy the co-linked copies.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (756652, MZONE),
    (2830619, MZONE),
    (756652, MZONE),
    (32295838, HAND),
    (36211150, HAND),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_link_monster(duel) && c.get_mutual_linked_group_count(duel) > 0
    });
    e1.set_value(duel, Value::Effect(efilter));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_MUST_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_MUST_ATTACK_MONSTER);
    e3.set_value(duel, Value::Card(|duel, e, c| i32::from(c == e.get_handler(duel))));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_DRAW);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_cost(duel, drcost);
    e4.set_target(duel, drtg);
    e4.set_operation(duel, drop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_DISABLE);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e5.set_property(duel, EFFECT_FLAG_DELAY);
    e5.set_code(duel, EVENT_DESTROYED);
    e5.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD) && a.e.get_label(duel) > 0
    });
    e5.set_target(duel, distg);
    e5.set_operation(duel, disop);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e6.set_code(duel, EVENT_LEAVE_FIELD_P);
    e6.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e6.set_label_object(duel, Some(e5));
    e6.set_operation(duel, chk);
    c.register_effect(duel, e6);
}

fn efilter(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    let loc: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_LOCATION);
    te.is_activated(duel)
        && te.is_monster_effect(duel)
        && loc == LOCATION_MZONE
        && e.get_handler_player(duel) == 1 - te.get_handler_player(duel)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_CYBERSE) && c.is_discardable(duel)
}

fn drcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, tgfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.draw(a.tp, 1, REASON_EFFECT)?;
    Ok(())
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk != 0 || duel.is_existing_matching_card(CardId::is_negatable, a.tp, 0, LOCATION_ONFIELD, 1, None))
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(CardId::is_negatable, a.tp, 0, LOCATION_ONFIELD, None);
    if g.is_empty() {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e2);
        if tc.is_type(duel, TYPE_TRAPMONSTER) {
            let e3 = duel.create_effect(c);
            e3.set_type(duel, EFFECT_TYPE_SINGLE);
            e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
            e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e3);
        }
    }
    Ok(())
}

fn chk(duel: &mut Duel, a: &Args) -> Script<()> {
    let count = a.e.get_handler(duel).get_mutual_linked_group_count(duel);
    if let Some(e5) = a.e.get_label_object(duel) {
        e5.set_label(duel, count as i64);
    }
    Ok(())
}
