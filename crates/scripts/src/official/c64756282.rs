// Witchcrafter Genni
use engine::{Operation, Target};

use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 64756282;

pub const LISTED_SERIES: &[u16] = &[SET_WITCHCRAFTER];
pub const LISTED_NAMES: &[u32] = &[ID];

// With Witchcrafter Creation in the GY to copy and in the hand to discard; no other Witchcrafter monster is ported to Special Summon from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, GRAVE), (ID, DECK), (57916305, GRAVE), (57916305, HAND)];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, Witchcrafter::tribute_and_discard_cost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_WITCHCRAFTER]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(duel.get_mzone_count(tp, Some(c.into())) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}

fn copyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WITCHCRAFTER])
        && c.is_spell(duel)
        && c.is_able_to_remove_as_cost(duel)
        && c.check_activate_effect(duel, true, true).is_some()
        && c.check_activate_effect(duel, true, true)
            .is_some_and(|te| te.get_operation(duel).is_some())
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_remove_as_cost(duel) && duel.is_existing_matching_card(copyfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let Some(sc) = duel
        .select_matching_card(tp, copyfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(false);
    };
    duel.remove(CardSet::from_iter([sc, c]), POS_FACEUP, REASON_COST)?;
    let Some(te) = sc.check_activate_effect(duel, true, true) else {
        duel.reach_unported(concat!(
            file!(),
            ":",
            line!(),
            " (the copied Spell has no activation, where the Lua errors)"
        ));
        return Ok(false);
    };
    duel.clear_target_card();
    let tg = te.get_target(duel);
    let (property, property2) = te.get_property_full(duel);
    e.set_property_full(duel, property, property2);
    match tg {
        Target::Action(tg) => {
            tg(
                duel,
                &Args {
                    e,
                    tp,
                    ep: PLAYER_NONE,
                    rp: PLAYER_NONE,
                    ..Args::default()
                },
                1,
                None,
            )?;
        }
        other => unported!(duel, !matches!(other, Target::None)),
    }
    let label_object = e.get_label_object(duel);
    te.set_label_object(duel, label_object);
    e.set_label_object(duel, Some(te));
    e.set_category(duel, 0);
    duel.clear_operation_info(0);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(te) = a.e.get_label_object(duel) else { return Ok(()) };
    let label_object = te.get_label_object(duel);
    a.e.set_label_object(duel, label_object);
    let op = te.get_operation(duel);
    unported!(duel, !matches!(op, None | Some(Operation::Event(_) | Operation::Promptless(_))));
    if let Some(Operation::Event(op) | Operation::Promptless(op)) = op {
        op(duel, a)?;
    }
    Ok(())
}
