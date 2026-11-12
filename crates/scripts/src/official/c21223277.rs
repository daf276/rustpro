// Artorigus, King of the Noble Knights
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 21223277;

// In the Extra Deck, Xyz Summoned with two Level 4 Noble Knights (Noble Knight Artorigus, Shield-Bearer), with Noble Arms of Destiny
// in the Graveyard to equip, and Sangan on the opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (92125819, MZONE),
    (92125819, MZONE),
    (34242278, MZONE),
    (34242278, MZONE),
    (7452945, GRAVE),
    (7452945, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> =
        Some(|c, duel, scard, sumtype, tp| c.is_set_card_full(duel, &[SET_NOBLE_KNIGHT], Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

pub fn condition(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_xyz_summoned(duel)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ec: CardId) -> bool {
    c.is_set_card(duel, &[SET_NOBLE_ARMS])
        && c.is_can_be_effect_target(duel, e)
        && c.check_unique_on_field(duel, tp)
        && c.check_equip_target(duel, ec)
}

pub fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(|tc, duel| filter(tc, duel, e, tp, c), tp, LOCATION_GRAVE, 0, None);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel, e, tp, c));
    }
    let ft = duel.get_location_count(tp, LOCATION_SZONE).min(3);
    if chk == 0 {
        return Ok(ft > 0 && duel.is_existing_matching_card(|tc, duel| filter(tc, duel, e, tp, c), tp, LOCATION_GRAVE, 0, 1, None));
    }
    let eg = aux::select_unselect_group(duel, g, 1, ft as usize, &aux::dncheck, tp, HINTMSG_EQUIP)?;
    duel.set_target_card(eg);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(eg), eg.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_SZONE);
    let g = duel.get_target_cards(e).unwrap_or_default();
    if g.is_empty() || ft < g.len() as i32 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    for tc in g.iter() {
        duel.equip_full(tp, tc, c, true, true)?;
    }
    duel.equip_complete()
}

fn nbarmsfilters(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_equip_spell(duel) && c.is_set_card(duel, &[SET_NOBLE_ARMS])
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(nbarmsfilters, tp, LOCATION_SZONE, 0, 1, None)
            && duel.is_existing_matching_card(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(nbarmsfilters, tp, LOCATION_SZONE, 0, None);
    if ct == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, ct as u8, None)?;
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
