// End of the World Ruler
use engine::{Operation, Target};

use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 3739500;
const DEMISE: u32 = 72426662;

pub const LISTED_NAMES: &[u32] = &[DEMISE];

// In the hand with Advanced Ritual Art and Black Illusion Ritual in the Deck to apply, with Normal Monsters in the hand and Deck
// to be their materials, and on the field with cards on both sides to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (46052429, DECK),
    (41426869, DECK),
    (46052429, DECK),
    (3606209, HAND),
    (3606209, DECK),
    (15025844, HAND),
    (28546905, DECK),
    (28546905, DECK),
    (26202165, MZONE | OPPONENT),
    (4206964, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e1.set_value(duel, DEMISE as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::self_reveal, Cost::pay_lp::<2000>, applycost])
    });
    e2.set_target(duel, applytg);
    e2.set_operation(duel, applyop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY + CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::pay_lp::<2000>);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn applyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_ritual_spell(duel) && c.is_able_to_remove_as_cost(duel) && c.check_activate_effect(duel, true, true).is_some()
}

fn applycost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(applyfilter, a.tp, LOCATION_DECK, 0, 1, None))
}

fn applytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        // The Lua reads an undefined `tg`.
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(applyfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, applyfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    let te = g.get_first().and_then(|sc| sc.check_activate_effect(duel, true, false));
    unported!(duel, te.is_none());
    let Some(te) = te else { return Ok(false) };
    e.set_label(duel, te.get_label(duel));
    let label_object = te.get_label_object(duel);
    e.set_label_object(duel, label_object);
    let tg = te.get_target(duel);
    let (property, property2) = te.get_property_full(duel);
    e.set_property_full(duel, property, property2);
    match tg {
        Target::None => {}
        Target::Action(tg) => {
            tg(duel, a, 1, None)?;
        }
        other => unported!(duel, !matches!(other, Target::None)),
    }
    te.set_label(duel, e.get_label(duel));
    let label_object = e.get_label_object(duel);
    te.set_label_object(duel, label_object);
    e.set_label_object(duel, Some(te));
    duel.clear_operation_info(0);
    Ok(false)
}

fn applyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(te) = e.get_label_object(duel) else { return Ok(()) };
    e.set_label(duel, te.get_label(duel));
    let label_object = te.get_label_object(duel);
    e.set_label_object(duel, label_object);
    let op = te.get_operation(duel);
    unported!(duel, !matches!(op, None | Some(Operation::Event(_) | Operation::Promptless(_))));
    if let Some(Operation::Event(op) | Operation::Promptless(op)) = op {
        op(duel, a)?;
    }
    te.set_label(duel, e.get_label(duel));
    let label_object = e.get_label_object(duel);
    te.set_label_object(duel, label_object);
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let exc = c.is_relate_to_effect(duel, e).then(|| c.into());
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, exc);
    if !g.is_empty() && duel.destroy(g, REASON_EFFECT)? > 0 && c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        duel.break_effect()?;
        c.update_attack(duel, 2900, None, None, None);
    }
    Ok(())
}
