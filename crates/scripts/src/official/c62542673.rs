// Destiny HERO - Doom Overlord
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 62542673;

pub const LISTED_SERIES: &[u16] = &[SET_DESTINY_HERO, SET_HERO];

const CLOCK_TOWER_PRISON: u32 = 75041269;
const DARK_CITY_AT_MIDNIGHT: u32 = 4663194;

pub const LISTED_NAMES: &[u32] = &[CLOCK_TOWER_PRISON, DARK_CITY_AT_MIDNIGHT];

// Copies in the hand to summon, Destiny HERO - Dreadmaster to send, Clock Tower Prison and Dark City at Midnight to add, and a target to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (40591390, DECK),
    (40591390, DECK),
    (CLOCK_TOWER_PRISON, DECK),
    (DARK_CITY_AT_MIDNIGHT, GRAVE),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct ReturnOp;

impl aux::DelayedOperation for ReturnOp {
    fn operate(duel: &mut Duel, rg: CardSet, a: &Args) -> Script<()> {
        aux::DefaultFieldReturnOp::operate(duel, rg, a)
    }

    // The turn count banop keeps in the third label, set only when it banished during a Standby Phase.
    fn condition(duel: &Duel, _: CardSet, a: &Args) -> bool {
        let &[_, _, turn_count] = a.e.get_labels(duel) else { return true };
        i64::from(duel.get_turn_count()) != turn_count
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_REMOVE);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, (ID, 0));
    e1a.set_target(duel, bantg);
    e1a.set_operation(duel, banop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, thcost);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_able_to_remove(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) {
        let (reset_count, turn_count) = if duel.is_phase(PHASE_STANDBY) {
            (2, Some(duel.get_turn_count()))
        } else {
            (1, None)
        };
        let e1 = aux::remove_until_full::<ReturnOp>(
            duel,
            tc,
            0,
            REASON_EFFECT,
            (PHASE_STANDBY, RESET_PHASE | PHASE_STANDBY),
            ID,
            e,
            tp,
            reset_count,
            0,
        )?;
        if let (Some(e1), Some(turn_count)) = (e1, turn_count)
            && let &[fid, flag] = e1.get_labels(duel)
        {
            e1.set_labels(duel, &[fid, flag, i64::from(turn_count)]);
        }
    }
    Ok(())
}

fn thcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DESTINY_HERO]) && c.is_monster(duel) && c.is_able_to_grave_as_cost(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, thcostfilter, LOCATION_DECK)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CLOCK_TOWER_PRISON) || c.is_code(duel, DARK_CITY_AT_MIDNIGHT)) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let filter = aux::necro_valley_filter(thfilter);
    if let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    {
        if sc.is_location(duel, LOCATION_GRAVE) {
            duel.hint_selection(sc);
        }
        duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)?;
        if sc.is_previous_location(duel, LOCATION_DECK) {
            duel.confirm_cards(1 - tp, sc)?;
        }
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, card| {
        !(card.is_attribute(duel, ATTRIBUTE_DARK) && card.is_set_card(duel, &[SET_HERO]))
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
