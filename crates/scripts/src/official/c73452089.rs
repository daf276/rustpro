// Dark Cavalry
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 73452089;
const GAIA: u32 = 6368038;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN];
pub const MATERIAL_SETCODE: &[u16] = &[SET_DARK_MAGICIAN];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(CARD_DARK_MAGICIAN),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_WARRIOR, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck and on the field, Fusion Summoned by Polymerization with Dark Magician and Gaia The Fierce Knight, to
// negate the opponent's Mystical Space Typhoon with a card to discard and Spells on the field to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (CARD_DARK_MAGICIAN, HAND),
    (GAIA, HAND),
    (5318639, HAND | OPPONENT),
    (5318639, SZONE),
    (5318639, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(val));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, discon);
    e3.set_cost(duel, discost);
    e3.set_target(duel, stock::negate_destroy_target);
    e3.set_operation(duel, disop);
    c.register_effect(duel, e3);
}

fn val(duel: &Duel, _: EffectId, _: CardId) -> i32 {
    duel.get_matching_group_count(
        CardId::is_spell_trap,
        0,
        LOCATION_ONFIELD | LOCATION_GRAVE,
        LOCATION_ONFIELD | LOCATION_GRAVE,
        None,
    ) as i32
        * 100
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand_full(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD, None)?;
    Ok(false)
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) || !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let tg = duel.get_chain_info::<Option<CardSet>>(a.ev as u8, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.is_exists(duel, CardId::is_on_field, 1, None)) && duel.is_chain_negatable(a.ev as u8)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
