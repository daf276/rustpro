// Gigarays Gandora the Dragon of Destruction
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 58330108;

// On the field to use its effect, and in the hand to Special Summon with two Normal Monsters, with Gandora-G and Gandora-X in the
// Graveyard (a third Gandora makes it banish from the Graveyard too) and the opponent's Sangan to remove.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (ID, GRAVE),
    (15025844, HAND),
    (15025844, MZONE),
    (2333466, GRAVE),
    (71525232, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(value));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DESTROY + CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, cost);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_grave_as_cost(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let g = duel.get_matching_group(cfilter, tp, LOCATION_MZONE | LOCATION_HAND, 0, Some(c.into()));
    g.len() >= 2
        && duel.get_location_count(tp, LOCATION_MZONE) > -2
        && aux::select_unselect_group_check(duel, g, 2, 2, &aux::chkf_mmz(tp, 1))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(c) = a.c else { return Ok(false) };
    let sg = duel.get_matching_group(cfilter, tp, LOCATION_MZONE | LOCATION_HAND, 0, Some(c.into()));
    let g = aux::select_unselect_group(duel, sg, 2, 2, &aux::chkf_mmz(tp, 1), tp, HINTMSG_TOGRAVE)?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.send_to_grave(g, REASON_COST)?;
    }
    Ok(())
}

fn value(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    (duel.get_field_group_count(c.get_controler(duel), LOCATION_REMOVED, LOCATION_REMOVED) * 300) as i32
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.pay_lp_cost(a.tp, (duel.get_lp(a.tp) / 2) as u32)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_GANDORA])
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel) && (c.is_location(duel, LOCATION_SZONE) || aux::sp_elim_filter(c, duel, false, true))
}

fn class_count(duel: &Duel, tp: u8) -> usize {
    duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None)
        .get_class_count(duel, |c, duel| i64::from(c.get_code(duel)))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = Some(CardSet::from(a.e.get_handler(duel)));
    let gc = class_count(duel, tp);
    if chk == 0 {
        let b1 = duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, c);
        let b2 = duel.is_existing_matching_card(CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, c);
        let b3 = duel.is_existing_matching_card(
            rmfilter,
            tp,
            LOCATION_ONFIELD | LOCATION_GRAVE,
            LOCATION_ONFIELD | LOCATION_GRAVE,
            1,
            c,
        );
        return Ok((gc == 1 && b1) || (gc == 2 && b2) || (gc > 2 && b3));
    }
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_ALL, LOCATION_ONFIELD as i32);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, PLAYER_ALL, (LOCATION_ONFIELD | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = Some(CardSet::from(a.e.get_handler(duel)));
    let gc = class_count(duel, tp);
    if gc == 1 {
        let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, c);
        duel.destroy(g, REASON_EFFECT)?;
    } else {
        let loc = if gc > 2 {
            LOCATION_ONFIELD | LOCATION_GRAVE
        } else {
            LOCATION_ONFIELD
        };
        let g = duel.get_matching_group(rmfilter, tp, loc, loc, c);
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
