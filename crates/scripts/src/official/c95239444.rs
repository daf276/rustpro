// Orea, the Sylvan High Arbiter
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 95239444;

// On the field with Xyz Import to attach Sangan, Level 7 Normal Monsters (Gaia The Fierce Knight) to Xyz Summon it, and Plant monsters
// (Snakeyashi, Bean Soldier) in the hand to discard and to excavate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (6368038, MZONE),
    (6368038, MZONE),
    (29802344, HAND),
    (29802344, HAND),
    (84990171, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, stcost);
    e1.set_operation(duel, stop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_race(duel, RACE_PLANT)
        && c.is_level_below(duel, lv)
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_able_to_grave_as_cost(duel)
}

fn stcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ct = duel.get_field_group_count(tp, LOCATION_DECK, 0) as u32;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| cfilter(c, duel, ct), tp, LOCATION_MZONE | LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| cfilter(c, duel, ct),
        tp,
        LOCATION_MZONE | LOCATION_HAND,
        0,
        1,
        1,
        None,
    )?;
    // Lua errors on an empty selection
    unported!(duel, g.is_empty());
    let Some(tc) = g.get_first() else { return Ok(false) };
    e.set_label(duel, i64::from(tc.get_level(duel)));
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn stop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = a.e.get_label(duel) as usize;
    if duel.get_field_group_count(tp, LOCATION_DECK, 0) >= ct {
        duel.sort_decktop(tp, tp, ct as u16)?;
    }
    Ok(())
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, 1)
            && duel.is_existing_matching_card(
                CardId::is_able_to_hand,
                tp,
                LOCATION_ONFIELD,
                LOCATION_ONFIELD,
                1,
                Some(e.get_handler(duel).into()),
            ));
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !duel.is_player_can_discard_deck(tp, 1) {
        return Ok(());
    }
    let ct = duel.get_field_group_count(tp, LOCATION_DECK, 0).min(3);
    if ct == 0 {
        return Ok(());
    }
    // Lua passes the arguments swapped; the core truncates the hint type to a byte
    duel.hint(HINTMSG_NUMBER as u8, tp, u64::from(HINT_NUMBER));
    let mut ac = duel.announce_number_range(tp, 1, ct as u32)? as usize;
    duel.confirm_decktop(tp, ac)?;
    let g = duel.get_decktop_group(tp, ac);
    let sg = g.filter(duel, |c, duel| c.is_race(duel, RACE_PLANT), None);
    duel.disable_shuffle_check(true);
    if duel.send_to_grave(sg, REASON_EFFECT + REASON_EXCAVATE)? != 0 {
        let ct = duel
            .get_operated_group()
            .filter_count(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
        if ct > 0 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
            let tg = duel.select_matching_card(
                tp,
                CardId::is_able_to_hand,
                tp,
                LOCATION_ONFIELD,
                LOCATION_ONFIELD,
                1,
                sg.len() as u8,
                Some(c.into()),
            )?;
            if !tg.is_empty() {
                duel.break_effect()?;
                duel.disable_shuffle_check(false);
                duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
            }
        }
    }
    ac -= sg.len();
    if ac > 0 {
        duel.sort_decktop(tp, tp, ac as u16)?;
        for _ in 0..ac {
            if let Some(tc) = duel.get_decktop_group(tp, 1).get_first() {
                duel.move_sequence(tc, 1)?;
            }
        }
    }
    Ok(())
}
