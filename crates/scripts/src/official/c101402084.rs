// Kumenyo, the Nine-Hued Deer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 101402084;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> =
        Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

// On the field with Xyz Import to attach Sangan, in the Extra Deck with Level 4 DARK and LIGHT materials (Dark Blade, Mystical Elf),
// and cards to attach in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (11321183, MZONE),
    (15025844, MZONE),
    (26202165, GRAVE),
    (44095762, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, |duel, a, chk| {
        Cost::detach_from_self_op::<1, 1>(duel, a, chk, |duel, e, og| {
            if let Some(mc) = og.get_first() {
                mc.create_effect_relation(duel, e);
            }
        })
    });
    e1.set_target(duel, attachtg);
    e1.set_operation(duel, attachop);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| race_count(duel, a.e) >= 2);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EFFECT_DRAW_COUNT);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 1, 0);
    e3.set_condition(duel, |duel, a| race_count(duel, a.e) >= 3);
    e3.set_value(duel, 2);
    c.register_effect(duel, e3);
}

fn race_count(duel: &Duel, e: EffectId) -> usize {
    e.get_handler(duel)
        .get_overlay_group(duel)
        .get_bin_class_count(duel, CardId::get_race)
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(c.is_xyz_monster(duel)
            && duel.is_existing_matching_card(
                |mc, duel| mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT),
                tp,
                LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED,
                0,
                1,
                None,
            ));
    }
    duel.set_possible_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let detached_material = e.get_chain_data_group(duel, COST_DETACHED_MATERIALS).and_then(|og| og.get_first());
    let except = detached_material.filter(|mc| mc.is_relate_to_effect(duel, e));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let filter = aux::necro_valley_filter(|mc, duel| mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT));
    let g = duel.select_matching_card(
        tp,
        filter,
        tp,
        LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        except.map(CardSet::from),
    )?;
    if let Some(sc) = g.get_first() {
        if sc.is_location(duel, LOCATION_HAND) {
            duel.confirm_cards(1 - tp, sc)?;
        } else {
            duel.hint_selection(sc);
        }
        duel.overlay(c, sc)?;
    }
    Ok(())
}
