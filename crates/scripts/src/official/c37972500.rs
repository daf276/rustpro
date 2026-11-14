// GranSolfachord Musecia
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, PZONE};

const ID: u32 = 37972500;

pub const LISTED_SERIES: &[u16] = &[SET_SOLFACHORD];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_PENDULUM, Some(lc), sumtype, tp)
    }
}

// Copies in the Extra Deck, Link Summoned from Pendulum Monsters (Flash Knight), with Pendulum Monsters in the hand and FaSolfachord Fancia on the field
// to send DoSolfachord Cutia from the Deck face-up to the Extra Deck, and in the Pendulum Zone to Pendulum Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (17390179, MZONE),
    (17390179, MZONE),
    (17390179, HAND),
    (54100561, MZONE),
    (54100561, HAND),
    (55226153, DECK),
    (55226153, DECK),
    (55226153, PZONE),
];
// The second effect needs a Pendulum Summoned Solfachord whose Scale is the Level of another Solfachord Pendulum Monster in the Deck: the ported ones (Scales 5 and 8, Levels 1 and 4) have no such pair.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, xthtg);
    e1.set_operation(duel, xthop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn xthfilter1(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_type(duel, TYPE_PENDULUM)
        && !c.is_forbidden(duel)
        && duel.is_existing_matching_card(|ec, duel| xthfilter2(ec, duel, c), tp, LOCATION_EXTRA, 0, 1, Some(c.into()))
}

fn xthfilter2(c: CardId, duel: &Duel, ac: CardId) -> bool {
    c.is_faceup(duel)
        && c.is_type(duel, TYPE_PENDULUM)
        && c.is_able_to_hand(duel)
        && !(c.get_scale(duel) + ac.get_scale(duel)).is_multiple_of(2)
}

fn xthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| xthfilter1(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn xthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_matching_card(tp, |c, duel| xthfilter1(c, duel, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(xtc) = g.first()
        && duel.send_to_extra_p(xtc, PLAYER_NONE, REASON_EFFECT)? != 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(
            tp,
            |c, duel| xthfilter2(c, duel, xtc),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            1,
            Some(xtc.into()),
        )?;
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn thfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_SOLFACHORD])
        && c.is_summon_player(duel, tp)
        && c.is_pendulum_summoned(duel)
        && c.is_can_be_effect_target(duel, e)
        && duel.is_existing_matching_card(|dc, duel| thfilter2(dc, duel, c.get_scale(duel)), tp, LOCATION_DECK, 0, 1, None)
}

fn thfilter2(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_type(duel, TYPE_PENDULUM) && c.is_set_card(duel, &[SET_SOLFACHORD]) && c.is_level(duel, &[lv]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let eg = a.eg.unwrap_or_default();
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| thfilter1(c, duel, e, tp), 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SELECT);
    let g = eg.filter_select(duel, tp, |c, duel| thfilter1(c, duel, e, tp), 1, 1, None)?;
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let scale = tc.get_scale(duel);
    let g = duel.select_matching_card(tp, |c, duel| thfilter2(c, duel, scale), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
