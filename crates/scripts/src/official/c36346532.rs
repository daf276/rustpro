// Paleozoic Cambroraster
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 36346532;

pub const LISTED_SERIES: &[u16] = &[SET_PALEOZOIC];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_PALEOZOIC], Some(lc), sumtype, tp)
    }
}

// On the field with Set Paleozoic Traps to send, Paleozoic Canadia in the Deck to Set, and Mystical Space Typhoon of the opponent to destroy a Set card.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (98414735, DECK),
    (98414735, DECK),
    (98414735, DECK),
    (64765016, SZONE),
    (64765016, SZONE),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Replaced];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Effect(|duel, e, re| re.is_monster_effect(duel) && re.get_owner(duel) != e.get_owner(duel)),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, gytg);
    e2.set_operation(duel, gyop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_range(duel, LOCATION_GRAVE | LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, reptg);
    e3.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e3.set_operation(duel, repop);
    c.register_effect(duel, e3);
}

fn gyfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    if c.is_faceup(duel) {
        return false;
    }
    let zc = if c.is_controler(duel, tp) && c.get_sequence(duel) < 5 {
        -1
    } else {
        0
    };
    duel.get_location_count(tp, LOCATION_SZONE) > zc
}

fn setfilter(c: CardId, duel: &Duel, ignore: bool) -> bool {
    c.is_set_card(duel, &[SET_PALEOZOIC]) && c.is_trap(duel) && c.is_ssetable_full(duel, ignore, None)
}

fn gytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_SZONE) && gyfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| gyfilter(c, duel, tp), tp, LOCATION_SZONE, LOCATION_SZONE, 1, None)
                && duel.is_existing_matching_card(|c, duel| setfilter(c, duel, true), tp, LOCATION_DECK, 0, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, |c, duel| gyfilter(c, duel, tp), tp, LOCATION_SZONE, LOCATION_SZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn gyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && duel.send_to_grave(tc, REASON_EFFECT)? > 0
        && duel
            .get_operated_group()
            .get_first()
            .is_some_and(|oc| oc.is_location(duel, LOCATION_GRAVE))
        && duel.get_location_count(tp, LOCATION_SZONE) > 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let g = duel.select_matching_card(tp, |c, duel| setfilter(c, duel, false), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first() {
            duel.sset(tp, sc)?;
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_description(duel, aux::stringid(ID, 0));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
            e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            sc.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp)
        && c.is_on_field(duel)
        && c.is_facedown(duel)
        && !c.is_reason(duel, REASON_REPLACE)
        && c.is_reason(duel, REASON_EFFECT)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_status(duel, STATUS_DESTROY_CONFIRMED)
            && c.is_able_to_remove(duel)
            && a.eg
                .unwrap_or_default()
                .is_exists(duel, |rc, duel| repfilter(rc, duel, tp), 1, None));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.remove(a.e.get_handler(duel), POS_FACEUP, REASON_EFFECT | REASON_REPLACE)
        .map(drop)
}
