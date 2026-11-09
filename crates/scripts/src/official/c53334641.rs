// Ghostrick Angel of Mischief
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 53334641;

pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

// On the field and in the Extra Deck, with Xyz Import to attach the opponent's monsters; Ghostrick Shot in the hand to attach and
// in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (16195942, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (69809989, HAND),
    (69809989, DECK),
    (69809989, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EVENT_ADJUST);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_operation(duel, winop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, mttg);
    e3.set_operation(duel, mtop);
    c.register_effect(duel, e3);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    let sumtype = u64::from(SUMMON_TYPE_XYZ);
    c.is_faceup(duel)
        && c.is_set_card_full(duel, &[SET_GHOSTRICK], Some(xyzc), sumtype, tp)
        && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), sumtype, tp)
        && !c.is_summon_code(duel, Some(xyzc), sumtype, tp, &[ID])
}

fn winop(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.e.get_handler(duel).get_overlay_count(duel) == 10 {
        duel.win(a.tp, WIN_REASON_GHOSTRICK_MISCHIEF);
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GHOSTRICK]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn mtfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GHOSTRICK])
}

fn mttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(
            a.e.get_handler(duel).is_type(duel, TYPE_XYZ) && duel.is_existing_matching_card(mtfilter, a.tp, LOCATION_HAND, 0, 1, None)
        );
    }
    Ok(false)
}

fn mtop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) || c.is_facedown(duel) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let g = duel.select_matching_card(tp, mtfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.overlay(c, g)?;
    }
    Ok(())
}
