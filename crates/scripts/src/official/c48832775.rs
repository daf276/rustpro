// Dinomorphia Kentregina
use engine::{Operation, Target};

use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 48832775;

pub const LISTED_SERIES: &[u16] = &[SET_DINOMORPHIA];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::GroupFilter(ffilter); 2];
}

// Fusion Summoned by Polymerization with Therizia and Diplos; on the field with Domain in the GY to copy and Diplos there to
// summon once the opponent's Raigeki destroys it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (24094653, HAND),
    (92133240, HAND),
    (38628859, HAND),
    (26631975, GRAVE),
    (26631975, GRAVE),
    (38628859, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, Value::Card(|duel, _, c| -duel.get_lp(c.get_controler(duel))));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_cost(duel, efcost);
    e2.set_target(duel, eftg);
    e2.set_operation(duel, efop);
    c.register_effect(duel, e2);
    super::c92133240::register_destroyed_summon::<ID, 4, false>(duel, c);
}

// Also Stealthbergia's material filter: Dinomorphia monsters with different names.
pub fn ffilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, sg: Option<CardSet>) -> bool {
    c.is_set_card_full(duel, &[SET_DINOMORPHIA], Some(fc), sumtype, tp)
        && sg.is_none_or(|sg| {
            // A multiple-value GetCode in the middle of an argument list passes only its first value.
            let codes = c.get_codes_full(duel, Some(fc), sumtype, tp);
            let code = codes.first().map_or(&[][..], std::slice::from_ref);
            !sg.is_exists(duel, |sc, duel| fusfilter(sc, duel, code, fc, sumtype, tp), 1, Some(c.into()))
        })
}

fn fusfilter(c: CardId, duel: &Duel, code: &[u32], fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_summon_code(duel, Some(fc), sumtype, tp, code) && c.is_has_effect(duel, 511002961).is_none()
}

fn effilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove_as_cost(duel)
        && c.is_set_card(duel, &[SET_DINOMORPHIA])
        && c.is_normal_trap(duel)
        && c.check_activate_effect(duel, false, true).is_some()
}

fn efcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(effilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    let lp = duel.get_lp(tp) / 2;
    duel.pay_lp_cost(tp, lp as u32)?;
    Ok(false)
}

// Effect.SetLabel(to, from:GetLabel()), where no label reads as 0.
fn copy_label(duel: &mut Duel, from: EffectId, to: EffectId) {
    let labels = from.get_labels(duel).to_vec();
    to.set_labels(duel, if labels.is_empty() { &[0] } else { &labels });
}

fn eftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        let Some(te) = e.get_label_object(duel) else { return Ok(false) };
        return match te.get_target(duel) {
            Target::None => Ok(false),
            Target::Action(tg) => tg(duel, a, 0, Some(chkc)),
            _ => {
                unported!(duel, true);
                Ok(false)
            }
        };
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, effilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if duel.remove(g, POS_FACEUP, REASON_COST)? == 0 {
        return Ok(false);
    }
    let Some(te) = g.get_first().and_then(|sc| sc.check_activate_effect(duel, false, true)) else {
        // The Lua errors on the nil effect.
        unported!(duel, true);
        return Ok(false);
    };
    copy_label(duel, te, e);
    let label_object = te.get_label_object(duel);
    e.set_label_object(duel, label_object);
    match te.get_target(duel) {
        Target::None => {}
        Target::Action(tg) => {
            tg(duel, a, 1, None)?;
        }
        _ => unported!(duel, true),
    }
    copy_label(duel, e, te);
    let label_object = e.get_label_object(duel);
    te.set_label_object(duel, label_object);
    e.set_label_object(duel, Some(te));
    duel.clear_operation_info(0);
    Ok(false)
}

fn efop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(te) = e.get_label_object(duel) else { return Ok(()) };
    copy_label(duel, te, e);
    let label_object = te.get_label_object(duel);
    e.set_label_object(duel, label_object);
    let op = te.get_operation(duel);
    unported!(duel, !matches!(op, None | Some(Operation::Event(_) | Operation::Promptless(_))));
    if let Some(Operation::Event(op) | Operation::Promptless(op)) = op {
        op(duel, a)?;
    }
    copy_label(duel, e, te);
    let label_object = e.get_label_object(duel);
    te.set_label_object(duel, label_object);
    Ok(())
}
