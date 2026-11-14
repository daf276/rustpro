// Clown Crew Cappello
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, PZONE};

const ID: u32 = 19491080;

pub const LISTED_SERIES: &[u16] = &[SET_CLOWN_CREW];
// Copies in the Pendulum Zone and on the field (to Tribute for Emissary from Pandemonium in the hand), and non-Pendulum Clown Crew
// monsters in the Deck and the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (42685062, HAND),
    (42685062, HAND),
    (42759961, DECK),
    (82159583, DECK),
    (31533473, EXTRA),
    (93172951, EXTRA),
    (91237821, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_target(duel, gytg);
    e1.set_operation(duel, gyop);
    e1.set_count_limit(duel, 1, (ID, 0));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            CardId::is_tribute_summoned,
            a.e.get_handler_player(duel),
            LOCATION_MZONE,
            0,
            1,
            None,
        )
    });
    e2.set_value(duel, -1500);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_RELEASE);
    e3.set_target(duel, efftg);
    e3.set_operation(duel, effop);
    c.register_effect(duel, e3);
}

fn gyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CLOWN_CREW]) && !c.is_pendulum_monster(duel) && c.is_able_to_grave(duel)
}

fn gytg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(gyfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn gyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, gyfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_grave(g, REASON_EFFECT)? > 0 && c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::EffectPlayer(actval));
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
    duel.register_effect(e1, tp);
    Ok(())
}

fn actval(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    re.is_monster_effect(duel) && rc.is_location(duel, LOCATION_MZONE) && rc.is_summon_location(duel, LOCATION_DECK | LOCATION_EXTRA)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if c.is_location(duel, LOCATION_DECK) && duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return false;
    }
    if c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, None, c) <= 0 {
        return false;
    }
    c.is_set_card(duel, &[SET_CLOWN_CREW])
        && c.is_monster(duel)
        && !c.is_pendulum_monster(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_pendulum_monster(duel) && c.is_faceup(duel) && c.is_able_to_deck(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let both = LOCATION_MZONE | LOCATION_EXTRA;
    let b1 = !duel.has_flag_effect(tp, ID) && duel.is_existing_matching_card(tdfilter, tp, both, both, 1, None);
    let b2 = !duel.has_flag_effect(tp, ID + 1)
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_TODECK);
        duel.set_operation_info(0, CATEGORY_TODECK, None, 1, PLAYER_ALL, both as i32);
    } else if op == Some(2) {
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let both = LOCATION_MZONE | LOCATION_EXTRA;
    match e.get_label(duel) {
        1 => {
            let g = duel.get_matching_group(tdfilter, tp, both, both, None);
            if !g.is_empty() {
                duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(
                tp,
                |c, duel| spfilter(c, duel, e, tp),
                tp,
                LOCATION_DECK | LOCATION_EXTRA,
                0,
                1,
                1,
                None,
            )?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, true, false, POS_FACEUP)?;
            }
        }
        _ => {}
    }
    Ok(())
}
