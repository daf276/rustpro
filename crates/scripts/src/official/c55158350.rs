// Fusion Draft
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 55158350;

// Gem-Knight Ruby lists Gem-Knight Garnet, a Normal Monster, so a second copy can be added.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (76614340, EXTRA), (91731841, DECK), (91731841, DECK), (91731841, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn materials(c: CardId, duel: &Duel) -> Vec<u32> {
    Fusion::material(duel, c.get_original_code(duel))
}

fn revealfilter(c: CardId, duel: &Duel, tp: u8, cost_card: Option<CardId>) -> bool {
    let codes = materials(c, duel);
    c.is_fusion_monster(duel)
        && !c.is_public(duel)
        && !codes.is_empty()
        && duel.is_existing_matching_card(
            |c, duel| thfilter(c, duel, &codes),
            tp,
            LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            cost_card.map(CardSet::from),
        )
}

fn thfilter(c: CardId, duel: &Duel, codes: &[u32]) -> bool {
    codes.iter().any(|&code| c.is_code(duel, code)) && c.is_able_to_hand(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, -100);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(
            duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, Some(c.into()))
                && duel.is_existing_matching_card(|c, duel| revealfilter(c, duel, tp, None), tp, LOCATION_EXTRA, 0, 1, None),
        );
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    let cost_card = duel.get_operated_group().get_first();
    e.set_label_object_group(duel, cost_card.map(CardSet::from));
    if let Some(cost_card) = cost_card {
        cost_card.create_effect_relation(duel, e);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let filter = |c, duel: &Duel| revealfilter(c, duel, tp, cost_card);
    let Some(reveal_card) = duel
        .select_matching_card(tp, filter, tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(false);
    };
    duel.confirm_cards(1 - tp, reveal_card)?;
    duel.shuffle_extra(tp)?;
    let labels: Vec<i64> = materials(reveal_card, duel).into_iter().map(i64::from).collect();
    e.set_labels(duel, &labels);
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let cost_chk = a.e.get_label(duel) == -100;
        a.e.set_label(duel, 0);
        return Ok(cost_chk);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let codes: Vec<u32> = e.get_labels(duel).iter().map(|&code| code as u32).collect();
    let exc = e
        .get_label_object_group(duel)
        .and_then(|g| g.get_first())
        .filter(|cost_card| cost_card.is_relate_to_effect(duel, e))
        .map(CardSet::from);
    let filter = aux::necro_valley_filter(|c, duel| thfilter(c, duel, &codes));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let sc = duel
        .select_matching_card(tp, &filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, exc)?
        .get_first();
    if let Some(sc) = sc
        && duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_HAND)
    {
        duel.confirm_cards(1 - tp, sc)?;
        duel.shuffle_hand(tp)?;
        if sc.is_type(duel, TYPE_NORMAL)
            && duel.is_existing_matching_card(&filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, exc)
            && duel.select_yes_no(tp, aux::stringid(ID, 1))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let g = duel.select_matching_card(tp, &filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, exc)?;
            if !g.is_empty() {
                duel.break_effect()?;
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA) && !c.is_fusion_monster(duel));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
