// Goblin Biker Big Gabonga
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 34001672;

pub const LISTED_SERIES: &[u16] = &[SET_GOBLIN];
// On the field with Samuel, which detaches a material that Xyz Import took from the opponent's Sangan, and a copy in the Extra
// Deck with two Sangan for the Xyz Summon; Goblin Zombie and Goblin Attack Force in the Deck to search and attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (76213610, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE),
    (26202165, MZONE),
    (63665875, DECK),
    (63665875, DECK),
    (78658564, DECK),
    (78658564, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    duel.enable_global_flag(GLOBALFLAG_DETACH_EVENT);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_DETACH_MATERIAL);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.r & REASON_LOST_TARGET == 0);
    e2.set_target(duel, fldattchtg);
    e2.set_operation(duel, fldattchop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_target(duel, dattchtg);
    e3.set_operation(duel, dattchop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GOBLIN]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn fldattchfilter(c: CardId, duel: &Duel, xyzc: CardId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_can_be_xyz_material(duel, Some(xyzc), tp, REASON_EFFECT)
        && (c.is_controler(duel, tp) || c.is_able_to_change_controler(duel))
}

fn fldattchtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let filter = |tc, duel: &Duel| fldattchfilter(tc, duel, c, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(c.is_type(duel, TYPE_XYZ) && duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, Some(c.into()))?;
    Ok(false)
}

fn fldattchop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && !tc.is_immune_to_effect(duel, a.e)
    {
        duel.overlay_full(c, tc, true)?;
    }
    Ok(())
}

fn dattchfilter(c: CardId, duel: &Duel, xyzc: Option<CardId>, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GOBLIN]) && c.is_monster(duel) && c.is_can_be_xyz_material(duel, xyzc, tp, REASON_EFFECT)
}

fn dattchtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let (tp, c) = (a.tp, a.e.get_handler(duel));
        return Ok(c.is_type(duel, TYPE_XYZ)
            && duel.is_existing_matching_card(|tc, duel| dattchfilter(tc, duel, Some(c), tp), tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn dattchop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
    let g = duel.select_matching_card(
        tp,
        |tc, duel| dattchfilter(tc, duel, None, duel.get_reason_player()),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.overlay(c, g)?;
    }
    Ok(())
}
