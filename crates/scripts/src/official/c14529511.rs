// Mementomictlan Tecuhtlica - Creation King
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 14529511;

pub const LISTED_SERIES: &[u16] = &[SET_MEMENTO];

const MEMENTOMICTLAN: u32 = 43338320;
const MEMENTOTLAN_ANGWITCH: u32 = 54550967;
const MEMENTOTLAN_TATSUNOOTOSHIGO: u32 = 81677154;
const MEMENTOTLAN_GOBLIN: u32 = 17943271;
const MEMENTOTLAN_BONE_PARTY: u32 = 80722024;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_MEMENTO], Some(fc), sumtype, tp)); 3];
}

pub const LISTED_NAMES: &[u32] = &[MEMENTOMICTLAN];

// In the Extra Deck, Fusion Summoned by Polymerization with three Memento monsters to send Memento cards from the Deck and
// Extra Deck, on the field to destroy the opponent's Sangan with a Memento monster, and in the Graveyard to search Mementomictlan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, GRAVE),
    (CARD_POLYMERIZATION, HAND),
    (MEMENTOTLAN_ANGWITCH, HAND),
    (MEMENTOTLAN_TATSUNOOTOSHIGO, HAND),
    (MEMENTOTLAN_GOBLIN, HAND),
    (MEMENTOTLAN_BONE_PARTY, DECK),
    (MEMENTOMICTLAN, DECK),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_banish);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e2.set_cost(duel, opccost);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END);
    e3.set_cost(duel, opccost);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, MEMENTOMICTLAN) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn opccost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(!duel.has_flag_effect(tp, ID));
    }
    duel.register_flag_effect(tp, ID, RESET_CHAIN, 0, 1);
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MEMENTO]) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let location = LOCATION_DECK | LOCATION_EXTRA;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, location, 0, 3, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 3, tp, location as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, 3, 3, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_controler(duel, 1 - tp) || (c.is_faceup(duel) && c.is_set_card(duel, &[SET_MEMENTO]))) && c.is_can_be_effect_target(duel, e)
}

fn desrescon(maxc: usize, tp: u8) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, _, _| {
        let ct1 = sg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None);
        let ct2 = sg.len() - ct1;
        (ct1 == ct2, ct1 > maxc || ct2 > maxc)
    }
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_MEMENTO])),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        ) && duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(|c, duel| desfilter(c, duel, e, tp), tp, LOCATION_MZONE, LOCATION_ONFIELD, None);
    let ct = g.filter_count(duel, |c, duel| c.is_controler(duel, tp), None);
    let rescon = desrescon(ct.min(g.len() - ct), tp);
    let tg = aux::select_unselect_group_full(duel, g, 2, g.len(), &rescon, tp, HINTMSG_DESTROY, Some(&rescon), false)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(tg), tg.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
