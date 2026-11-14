// Gem-Knight Lady Brilliant Diamond
use super::c3113836::{gem_knight, register_splimit};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 19355597;

pub const LISTED_SERIES: &[u16] = &[SET_GEM_KNIGHT];
pub const MATERIAL_SETCODE: &[u16] = &[SET_GEM, SET_GEM_KNIGHT];

const RUBY: u32 = 76614340;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(gem_knight); 3];
}

// In the Extra Deck, Fusion Summoned by Polymerization with three Gem-Knight Garnet; on the field with Gem-Knight Ruby in the Extra
// Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (91731841, HAND),
    (91731841, HAND),
    (91731841, HAND),
    (RUBY, EXTRA),
    (RUBY, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0), Check::SpecialSummonedCode(RUBY)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    register_splimit(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_GEM_KNIGHT])
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, Some(c)), tp, LOCATION_EXTRA, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: Option<CardId>) -> bool {
    c.is_set_card(duel, &[SET_GEM_KNIGHT])
        && c.is_type(duel, TYPE_FUSION)
        && duel.get_location_count_from_ex(tp, tp, mc.map(CardSet::from), c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| tgfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn tgfilter2(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_GEM_KNIGHT])
        && duel.is_existing_matching_card(|sc, duel| spfilter2(sc, duel, tp, c), tp, LOCATION_EXTRA, 0, 1, None)
}

fn spfilter2(c: CardId, duel: &Duel, tp: u8, mc: CardId) -> bool {
    c.is_set_card(duel, &[SET_GEM_KNIGHT])
        && c.is_type(duel, TYPE_FUSION)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let tg = duel.select_matching_card(tp, |c, duel| tgfilter2(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    if let Some(tc) = tg.get_first()
        && duel.send_to_grave(tc, REASON_EFFECT)? != 0
        && tc.is_location(duel, LOCATION_GRAVE)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, None), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        duel.special_summon(g, 0, tp, tp, true, false, POS_FACEUP)?;
    }
    Ok(())
}
