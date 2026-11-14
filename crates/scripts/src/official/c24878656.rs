// Toy Box
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 24878656;

pub const LISTED_SERIES: &[u16] = &[SET_TOY];

// Face-up and in the hand, with Toy Soldier in the hand, Deck and Graveyard to Set, and Elfnote Lucina on the opponent's field to
// attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (65504487, HAND),
    (65504487, DECK),
    (65504487, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|ac| ac.is_controler(duel, 1 - a.tp))
    });
    e2.set_cost(duel, mondescost);
    e2.set_target(duel, mondestg);
    e2.set_operation(duel, mondesop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, efftg);
    e3.set_operation(duel, effop);
    c.register_effect(duel, e3);
}

const LOCATIONS_HAND_DECK_MZONE_GRAVE: u32 = LOCATION_HAND | LOCATION_DECK | LOCATION_MZONE | LOCATION_GRAVE;

fn cstfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_able_to_grave_as_cost(duel)
}

fn mondescost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, cstfilter, LOCATION_ONFIELD)
}

fn mondestg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let at = duel.get_attacker();
    if chk == 0 {
        return Ok(at.is_some_and(|at| at.is_relate_to_battle(duel)));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, at.map(CardSet::from), 1, 0, 0);
    Ok(false)
}

fn mondesop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(at) = duel.get_attacker()
        && at.is_relate_to_battle(duel)
        && at.is_controler(duel, 1 - a.tp)
    {
        duel.destroy(at, REASON_EFFECT)?;
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    let set_eff = c.is_has_effect(duel, EFFECT_MONSTER_SSET);
    c.is_original_set_card(duel, &[SET_TOY])
        && c.is_monster(duel)
        && set_eff.is_some_and(|set_eff| matches!(set_eff.get_value(duel), Value::Int(value) if value & TYPE_SPELL as i32 > 0))
        && c.is_ssetable(duel)
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_MZONE))
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_STZONE, 0, None);
    let b1 = duel.is_existing_matching_card(setfilter, tp, LOCATIONS_HAND_DECK_MZONE_GRAVE, 0, 1, None);
    let b2 = !g.is_empty();
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => e.set_category(duel, CATEGORY_SET),
        Some(2) => {
            e.set_category(duel, CATEGORY_DESTROY);
            duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    match a.e.get_label(duel) {
        1 => {
            let ft = duel.get_location_count(tp, LOCATION_SZONE).min(2);
            if ft == 0 {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
            let g = duel.select_matching_card(
                tp,
                aux::necro_valley_filter(setfilter),
                tp,
                LOCATIONS_HAND_DECK_MZONE_GRAVE,
                0,
                1,
                ft as u8,
                None,
            )?;
            if !g.is_empty() {
                duel.sset(tp, g)?;
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_STZONE, 0, 1, 2, None)?;
            if !g.is_empty() {
                duel.hint_selection_full(g, true);
                duel.destroy(g, REASON_EFFECT)?;
            }
        }
        _ => {}
    }
    Ok(())
}
