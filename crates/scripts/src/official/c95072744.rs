// Tohushaddoll Grysta
use engine::{Operation, Target};

use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 95072744;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_SHADDOLL])
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_able_to_remove(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        if duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_some() {
            return (None, None);
        }
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_remove);
        (Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None)), None)
    }

    fn extraop(duel: &mut Duel, e: EffectId, tc: CardId, tp: u8, sg: &mut CardSet) -> Script<()> {
        Fusion::banish_material(duel, e, tc, tp, sg)
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_operation_info(
                0,
                CATEGORY_REMOVE,
                None,
                1,
                a.tp,
                (LOCATION_HAND | LOCATION_ONFIELD | LOCATION_GRAVE) as i32,
            );
        }
        Ok(true)
    }
}

// Face-down on the field, to be flipped by an attack and sent to the Graveyard by Dark Hole drawn later, with Shaddoll
// monsters in the Graveyard to apply or banish for El Shaddoll Apkallone and Winda in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (30328508, GRAVE),
    (3717252, GRAVE),
    (77723643, GRAVE),
    (66675911, GRAVE),
    (50907446, EXTRA),
    (94977269, EXTRA),
    (53129443, DECK),
    (53129443, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(50907446),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, applytg);
    e1.set_operation(duel, applyop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_reason(duel, REASON_EFFECT) && !duel.is_phase(PHASE_DAMAGE)
    });
    e2.set_cost(duel, fusioncost);
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| !c.is_summon_location(duel, LOCATION_EXTRA) || c.is_set_card(duel, &[SET_SHADDOLL])),
    );
}

// The arguments the Lua passes to an effect's target or operation: an empty group, no player and the applying effect.
fn flip_args(eff: EffectId, tp: u8, e: EffectId) -> Args {
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

fn available_effects(duel: &Duel, c: CardId, e: EffectId, tp: u8) -> Vec<EffectId> {
    c.get_own_effects(duel)
        .into_iter()
        .filter(|&eff| {
            (eff.is_has_type(duel, EFFECT_TYPE_FLIP) || eff.get_code(duel) == EVENT_FLIP)
                && match eff.get_target(duel) {
                    Target::None => true,
                    Target::Action(f) => duel.check_mut(|duel| f(duel, &flip_args(eff, tp, e), 0, None)).unwrap_or(false),
                    _ => {
                        unported!(duel, true);
                        false
                    }
                }
        })
        .collect()
}

fn applyfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    !c.is_code(duel, ID)
        && !c.is_race(duel, RACE_ROCK)
        && c.is_set_card(duel, &[SET_SHADDOLL])
        && c.is_monster(duel)
        && !available_effects(duel, c, e, tp).is_empty()
}

fn applytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| applyfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let Some(tc) = duel
        .select_target(tp, |c, duel| applyfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .and_then(|g| g.get_first())
    else {
        return Ok(false);
    };
    let effs = available_effects(duel, tc, e, tp);
    let eff = if effs.len() > 1 {
        let descriptions: Vec<u64> = effs.iter().map(|eff| eff.get_description(duel)).collect();
        let op = duel.select_option(tp, &descriptions)?;
        usize::try_from(op).ok().and_then(|op| effs.get(op).copied())
    } else {
        effs.first().copied()
    };
    let Some(eff) = eff else { return Ok(false) };
    duel.hint(HINT_OPSELECTED, 1 - tp, eff.get_description(duel));
    duel.clear_target_card();
    tc.create_effect_relation(duel, e);
    e.set_label(duel, eff.get_label(duel));
    e.set_label_object(duel, eff.get_label_object(duel));
    match eff.get_target(duel) {
        Target::None => {}
        Target::Action(f) => {
            f(duel, a, 1, None)?;
            eff.set_label(duel, e.get_label(duel));
            eff.set_label_object(duel, e.get_label_object(duel));
        }
        _ => unported!(duel, true),
    }
    e.set_label_object(duel, Some(eff));
    duel.clear_operation_info(0);
    Ok(false)
}

fn applyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(eff) = e.get_label_object(duel) else { return Ok(()) };
    if !eff.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    e.set_label(duel, eff.get_label(duel));
    e.set_label_object(duel, eff.get_label_object(duel));
    let op = eff.get_operation(duel);
    unported!(
        duel,
        op.is_some_and(|op| !matches!(op, Operation::Event(_) | Operation::Promptless(_)))
    );
    if let Some(Operation::Event(f) | Operation::Promptless(f)) = op {
        f(duel, &flip_args(e, a.tp, e))?;
    }
    e.set_label(duel, 0);
    e.set_label_object(duel, None);
    Ok(())
}

fn fusioncost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_set_card(duel, &[SET_SHADDOLL])
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}
