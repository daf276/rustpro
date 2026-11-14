// Neo Blue-Eyes Ultimate Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_CHAIN_NEGATED;

const ID: u32 = 56532353;

pub const LISTED_SERIES: &[u16] = &[SET_BLUE_EYES];
pub const MATERIAL_SETCODE: &[u16] = &[SET_BLUE_EYES];

pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_BLUEEYES_W_DRAGON); 3];
}

// In the Extra Deck, Fusion Summoned by Polymerization with three Blue-Eyes White Dragons, with a Blue-Eyes Fusion Monster in the
// Extra Deck for the cost, a copy in the Graveyard to negate an opponent's Mage Power targeting a Blue-Eyes White Dragon, and
// the opponent's monster to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (CARD_POLYMERIZATION, HAND),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (83746708, HAND | OPPONENT),
    (83746708, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
// Neither effect has a description, so the negation is told from the chain attack by its message.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0), Check::Message(MSG_CHAIN_NEGATED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_DAMAGE_STEP_END);
    e1.set_count_limit(duel, 2, ID);
    e1.set_condition(duel, atcon);
    e1.set_cost(duel, atcost);
    e1.set_operation(duel, atop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_condition(duel, condition);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BLUE_EYES]) && c.is_type(duel, TYPE_FUSION) && c.is_able_to_grave_as_cost(duel)
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    c.is_fusion_summoned(duel)
        && duel.get_attacker() == Some(c)
        && duel.check_mut(|duel| c.can_chain_attack_full(duel, 0, false))
        && !duel.is_existing_matching_card(CardId::is_faceup, tp, LOCATION_ONFIELD, 0, 1, Some(c.into()))
}

fn atcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, costfilter, LOCATION_EXTRA)
}

fn atop(duel: &mut Duel, _: &Args) -> Script<()> {
    duel.chain_attack();
    Ok(())
}

fn filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.is_location(duel, LOCATION_MZONE) && c.is_faceup(duel) && c.is_set_card(duel, &[SET_BLUE_EYES])
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let g: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    g.is_some_and(|g| g.is_exists(duel, |c, duel| filter(c, duel, a.tp), 1, None)) && duel.is_chain_negatable(a.ev as u8)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if let Some(re) = a.re
        && re.get_handler(duel).is_destructable(duel)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
        && let Some(eg) = a.eg
    {
        duel.destroy(eg, REASON_EFFECT)?;
    }
    Ok(())
}
