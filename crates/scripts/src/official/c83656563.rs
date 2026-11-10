// Fossil Machine Skull Wagon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 83656563;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, _, _, tp| {
            c.is_race(duel, RACE_ROCK) && c.is_location(duel, LOCATION_GRAVE) && c.is_controler(duel, tp)
        }),
        FusionMaterial::Filter(|c, duel, _, _, _| matfilter(c, duel)),
    ];
}

pub const LISTED_NAMES: &[u32] = &[CARD_FOSSIL_FUSION];

// In the Extra Deck, Fusion Summoned by Fossil Fusion from two Stone Ogre Grotto in the Graveyard, with copies in the Graveyard to
// banish, and the opponent's Goblin Calligraphers to destroy by battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (CARD_FOSSIL_FUSION, HAND),
    (CARD_FOSSIL_FUSION, HAND),
    (15023985, GRAVE),
    (15023985, GRAVE),
    (ID, MZONE),
    (ID, MZONE),
    (12057781, MZONE | OPPONENT),
    (12057781, MZONE | OPPONENT),
    (12057781, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.enable_revive_limit(duel);
    aux::add_lizard_check(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SINGLE_RANGE);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_value(duel, Value::SpSummonCondition(aux::fossil_limit));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_condition(duel, aux::bdocon);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn matfilter(c: CardId, duel: &Duel) -> bool {
    c.has_level(duel) && c.is_level(duel, &[5, 6])
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_target_player(1 - tp);
    duel.set_target_param(800);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 800);
    Ok(false)
}

fn damop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.damage(p, d, REASON_EFFECT)?;
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_spell_trap(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
