// Predaplant Triphyoverutum
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 79864860;

pub const COUNTER_LIST: &[u16] = &[COUNTER_PREDATOR];

// In the Graveyard, with Dragostapelia putting a Predator Counter on the opponent's Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (69946549, MZONE), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(ffilter); 3];
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_SPSUMMON);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, discon);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP_DEFENSE>);
    c.register_effect(duel, e3);
}

fn ffilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(fc), sumtype, tp) && c.is_on_field(duel)
}

fn vfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_counter(duel, COUNTER_PREDATOR) > 0
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let g = duel.get_matching_group(vfilter, c.get_controler(duel), LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    g.get_sum(duel, |c, duel| i64::from(c.get_base_attack(duel))) as i32
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.tp != a.ep
        && duel.get_current_chain() == 0
        && a.eg
            .is_some_and(|eg| eg.is_exists(duel, |c, duel| c.get_summon_location(duel) == LOCATION_EXTRA, 1, None))
        && a.e.get_handler(duel).is_fusion_summoned(duel)
}

fn extra_summoned(duel: &Duel, a: &Args) -> CardSet {
    a.eg.unwrap_or_default()
        .filter(duel, |c, duel| c.is_previous_location(duel, LOCATION_EXTRA), None)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = extra_summoned(duel, a);
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = extra_summoned(duel, a);
    duel.negate_summon(g);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(vfilter, a.tp, 0, LOCATION_MZONE, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}
