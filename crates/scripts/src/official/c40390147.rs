// Starliege Lord Galaxion
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 40390147;

pub const LISTED_NAMES: &[u32] = &[CARD_GALAXYEYES_P_DRAGON];

// In the Extra Deck, Xyz Summoned with Level 4 Photons (Photon Delta Wing), with Galaxy-Eyes Photon Dragon in the hand and the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (47051709, MZONE),
    (47051709, MZONE),
    (CARD_GALAXYEYES_P_DRAGON, HAND),
    (CARD_GALAXYEYES_P_DRAGON, DECK),
    (CARD_GALAXYEYES_P_DRAGON, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_set_card_full(duel, &[SET_PHOTON], Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_GALAXYEYES_P_DRAGON) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcheck<const LOC: u32>(duel: &Duel, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOC, 0, 1, None)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let choices: [CostChoice; 2] = [
        (Cost::detach_from_self::<1, 1>, aux::stringid(ID, 1), spcheck::<LOCATION_HAND>),
        (Cost::detach_from_self::<2, 2>, aux::stringid(ID, 2), spcheck::<LOCATION_DECK>),
    ];
    Cost::choice(duel, a, chk, &choices)
}

fn location(duel: &Duel, e: EffectId) -> u32 {
    if Cost::get_choice(duel, e) == Some(1) {
        LOCATION_HAND
    } else {
        LOCATION_DECK
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0);
    }
    let loc = location(duel, a.e);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, loc as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let loc = location(duel, e);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, loc, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
