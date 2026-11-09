// Chasma, Dragon Ruler of Auroras
use engine::{Condition, Operation, Target};

use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 4965193;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_DRAGON, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

// Xyz Summoned from Red-Eyes Black Dragons, with Tidal (a Level 7 Dragon Ruler with a discard effect) in the Deck and the Graveyard
// and the opponent's Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (74677422, MZONE),
    (74677422, MZONE),
    (74677422, MZONE),
    (26400609, DECK),
    (26400609, DECK),
    (26400609, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[applycost, Cost::detach_from_self::<1, 1>])
    });
    e1.set_target(duel, applytg);
    e1.set_operation(duel, applyop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

// The parameters runfn passes: an empty group, no player, no effect and REASON_EFFECT.
fn runfn_args(eff: EffectId, tp: u8) -> Args {
    Args {
        e: eff,
        tp,
        eg: Some(CardSet::default()),
        ep: PLAYER_NONE,
        r: REASON_EFFECT,
        rp: PLAYER_NONE,
        ..Args::default()
    }
}

fn runfn(duel: &Duel, eff: EffectId, tp: u8) -> bool {
    let a = runfn_args(eff, tp);
    let condition = match eff.get_condition(duel) {
        None => true,
        Some(Condition::Check(f)) => f(duel, &a),
        Some(Condition::Mut(f)) => duel.check_mut(|duel| f(duel, &a)),
    };
    condition
        && match eff.get_target(duel) {
            Target::None => true,
            Target::Action(f) => duel.check_mut(|duel| f(duel, &a, 0, None)).unwrap_or(false),
            _ => {
                unported!(duel, true);
                false
            }
        }
}

fn applyfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DRAGON_RULER])
        && c.is_level(duel, &[7])
        && c.is_able_to_grave_as_cost(duel)
        && c.get_own_effects(duel)
            .into_iter()
            .any(|eff| eff.has_self_discard_cost(duel) && runfn(duel, eff, tp))
}

fn applycost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| applyfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| applyfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(false) };
    let effs: Vec<EffectId> = sc
        .get_own_effects(duel)
        .into_iter()
        .filter(|eff| eff.has_self_discard_cost(duel))
        .collect();
    let options: Vec<(bool, u64)> = effs.iter().map(|&eff| (runfn(duel, eff, tp), eff.get_description(duel))).collect();
    let op = if options.len() == 1 {
        Some(1)
    } else {
        duel.select_effect(tp, &options)?
    };
    let te = op.and_then(|op| effs.get(op - 1).copied());
    a.e.set_label_object(duel, te);
    duel.send_to_grave(sc, REASON_COST)?;
    Ok(false)
}

fn applytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let e = a.e;
    let te = e.get_label_object(duel);
    if let Some(chkc) = chkc {
        return Ok(match te.map(|te| te.get_target(duel)) {
            Some(Target::Action(f)) => f(duel, a, 0, Some(chkc))?,
            _ => false,
        });
    }
    if chk == 0 {
        return Ok(true);
    }
    let Some(te) = te else { return Ok(false) };
    e.set_label(duel, te.get_label(duel));
    e.set_label_object(duel, te.get_label_object(duel));
    e.set_property(
        duel,
        if te.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
            EFFECT_FLAG_CARD_TARGET
        } else {
            0
        },
    );
    match te.get_target(duel) {
        Target::None => {}
        Target::Action(f) => {
            f(duel, a, 1, None)?;
            te.set_label(duel, e.get_label(duel));
            te.set_label_object(duel, e.get_label_object(duel));
            duel.clear_operation_info(0);
        }
        _ => unported!(duel, true),
    }
    e.set_label_object(duel, Some(te));
    Ok(false)
}

fn applyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(te) = e.get_label_object(duel) else { return Ok(()) };
    let op = te.get_operation(duel);
    unported!(
        duel,
        op.is_some_and(|op| !matches!(op, Operation::Event(_) | Operation::Promptless(_)))
    );
    if let Some(Operation::Event(f) | Operation::Promptless(f)) = op {
        e.set_label(duel, te.get_label(duel));
        e.set_label_object(duel, te.get_label_object(duel));
        f(duel, a)?;
    }
    e.set_property(duel, 0);
    e.set_label(duel, 0);
    e.set_label_object(duel, None);
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_BATTLE) || (a.rp == 1 - a.tp && c.is_reason(duel, REASON_EFFECT) && c.is_previous_controler(duel, a.tp))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DRAGON_RULER])
        && c.is_level(duel, &[7])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && c.is_faceup(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp),
                tp,
                LOCATION_GRAVE | LOCATION_REMOVED,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_GRAVE | LOCATION_REMOVED) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
