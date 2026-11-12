// Inzektor Exa-Beetle
use super::c26211048::equipop;
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 44505297;

// In the Extra Deck, Xyz Summoned from Level 6 Normal Monsters (Summoned Skull, Swordstalker) with monsters in either Graveyard to equip,
// and Sangan on the opponent's side to send to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (70781052, MZONE),
    (70781052, MZONE),
    (50005633, MZONE),
    (50005633, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
    (50005633, GRAVE),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, None, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn eqfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && !c.is_forbidden(duel)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && eqfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(eqfilter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_target(tp, eqfilter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        equipop(duel, c, e, tp, tc)?;
    }
    Ok(())
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_ONFIELD, 0, 1, None)
            && duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let mut g1 = duel
        .select_target(tp, CardId::is_faceup, tp, LOCATION_ONFIELD, 0, 1, 1, None)?
        .unwrap_or_default();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g2 = duel
        .select_target(tp, CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, 1, None)?
        .unwrap_or_default();
    g1.merge(g2);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g1), 2, 0, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let tg = g.filter(duel, |c, duel| c.is_faceup(duel) && c.is_relate_to_effect(duel, e), None);
    if !tg.is_empty() {
        duel.send_to_grave(tg, REASON_EFFECT)?;
    }
    Ok(())
}
