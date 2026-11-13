// Daedalus Maelstrom
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 64233143;

pub const LISTED_NAMES: &[u32] = &[CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON, CARD_UMI];

// In the Graveyard with Levia-Dragon of Atlantis - Daedalus (named Umi on the field) and the opponent's Sangan. The Summon negation
// needs Atlantis, the Dragon City, an OCG-only Field Spell that is not ported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (64603351, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e1a.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1a.set_code(duel, EVENT_SUMMON);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_condition(duel, condition);
    e1a.set_target(duel, target);
    e1a.set_operation(duel, activate);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, |duel, a, chk| Cost::and(duel, a, chk, &[Cost::self_banish, tgcost]));
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_current_chain_full(true) == 0
        && a.eg
            .unwrap_or_default()
            .is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - tp), 1, None)
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON)),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let count = a.eg.map_or(0, |eg| eg.len()) as u8;
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, a.eg, count, tp, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, count, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_ALL, LOCATION_MZONE as i32);
    Ok(false)
}

fn desconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_link_monster(duel)
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute_except(duel, ATTRIBUTE_WATER) || c.is_facedown(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.negate_summon(a.eg.unwrap_or_default());
    if duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)? > 0
        && duel.is_existing_matching_card(desconfilter, tp, LOCATION_MZONE, 0, 1, None)
        && duel.is_existing_matching_card(desfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        let g = duel.get_matching_group(desfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
        if !g.is_empty() {
            duel.break_effect()?;
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn tgcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_UMI) && c.is_faceup(duel) && c.is_able_to_grave_as_cost(duel)
}

fn tgcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, tgcostfilter, LOCATION_ONFIELD)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_able_to_grave(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_grave, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, CardId::is_able_to_grave, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, 1, tp, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_grave(tc, REASON_EFFECT)?;
    }
    Ok(())
}
