// Kewl Tune Loudness War
use engine::{EffectSet, Operation, Target};

use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 41069676;

// Tuners on the field for the Synchro Summon, Kewl Tune monsters in the Graveyard to apply, and the opponent's Dark Hole to
// chain to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (42781164, MZONE),
    (16509007, MZONE),
    (16509007, MZONE),
    (43904702, MZONE),
    (16509007, GRAVE),
    (42781164, GRAVE),
    (16387555, GRAVE),
    (43904702, GRAVE),
    (53129443, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

// Chain data slots: the card banished as cost, the bit mask of its own effects (by position) that can be applied, and the
// position of the chosen one.
const BANISHED: u8 = 0;
const AVAILABLE: u8 = 1;
const CHOSEN: u8 = 2;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_TUNER, Some(sc), sumtype, tp) || c.is_has_effect(duel, EFFECT_CAN_BE_TUNER).is_some()
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_target_range(duel, LOCATION_MZONE, 0);
    e1a.set_target_card(duel, |duel, e, c| c.is_type(duel, TYPE_TUNER) && c != e.get_handler(duel));
    e1a.set_value(duel, 1);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1b.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1b.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |_, a| a.rp == 1 - a.tp);
    e2.set_cost(duel, applycost);
    e2.set_target(duel, applytg);
    e2.set_operation(duel, applyop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_code(duel, EFFECT_MULTIPLE_TUNERS);
    c.register_effect(duel, e3);
}

// The parameters the Lua passes to the applied effect's target and operation: an empty group, no player and REASON_EFFECT.
fn args(e: EffectId, tp: u8, re: EffectId) -> Args {
    Args {
        e,
        tp,
        eg: Some(CardSet::default()),
        ep: PLAYER_NONE,
        re: Some(re),
        r: REASON_EFFECT,
        rp: PLAYER_NONE,
        ..Args::default()
    }
}

fn applicable(duel: &Duel, eff: EffectId, e: EffectId, tp: u8) -> bool {
    eff.get_code(duel) == EVENT_BE_MATERIAL
        && match eff.get_target(duel) {
            Target::None => true,
            Target::Action(f) => duel.check_mut(|duel| f(duel, &args(eff, tp, e), 0, None)).unwrap_or(false),
            _ => {
                unported!(duel, true);
                false
            }
        }
}

fn rmfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_KEWL_TUNE])
        && c.is_monster(duel)
        && c.is_able_to_remove_as_cost(duel)
        && c.get_own_effects(duel).into_iter().any(|eff| applicable(duel, eff, e, tp))
}

fn applycost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| rmfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, |c, duel| rmfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    let Some(rc) = g.get_first() else { return Ok(false) };
    duel.remove(rc, POS_FACEUP, REASON_COST)?;
    let available = rc
        .get_own_effects(duel)
        .into_iter()
        .enumerate()
        .filter(|&(_, eff)| applicable(duel, eff, e, tp))
        .fold(0, |mask, (position, _)| mask | 1 << position);
    e.set_chain_data_group(duel, BANISHED, rc.into());
    e.set_chain_data(duel, AVAILABLE, available);
    Ok(false)
}

fn banished_effects(duel: &Duel, e: EffectId) -> EffectSet {
    e.get_chain_data_group(duel, BANISHED)
        .and_then(|g| g.get_first())
        .map(|rc| rc.get_own_effects(duel))
        .unwrap_or_default()
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
    let mask = e.get_chain_data(duel, AVAILABLE).unwrap_or(0);
    let available: Vec<(usize, EffectId)> = banished_effects(duel, e)
        .into_iter()
        .enumerate()
        .filter(|&(position, _)| mask >> position & 1 == 1)
        .collect();
    // The Lua indexes a nil effect where no effect can be applied after the banish.
    unported!(duel, available.is_empty());
    let chosen = if available.len() > 1 {
        let descriptions: Vec<u64> = available.iter().map(|(_, eff)| eff.get_description(duel)).collect();
        available.get(usize::try_from(duel.select_option(tp, &descriptions)?).unwrap_or(0))
    } else {
        available.first()
    };
    let Some(&(position, eff)) = chosen else { return Ok(false) };
    duel.hint(HINT_OPSELECTED, 1 - tp, eff.get_description(duel));
    e.set_label(duel, eff.get_label(duel));
    e.set_label_object(duel, eff.get_label_object(duel));
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
    e.set_category(duel, 0);
    duel.clear_operation_info(0);
    e.set_chain_data(duel, CHOSEN, position as i64);
    Ok(false)
}

fn applyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(position) = e.get_chain_data(duel, CHOSEN) else {
        return Ok(());
    };
    let eff = banished_effects(duel, e).get(usize::try_from(position).unwrap_or(0)).copied();
    let op = eff.and_then(|eff| eff.get_operation(duel));
    unported!(
        duel,
        op.is_some_and(|op| !matches!(op, Operation::Event(_) | Operation::Promptless(_)))
    );
    if let Some(Operation::Event(f) | Operation::Promptless(f)) = op {
        f(duel, &args(e, tp, e))?;
    }
    Ok(())
}
