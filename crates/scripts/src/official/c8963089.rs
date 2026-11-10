// Heosvarog the Mechanical Dawn
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 8963089;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(matfilter); 2];
}

// In the Extra Deck, Fusion Summoned by Polymerization with LIGHT Machines (Gradius, V-Tiger Jet), with a Polymerization in
// the Graveyard to add back and the opponent's Raigeki to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, GRAVE),
    (10992251, HAND),
    (10992251, MZONE),
    (51638941, HAND),
    (51638941, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e1.set_target(duel, regtg);
    e1.set_operation(duel, regop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.ep == 1 - a.tp
            && a.re.is_some_and(|re| re.is_spell_trap_effect(duel))
            && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
            && duel.is_chain_negatable(a.ev as u8)
    });
    e2.set_cost(duel, negcost);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, |duel, a| {
        duel.negate_activation(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e2);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(fc), sumtype, tp) && c.is_race_full(duel, RACE_MACHINE, Some(fc), sumtype, tp)
}

fn regtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let turn_ct = duel.get_turn_count();
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_count_limit(duel, 1, 0);
    e1.set_label(duel, i64::from(turn_ct));
    e1.set_condition(duel, |duel, a| i64::from(duel.get_turn_count()) != a.e.get_label(duel));
    e1.set_operation(duel, thop);
    let reset_count = if duel.get_current_phase() <= PHASE_STANDBY { 2 } else { 1 };
    e1.set_reset_full(duel, RESET_PHASE | PHASE_STANDBY, reset_count);
    duel.register_effect(e1, tp);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel)
        && c.get_own_effects(duel)
            .iter()
            .any(|eff| eff.is_has_category(duel, CATEGORY_FUSION_SUMMON))
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_CARD, 0, u64::from(ID));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, aux::necro_valley_filter(thfilter), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_race(duel, RACE_MACHINE) && c.is_able_to_remove_as_cost(duel) && c.is_faceup(duel)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let in_location = |location, count| sg.is_exists(duel, |c, duel| c.is_location(duel, location), count, None);
    (
        (sg.len() == 1 && in_location(LOCATION_MZONE, 1)) || in_location(LOCATION_GRAVE, 2),
        false,
    )
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(costfilter, tp, LOCATION_MZONE, 0, 1, None)
            || duel.is_existing_matching_card(costfilter, tp, LOCATION_GRAVE, 0, 2, None));
    }
    let g = duel.get_matching_group(costfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    let cg = aux::select_unselect_group_full(duel, g, 1, 2, &rescon, tp, HINTMSG_REMOVE, Some(&rescon), false)?;
    duel.remove(cg, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
    Ok(false)
}
