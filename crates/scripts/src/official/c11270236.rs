// Invoked Elysium
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 11270236;
pub const MATERIAL_SETCODE: &[u16] = &[SET_INVOKED];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_INVOKED], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, _, _, _| c.is_summon_location(duel, LOCATION_EXTRA) && c.is_location(duel, LOCATION_MZONE)),
    ];
}

// An Invoked monster to banish with a FIRE monster of the opponent's, Invoked Sorath and Flame Manipulator, and another
// copy in the Extra Deck for its Fusion Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (43989315, MZONE),
    (43989315, GRAVE),
    (34460851, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(splimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_ADD_ATTRIBUTE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(
        duel,
        (ATTRIBUTE_EARTH | ATTRIBUTE_WATER | ATTRIBUTE_FIRE | ATTRIBUTE_WIND | ATTRIBUTE_DARK) as i32,
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, rmtg);
    e3.set_operation(duel, rmop);
    c.register_effect(duel, e3);
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, st: u32, _: u8, _: u8) -> bool {
    !e.get_handler(duel).is_location(duel, LOCATION_EXTRA) || st & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION
}

fn rmfilter1(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_INVOKED])
        && c.is_monster(duel)
        && (c.is_location(duel, LOCATION_GRAVE) || c.is_faceup(duel))
        && c.is_able_to_remove(duel)
        && aux::sp_elim_filter(c, duel, true, true)
}

fn rmfilter2(c: CardId, duel: &Duel, att: u32) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, att) && c.is_able_to_remove(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let locations = LOCATION_MZONE | LOCATION_GRAVE;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, locations) && chkc.is_controler(duel, tp) && rmfilter1(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(rmfilter1, tp, locations, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g1 = duel.select_target(tp, rmfilter1, tp, locations, 0, 1, 1, None)?.unwrap_or_default();
    let att = g1.get_first().map_or(0, |tc| tc.get_attribute(duel));
    g1.merge(duel.get_matching_group(|c, duel| rmfilter2(c, duel, att), tp, 0, LOCATION_MZONE, None));
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g1), g1.len() as u8, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        let mut tg = CardSet::from(tc);
        if tc.is_faceup(duel) {
            let att = tc.get_attribute(duel);
            tg.merge(duel.get_matching_group(|c, duel| rmfilter2(c, duel, att), tp, 0, LOCATION_MZONE, None));
        }
        duel.remove(tg, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
