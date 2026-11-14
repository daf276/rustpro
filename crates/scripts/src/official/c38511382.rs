// Sinful Spoils of Betrayal - Silvera
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 38511382;

pub const LISTED_SERIES: &[u16] = &[SET_DIABELLSTAR, SET_SINFUL_SPOILS];
// Set, in the hand and in the Graveyard, with Diabellstar the Black Witch in the hand and on the field to send, and Sinful Spoils
// Spells and the opponent's Ash Blossom to chain to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (ID, GRAVE),
    (72270339, HAND),
    (72270339, MZONE),
    (80845034, HAND),
    (3496543, HAND),
    (14558127, HAND | OPPONENT),
    (14558127, HAND | OPPONENT),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, discost);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, gydiscon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, gydistg);
    e2.set_operation(duel, |duel, a| {
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel, cc: CardId) -> bool {
    let excluded = [c, cc].into_iter().collect::<CardSet>().union(c.get_equip_group(duel));
    c.is_monster(duel)
        && c.is_set_card(duel, &[SET_DIABELLSTAR])
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_target(CardId::is_negatable, 0, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(excluded))
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|tc, duel| tgfilter(tc, duel, c), tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |tc, duel| tgfilter(tc, duel, c),
        tp,
        LOCATION_HAND | LOCATION_MZONE,
        0,
        1,
        1,
        None,
    )?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_negatable(duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(
        tp,
        CardId::is_negatable,
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        1,
        Some(c.into()),
    )?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_negatable(duel)
        && tc.is_relate_to_effect(duel, a.e)
    {
        tc.negate_effects(duel, a.e.get_handler(duel), 0, true);
    }
    Ok(())
}

fn gydiscon(duel: &Duel, a: &Args) -> bool {
    let (tp, ev) = (a.tp, a.ev);
    let ch = ev.saturating_sub(1);
    if ch == 0 || !(a.ep == 1 - tp && duel.is_chain_disablable(ev as u8)) || a.re.is_some_and(|re| re.get_handler(duel).is_disabled(duel)) {
        return false;
    }
    let ch_player: u8 = duel.get_chain_info(ch as u8, CHAININFO_TRIGGERING_PLAYER);
    let Some(ch_eff) = duel.get_chain_info::<Option<EffectId>>(ch as u8, CHAININFO_TRIGGERING_EFFECT) else {
        return false;
    };
    let ch_c = ch_eff.get_handler(duel);
    ch_player == tp
        && ((ch_c.is_set_card(duel, &[SET_DIABELLSTAR]) && ch_eff.is_monster_effect(duel))
            || (ch_c.is_set_card(duel, &[SET_SINFUL_SPOILS]) && ch_eff.is_spell_trap_effect(duel)))
}

fn gydistg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    Ok(false)
}
