// Dragon Master Magia
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 12381100;

pub const LISTED_SERIES: &[u16] = &[SET_BLUE_EYES, SET_CHAOS];

const BLUE_EYES_ULTIMATE_DRAGON: u32 = 23995346;
// s.type_list[tp], the card types already negated this turn, at this key plus tp.
const TYPE_LIST: u64 = (ID as u64) << 16;

pub const LISTED_NAMES: &[u32] = &[BLUE_EYES_ULTIMATE_DRAGON];

const FFILTER: FusionMaterial = FusionMaterial::Filter(ffilter);
const BLUE_EYES: FusionMaterial =
    FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_BLUE_EYES], Some(fc), sumtype, tp));

struct UltimateMaterials;

impl FusionMix for UltimateMaterials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(BLUE_EYES_ULTIMATE_DRAGON), FFILTER];
}

struct BlueEyesMaterials;

impl FusionMix for BlueEyesMaterials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[BLUE_EYES, BLUE_EYES, BLUE_EYES, FFILTER];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Blue-Eyes Ultimate Dragon and Magician of Black Chaos (a Chaos Ritual Monster),
// destroyed by the opponent's Raigeki to Special Summon from the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (24094653, HAND),
    (BLUE_EYES_ULTIMATE_DRAGON, MZONE),
    (30208479, MZONE),
    (BLUE_EYES_ULTIMATE_DRAGON, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    if let Some(f0) = Fusion::add_proc_mix::<UltimateMaterials>(duel, c) {
        f0.set_description(duel, aux::stringid(ID, 0));
    }
    if let Some(f1) = Fusion::add_proc_mix_n::<BlueEyesMaterials>(duel, c) {
        f1.set_description(duel, aux::stringid(ID, 1));
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(aux::fuslimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, negcon);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 3));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    aux::global_check(duel, ID, |duel| aux::add_values_reset(duel, reset));
}

fn reset(duel: &mut Duel) {
    duel.set_script_state(TYPE_LIST, 0);
    duel.set_script_state(TYPE_LIST | 1, 0);
}

fn ffilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_CHAOS], Some(fc), sumtype, tp) && c.is_type_full(duel, TYPE_RITUAL, Some(fc), sumtype, tp)
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    let negated_types = duel.get_script_state(TYPE_LIST | u64::from(a.tp));
    a.rp == 1 - a.tp
        && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && duel.is_chain_negatable(a.ev as u8)
        && negated_types & i64::from(re.get_active_type(duel)) == 0
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let Some(re) = a.re else { return Ok(false) };
    let key = TYPE_LIST | u64::from(a.tp);
    let negated_types = duel.get_script_state(key) | i64::from(re.get_active_type(duel) & (TYPE_MONSTER | TYPE_SPELL | TYPE_TRAP));
    duel.set_script_state(key, negated_types);
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    let rc = re.get_handler(duel);
    if rc.is_relate_to_effect(duel, re) && rc.is_destructable(duel) {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_position(duel, POS_FACEUP) && c.is_previous_controler(duel, a.tp) && a.rp == 1 - a.tp
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    if !((c.is_set_card(duel, &[SET_BLUE_EYES]) || (c.is_set_card(duel, &[SET_CHAOS]) && c.is_ritual_monster(duel)))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false))
    {
        return false;
    }
    if c.is_location(duel, LOCATION_GRAVE) {
        mmz_chk
    } else {
        duel.get_location_count_from_ex(tp, tp, None, c) > 0
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let location = LOCATION_EXTRA | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, mmz_chk), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, mmz_chk));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
