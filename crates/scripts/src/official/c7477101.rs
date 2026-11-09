// Seventh Tachyon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 7477101;

// Number 106: Giant Hand to reveal, and Prevent Rats, which share its Level and Attribute, in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (63746411, EXTRA), (549481, DECK), (549481, DECK), (549481, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, |duel, a, _| {
        a.e.set_label(duel, -1);
        Ok(true)
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn revfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    if !(c.is_set_card(duel, &[SET_NUMBER]) && !c.is_public(duel) && c.is_type(duel, TYPE_XYZ)) {
        return false;
    }
    c.xyz_number(duel).is_some_and(|number| (101..=107).contains(&number))
        && duel.is_existing_matching_card(
            |sc, duel| thfilter(sc, duel, c.get_race(duel), c.get_attribute(duel), c.get_rank(duel)),
            tp,
            LOCATION_DECK,
            0,
            1,
            None,
        )
}

fn thfilter(c: CardId, duel: &Duel, race: u64, attr: u32, rank: u32) -> bool {
    (c.is_race(duel, race) || c.is_attribute(duel, attr)) && c.is_level(duel, &[rank]) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let res =
            e.get_label(duel) == -1 && duel.is_existing_matching_card(|c, duel| revfilter(c, duel, tp), tp, LOCATION_EXTRA, 0, 1, None);
        e.set_label(duel, 0);
        return Ok(res);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let Some(rc) = duel
        .select_matching_card(tp, |c, duel| revfilter(c, duel, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(false);
    };
    // An effect holds two labels, so the Attribute and Rank share the second.
    e.set_labels(
        duel,
        &[
            rc.get_race(duel) as i64,
            i64::from(rc.get_attribute(duel)) << 32 | i64::from(rc.get_rank(duel)),
        ],
    );
    duel.confirm_cards(1 - tp, rc)?;
    duel.shuffle_extra(tp)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let &[race, attr_rank] = e.get_labels(duel) else { return Ok(()) };
    let (attr, rank) = (attr_rank >> 32, attr_rank & 0xffff_ffff);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        |c, duel| thfilter(c, duel, race as u64, attr as u32, rank as u32),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first()
        && duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_HAND)
    {
        duel.confirm_cards(1 - tp, sc)?;
        duel.shuffle_hand(tp)?;
        duel.shuffle_deck(tp)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
        let g = duel.select_matching_card(tp, CardId::is_able_to_deck, tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
        }
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA) && !c.is_type(duel, TYPE_XYZ));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_XYZ));
    Ok(())
}
