// Diplexer Chimera
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 78084378;
const CYNET_FUSION: u32 = 65801012;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_CYBERSE, Some(fc), sumtype, tp)); 2];
}

pub const LISTED_NAMES: &[u32] = &[CYNET_FUSION];

// In the Extra Deck, Fusion Summoned by Polymerization with Cyberse Normal Monsters, with Cynet Fusion and a Cyberse monster in the
// Graveyard to add and Raigeki of the opponent to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (24154052, HAND),
    (24154052, MZONE),
    (32295838, HAND),
    (CYNET_FUSION, GRAVE),
    (36211150, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, cost);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |c, duel| c.is_race(duel, RACE_CYBERSE), (1, 1), false, None, None));
    }
    let sg = duel.select_release_group_cost(tp, |c, duel| c.is_race(duel, RACE_CYBERSE), (1, 1), false, None, None)?;
    duel.release(sg, REASON_COST)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 1);
    e1.set_condition(duel, |duel, _| duel.is_battle_phase());
    e1.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.is_spell_trap_effect(duel)));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_fusion_summoned(duel)
}

fn filter1(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_CYBERSE) && c.is_able_to_hand(duel)
}

fn filter2(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CYNET_FUSION) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter1, tp, LOCATION_GRAVE, 0, 1, Some(c.into()))
            && duel.is_existing_target(filter2, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g1 = duel.select_target(tp, filter1, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g2 = duel.select_target(tp, filter2, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    let g = g1.unwrap_or_default().union(g2.unwrap_or_default());
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(g), 2, 0, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(g), 2, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let sg = g.filter(duel, |c, duel| c.is_relate_to_effect(duel, e), None);
    if !sg.is_empty() {
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}
