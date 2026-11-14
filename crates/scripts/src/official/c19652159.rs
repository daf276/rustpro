// Light and Darkness Dragonlord
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 19652159;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(matfilter::<ATTRIBUTE_LIGHT>),
        FusionMaterial::Filter(matfilter::<ATTRIBUTE_DARK>),
    ];
}

// On the field to negate, Blue-Eyes White Dragon and Tri-Horned Dragon in the GY to revive after the opponent's Raigeki,
// and in the Extra Deck with Polymerization and both Dragons in hand for its Fusion Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (24094653, HAND),
    (89631139, HAND),
    (39111158, HAND),
    (89631139, GRAVE),
    (39111158, GRAVE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_ADD_ATTRIBUTE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, ATTRIBUTE_DARK as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE + CATEGORY_NEGATE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_F);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) || re.is_monster_effect(duel))
    });
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp && a.e.get_handler(duel).is_previous_controler(duel, a.tp)
    });
    e3.set_target(duel, sptg);
    e3.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e3);
}

fn matfilter<const ATTRIBUTE: u32>(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_level(duel, &[8])
        && c.is_attribute_full(duel, ATTRIBUTE, Some(fc), sumtype, tp)
        && c.is_race_full(duel, RACE_DRAGON, Some(fc), sumtype, tp)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.has_flag_effect(duel, ID));
    }
    if c.is_has_effect(duel, EFFECT_REVERSE_UPDATE).is_some() {
        c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, a.e)
        && c.is_faceup(duel)
        && c.is_attack_above(duel, 1000)
        && c.is_defense_above(duel, 1000)
        && !c.is_status(duel, STATUS_BATTLE_DESTROYED))
    {
        return Ok(());
    }
    let (prev_atk, prev_def) = (c.get_attack(duel), c.get_defense(duel));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, -1000);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    if c.is_attack(duel, &[prev_atk - 1000]) && c.is_defense(duel, &[prev_def - 1000]) && u32::from(duel.get_current_chain()) == a.ev + 1 {
        duel.negate_activation(a.ev as u8);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}
