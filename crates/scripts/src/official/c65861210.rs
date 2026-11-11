// Fallen Paradise of the Sacred Beasts
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 65861210;
const LOCATIONS_HAND_DECK_GRAVE_REMOVED: u32 = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED;

// A Level 10 Sacred Beast monster on the field for the draw, and three monsters on the field and in the hand to send,
// with Sacred Beast monsters in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, 0),
    (96345184, MZONE),
    (59138498, MZONE),
    (59138498, HAND),
    (23856331, HAND),
    (50251045, DECK),
    (59138498, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_count_limit(duel, 3, 0);
    e1.set_target(duel, gytg);
    e1.set_operation(duel, gyop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, drcon);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
}

fn gyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_grave(duel) && (c.is_faceup(duel) || !c.is_on_field(duel))
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (
        sg.get_class_count(duel, |c, duel| i64::from(c.get_main_card_type(duel))) == 1,
        false,
    )
}

fn gytg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let g = duel.get_matching_group(gyfilter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, Some(e.get_handler(duel).into()));
        return Ok(aux::select_unselect_group_check(duel, g, 3, 3, &rescon));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, e.get_description(duel));
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 3, tp, (LOCATION_HAND | LOCATION_ONFIELD) as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATIONS_HAND_DECK_GRAVE_REMOVED as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SACRED_BEAST])
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_REMOVED))
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, true, POS_FACEUP, tp, 0xff)
}

fn gyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(gyfilter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, Some(c.into()));
    if g.len() < 3 {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 3, 3, &rescon, tp, HINTMSG_TOGRAVE)?;
    let necro_spfilter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    if sg.len() == 3
        && duel.send_to_grave(sg, REASON_EFFECT)? == 3
        && sg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), 1, None)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(necro_spfilter, tp, LOCATIONS_HAND_DECK_GRAVE_REMOVED, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(
            tp,
            |c, duel| spfilter(c, duel, e, tp),
            tp,
            LOCATIONS_HAND_DECK_GRAVE_REMOVED,
            0,
            1,
            1,
            None,
        )?;
        let Some(sc) = g.get_first() else {
            return Ok(());
        };
        duel.break_effect()?;
        if duel.special_summon_step(sc, 0, tp, tp, false, true, POS_FACEUP)? {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, aux::stringid(ID, 3));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
            e1.set_value(
                duel,
                Value::Effect(|duel, e, te| {
                    te.get_owner_player(duel) != e.get_handler_player(duel) && te.is_activated(duel) && te.is_spell_trap_effect(duel)
                }),
            );
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            sc.register_effect(duel, e1);
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}

fn drconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SACRED_BEAST]) && c.is_original_level(duel, 10) && c.is_faceup(duel)
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(drconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 2));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, a.e.get_description(duel));
    duel.set_target_player(tp);
    duel.set_target_param(2);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    Ok(false)
}
