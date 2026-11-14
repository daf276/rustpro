// Fusion Duplication
use engine::{Operation, Target};

use crate::keys;
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, OPPONENT};

const ID: u32 = 43331750;

pub const LISTED_SERIES: &[u16] = &[SET_FUSION];
// Polymerization in each Graveyard to copy, with Fallen of Albaz and Petit Angel (LIGHT) in the hand for Albion the Branded
// Dragon in the Elfnote Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (24094653, GRAVE),
    (24094653, GRAVE | OPPONENT),
    (68468459, HAND),
    (38142739, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cpfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FUSION])
        && (c.is_normal_spell(duel) || c.is_quick_play_spell(duel))
        && c.is_able_to_remove(duel)
        && c.check_activate_effect(duel, true, true).is_some()
}

// Fusion.SummonEffTG/OP keep chkf in the copied card's closure, which the procedure keys by the calling effect's owner.
fn with_closure_of<T>(duel: &mut Duel, e: EffectId, te: EffectId, op: bool, f: impl FnOnce(&mut Duel) -> Script<T>) -> Script<T> {
    let (own, copied) = (keys::fusion_chkf(e.get_owner(duel), op), keys::fusion_chkf(te.get_owner(duel), op));
    let state = duel.get_script_state(copied);
    duel.set_script_state(own, state);
    let result = f(duel)?;
    let state = duel.get_script_state(own);
    duel.set_script_state(copied, state);
    duel.set_script_state(own, 0);
    Ok(result)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        // The Lua calls GetFirst on the card label object and errors.
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(cpfilter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let Some(tc) = duel
        .select_target(tp, cpfilter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, 1, None)?
        .and_then(|g| g.get_first())
    else {
        return Ok(false);
    };
    e.set_label_object_group(duel, Some(tc.into()));
    let Some(te) = tc.check_activate_effect(duel, true, true) else {
        // The Lua errors on the nil effect.
        unported!(duel, true);
        return Ok(false);
    };
    let (property, _) = te.get_property_full(duel);
    e.set_property(duel, EFFECT_FLAG_CARD_TARGET | property);
    match te.get_target(duel) {
        Target::None => {}
        Target::Action(tg) => {
            with_closure_of(duel, e, te, false, |duel| tg(duel, a, 1, None))?;
        }
        _ => unported!(duel, true),
    }
    duel.clear_operation_info(0);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(tc.into()), 1, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(tc) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    if !tc.is_relate_to_effect(duel, e) || duel.remove(tc, POS_FACEUP, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let Some(te) = tc.check_activate_effect(duel, true, true) else {
        return Ok(());
    };
    let op = te.get_operation(duel);
    unported!(duel, !matches!(op, None | Some(Operation::Event(_) | Operation::Promptless(_))));
    if let Some(Operation::Event(op) | Operation::Promptless(op)) = op {
        with_closure_of(duel, e, te, true, |duel| op(duel, a))?;
    }
    Ok(())
}
