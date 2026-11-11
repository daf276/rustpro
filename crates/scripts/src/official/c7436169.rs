// Trivikarma
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 7436169;

pub const LISTED_NAMES: &[u32] = &[CARD_VISAS_STARFROST, ID];

// In the hand and the GY, with Visas Starfrost and an opponent's Effect Monster (Effect Veiler) to target, and Mannadium Reframing
// and Tearlaments Sulliek (which list Visas Starfrost) in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (CARD_VISAS_STARFROST, MZONE),
    (CARD_VISAS_STARFROST, MZONE),
    (97268402, MZONE | OPPONENT),
    (97268402, MZONE | OPPONENT),
    (18158393, DECK),
    (18158393, DECK),
    (74920585, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(
        duel,
        TIMING_DAMAGE_STEP,
        TIMING_DAMAGE_STEP | TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END,
    );
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_can_be_effect_target(duel, e)
        && ((c.is_code(duel, CARD_VISAS_STARFROST) && c.is_controler(duel, tp))
            || (c.is_controler(duel, 1 - tp) && c.is_type(duel, TYPE_EFFECT) && c.is_negatable_monster(duel)))
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    (sg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None) == 1, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let rg = duel.get_matching_group(|c, duel| tgfilter(c, duel, e, tp), tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, rg, 2, 2, &|duel, sg, _, _| {
            rescon(duel, sg, tp)
        }));
    }
    let tg = aux::select_unselect_group(duel, rg, 2, 2, &|duel, sg, _, _| rescon(duel, sg, tp), tp, HINTMSG_TARGET)?;
    duel.set_target_card(tg);
    let hg = tg.filter(duel, |c, duel| c.is_controler(duel, tp), None);
    e.set_label_object_group(duel, hg.get_first().map(Into::into));
    duel.set_operation_info(
        0,
        CATEGORY_DISABLE,
        Some(tg.filter(duel, |c, duel| c.is_controler(duel, 1 - tp), None)),
        1,
        0,
        0,
    );
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(hg), 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_target_cards(e).unwrap_or_default();
    if g.is_empty() {
        return Ok(());
    }
    let mut cards = g.iter();
    let (mut tc1, mut tc2) = (cards.next(), cards.next());
    if tc2.is_some() && tc2 == e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        (tc1, tc2) = (tc2, tc1);
    }
    let Some(tc2) = tc2 else { return Ok(()) };
    if tc2.is_controler(duel, 1 - tp) && tc2.is_faceup(duel) && tc2.is_can_be_disabled_by_effect(duel, e, true) {
        let c = e.get_handler(duel);
        duel.negate_related_chain(tc2, RESET_TURN_SET);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc2.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_value(duel, RESET_TURN_SET as i32);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc2.register_effect(duel, e2);
        let val = tc2.get_base_attack(duel).max(tc2.get_base_defense(duel)) / 2;
        if val == 0 {
            return Ok(());
        }
        if let Some(tc1) = tc1
            && tc1.is_controler(duel, tp)
            && tc1.is_relate_to_effect(duel, e)
            && tc1.is_faceup(duel)
        {
            tc1.update_attack(duel, val, Some(RESET_EVENT | RESETS_STANDARD), Some(c), None);
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, CARD_VISAS_STARFROST) && c.is_spell_trap(duel) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
