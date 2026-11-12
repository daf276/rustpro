// Shinobi Necro
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 90020780;

// On the field and in the Graveyard, with Monster Reborn and Skull Servants in the Graveyard to revive, and Soul Release to
// banish it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (83764718, HAND),
    (83764718, HAND),
    (5758500, HAND),
    (32274490, GRAVE),
    (32274490, GRAVE),
];

pub const CHECKS: &[Check] = &[Check::Resolves, Check::CardHint(3300)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_HANDES);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, dcondition);
    e1.set_target(duel, dtarget);
    e1.set_operation(duel, doperation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcondition);
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, spoperation);
    c.register_effect(duel, e2);
}

fn dfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_location(duel, LOCATION_GRAVE) && c.is_previous_controler(duel, tp) && c.is_race(duel, RACE_ZOMBIE)
}

fn dcondition(duel: &Duel, a: &Args) -> bool {
    let (c, eg) = (a.e.get_handler(duel), a.eg.unwrap_or_default());
    !eg.is_contains(c) && eg.is_exists(duel, |tc, duel| dfilter(tc, duel, a.tp), 1, Some(c.into()))
}

fn dtarget(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn doperation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.draw(p, d, REASON_EFFECT)? != 0 {
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn spcondition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_GRAVE)
        && (c.is_reason(duel, REASON_EFFECT)
            || (c.is_reason(duel, REASON_COST) && a.re.is_some_and(|re| re.is_has_type(duel, !EFFECT_TYPE_FIELD))))
}

fn spoperation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}
