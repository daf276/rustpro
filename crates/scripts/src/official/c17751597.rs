// Branded Retribution
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 17751597;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ, ID];

// Albion the Branded Dragon on the field and in the Graveyard (Fusion Monsters listing Albaz), Branded Fusion in the Graveyard to
// return, and the opponent's Monster Reborn with a monster in their Graveyard to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (87746184, MZONE),
    (87746184, GRAVE),
    (44362883, GRAVE),
    (83764718, HAND | OPPONENT),
    (73481154, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
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

fn condition(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    duel.is_chain_negatable(a.ev as u8)
        && (re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && re.is_has_category(duel, CATEGORY_SPECIAL_SUMMON)
}

fn texfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel)
        && c.is_type(duel, TYPE_FUSION)
        && c.is_monster(duel)
        && c.is_able_to_extra(duel)
        && c.lists_code_as_material(duel, &[CARD_ALBAZ])
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let in_location = |location| sg.filter_count(duel, |c, duel| c.is_location(duel, location), None);
    (
        (sg.len() == 1 && in_location(LOCATION_MZONE) >= 1) || (sg.len() == 2 && in_location(LOCATION_GRAVE) == 2),
        false,
    )
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let rg = duel.get_matching_group(texfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(!rg.is_empty() && aux::select_unselect_group_check(duel, rg, 1, 2, &rescon));
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
        }
    }
    duel.set_operation_info(0, CATEGORY_TOEXTRA, Some(rg), 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let rg = duel.get_matching_group(texfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    if rg.is_empty() {
        return Ok(());
    }
    let sg = aux::select_unselect_group_full(duel, rg, 1, 2, &rescon, tp, HINTMSG_TODECK, Some(&rescon), false)?;
    if sg.is_empty() {
        return Ok(());
    }
    duel.hint_selection(sg);
    if duel.send_to_deck(sg, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)? > 0
        && sg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_EXTRA), 1, None)
        && duel.negate_activation(a.ev as u8)
        && a.re.is_some_and(|re| re.get_handler(duel).is_relate_to_effect(duel, re))
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BRANDED]) && c.is_spell_trap(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        let handler = a.e.get_handler(duel);
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, Some(handler.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0
    {
        duel.confirm_cards(1 - a.tp, tc)?;
    }
    Ok(())
}
