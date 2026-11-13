// Tindangle Acute Cerberus
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 75119040;
const BASE_GARDNA: u32 = 94365540;
const TOKEN: u32 = ID + 1;

pub const LISTED_NAMES: &[u32] = &[BASE_GARDNA, TOKEN];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;
    const MAX: Option<usize> = Some(3);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_TINDANGLE], Some(lc), sumtype, tp)
    }
}

// Tindangle Base Gardna, Angel and Protector to Link Summon with and in the Graveyard for the ATK gain.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (BASE_GARDNA, MZONE),
    (68860936, MZONE),
    (67744384, MZONE),
    (BASE_GARDNA, GRAVE),
    (68860936, GRAVE),
    (67744384, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(TOKEN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, atkcon);
    e1.set_value(duel, 3000);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_attack_announced_count(duel) > 0);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TINDANGLE]) && c.is_monster(duel)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_matching_group(cfilter, a.e.get_handler(duel).get_controler(duel), LOCATION_GRAVE, 0, None);
    g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 3
        && g.is_exists(duel, |c, duel| c.is_code(duel, BASE_GARDNA), 1, None)
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_TINDANGLE])
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    c.get_linked_group(duel).filter_count(duel, atkfilter, None) as i32 * 500
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    duel.is_player_can_special_summon_monster(
        tp,
        TOKEN,
        MonsterStats::new(SET_TINDANGLE, TYPES_TOKEN, 0, 0, 1, RACE_FIEND, ATTRIBUTE_DARK),
    )
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(can_summon_token(duel, tp) && duel.get_location_count(tp, LOCATION_MZONE) > 0);
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_MZONE) > 0
        && can_summon_token(duel, tp)
        && let Some(token) = duel.create_token(tp, TOKEN)
    {
        duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
