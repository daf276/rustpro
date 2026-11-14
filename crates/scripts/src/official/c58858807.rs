// Tellarknight Constellar Caduceus
use engine::{Condition, Operation, Target};

use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 58858807;

pub const LISTED_SERIES: &[u16] = &[SET_TELLARKNIGHT, SET_CONSTELLAR];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

// In the Extra Deck, Xyz Summoned with Constellar Castors, which are also in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (33302589, MZONE),
    (33302589, MZONE),
    (33302589, MZONE),
    (33302589, GRAVE),
    (33302589, GRAVE),
];
// No other Constellar or tellarknight monster is ported, so no banished monster has an effect to apply that a scenario can reach.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[applycost, Cost::detach_from_self::<1, 1>])
    });
    e2.set_target(duel, applytg);
    e2.set_operation(duel, applyop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_set_card(duel, &[SET_TELLARKNIGHT, SET_CONSTELLAR]) && c.is_able_to_hand(duel) && c.is_can_be_effect_target(duel, e)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let has = |setcode| sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[setcode]), 1, None);
    (sg.len() == 1 || (has(SET_TELLARKNIGHT) && has(SET_CONSTELLAR)), false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| thfilter(c, duel, e), tp, LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    let tg = aux::select_unselect_group(duel, g, 1, 2, &rescon, tp, HINTMSG_ATOHAND)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(tg), tg.len() as u8, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

// The parameters of the call to an effect's condition and target: an empty group, no player, `e` as the effect and
// REASON_EFFECT.
fn runfn_args(eff: EffectId, tp: u8, e: EffectId) -> Args {
    Args {
        e: eff,
        tp,
        eg: Some(CardSet::default()),
        ep: PLAYER_NONE,
        re: Some(e),
        r: REASON_EFFECT,
        rp: PLAYER_NONE,
        ..Args::default()
    }
}

fn runfn(duel: &Duel, eff: EffectId, tp: u8, e: EffectId) -> bool {
    let a = runfn_args(eff, tp, e);
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

// The own effects of `c` that activate on being Normal Summoned and can be applied.
fn available_effects(duel: &Duel, c: CardId, tp: u8, e: EffectId) -> Vec<(usize, EffectId)> {
    c.get_own_effects(duel)
        .into_iter()
        .enumerate()
        .filter(|&(_, eff)| eff.get_code(duel) == EVENT_SUMMON_SUCCESS && runfn(duel, eff, tp, e))
        .collect()
}

fn rmfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_TELLARKNIGHT, SET_CONSTELLAR])
        && c.is_monster(duel)
        && c.is_able_to_remove_as_cost(duel)
        && !available_effects(duel, c, tp, e).is_empty()
}

// The effects the Lua keeps in a table as the label object are the own effects of the banished card whose bit is set in
// the label, the card being the label object group.
fn applycost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let locations = LOCATION_HAND | LOCATION_DECK;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| rmfilter(c, duel, e, tp), tp, locations, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, |c, duel| rmfilter(c, duel, e, tp), tp, locations, 0, 1, 1, None)?;
    let Some(rc) = g.get_first() else { return Ok(false) };
    duel.remove(rc, POS_FACEUP, REASON_COST)?;
    let mask = available_effects(duel, rc, tp, e)
        .into_iter()
        .fold(0, |mask, (index, _)| mask | 1 << index);
    e.set_label_object(duel, None);
    e.set_label_object_group(duel, Some(rc.into()));
    e.set_label(duel, mask);
    Ok(false)
}

fn applytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(match e.get_label_object(duel).map(|eff| eff.get_target(duel)) {
            Some(Target::Action(f)) => f(duel, a, 0, Some(chkc))?,
            _ => false,
        });
    }
    if chk == 0 {
        return Ok(true);
    }
    let rc = e.get_label_object_group(duel).and_then(|g| g.get_first());
    let mask = e.get_label(duel);
    let available: Vec<EffectId> = rc
        .map(|rc| rc.get_own_effects(duel).into_iter().enumerate())
        .into_iter()
        .flatten()
        .filter(|&(index, _)| mask >> index & 1 != 0)
        .map(|(_, eff)| eff)
        .collect();
    let eff = if available.len() > 1 {
        let descriptions: Vec<u64> = available.iter().map(|eff| eff.get_description(duel)).collect();
        let op = duel.select_option(tp, &descriptions)?;
        usize::try_from(op).ok().and_then(|op| available.get(op).copied())
    } else {
        available.first().copied()
    };
    let Some(eff) = eff else { return Ok(false) };
    duel.hint(HINT_OPSELECTED, 1 - tp, eff.get_description(duel));
    e.set_label(duel, eff.get_label(duel));
    e.set_label_object(duel, eff.get_label_object(duel));
    e.set_label_object_group(duel, None);
    e.set_property(
        duel,
        if eff.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
            EFFECT_FLAG_CARD_TARGET
        } else {
            0
        },
    );
    match eff.get_target(duel) {
        Target::None => {}
        Target::Action(f) => {
            f(duel, a, 1, None)?;
        }
        _ => unported!(duel, true),
    }
    eff.set_label(duel, e.get_label(duel));
    eff.set_label_object(duel, e.get_label_object(duel));
    e.set_label_object(duel, Some(eff));
    duel.clear_operation_info(0);
    Ok(false)
}

fn applyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(eff) = e.get_label_object(duel) else { return Ok(()) };
    e.set_label(duel, eff.get_label(duel));
    e.set_label_object(duel, eff.get_label_object(duel));
    let op = eff.get_operation(duel);
    unported!(
        duel,
        op.is_some_and(|op| !matches!(op, Operation::Event(_) | Operation::Promptless(_)))
    );
    if let Some(Operation::Event(f) | Operation::Promptless(f)) = op {
        f(duel, &runfn_args(e, tp, e))?;
    }
    e.set_label(duel, 0);
    e.set_label_object(duel, None);
    Ok(())
}
