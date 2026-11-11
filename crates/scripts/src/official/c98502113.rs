// Dark Paladin
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, SZONE};

const ID: u32 = 98502113;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_DARK_MAGICIAN), FusionMaterial::Code(CARD_BUSTER_BLADER)];
}

pub const MATERIAL_SETCODE: &[u16] = &[SET_DARK_MAGICIAN, SET_BUSTER_BLADER];
pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN, CARD_BUSTER_BLADER];

// On the field, to negate the opponent's set Mystical Space Typhoon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (5318639, SZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, discon);
    e1.set_cost(duel, discost);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(val));
    c.register_effect(duel, e2);
    c.add_must_be_fusion_summoned(duel);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && (c.is_location(duel, LOCATION_GRAVE) || c.is_faceup(duel))
}

fn val(duel: &Duel, _: EffectId, _: CardId) -> i32 {
    let locations = LOCATION_MZONE | LOCATION_GRAVE;
    duel.get_matching_group_count(filter, 0, locations, locations, None) as i32 * 500
}

fn discon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.re
            .is_some_and(|re| re.is_spell_effect(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && duel.is_chain_negatable(a.ev as u8)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(re) = a.re else { return Ok(()) };
    if !c.is_faceup(duel) || !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
