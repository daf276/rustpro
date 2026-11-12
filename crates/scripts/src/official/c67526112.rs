// Rapid Trigger
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 67526112;

// Dragoness the Wicked Knight is the Fusion Monster, with its two materials on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (70681994, EXTRA), (53153481, MZONE), (33064647, MZONE)];

pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(70681994)];

struct Params;

impl FusionSummonParams for Params {
    fn matfilter(c: CardId, duel: &Duel, e: EffectId, _: u8) -> bool {
        c.is_on_field(duel) && c.is_destructable_full(duel, e) && !c.is_immune_to_effect(duel, e)
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(extraop);

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, _: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        let Some(tc) = tc else { return Ok(()) };
        if chk != 1 {
            return Ok(());
        }
        let handler = e.get_handler(duel);
        let e1 = duel.create_effect(handler);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, Value::Card(atlimit));
        tc.register_effect_full(duel, e1, true);
        let e1 = duel.create_effect(handler);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect_full(duel, e1, true);
        let e3 = duel.create_effect(handler);
        e3.set_description(duel, aux::stringid(ID, 0));
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_IMMUNE_EFFECT);
        e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CLIENT_HINT);
        e3.set_range(duel, LOCATION_MZONE);
        e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e3.set_value(duel, Value::Effect(efilter));
        tc.register_effect_full(duel, e3, true);
        Ok(())
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        let dg = duel.get_fusion_material(a.tp).filter(duel, CardId::is_on_field, None);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), 2, a.tp, LOCATION_ONFIELD as i32);
        Ok(false)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON + CATEGORY_DESTROY);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn extraop(duel: &mut Duel, _: EffectId, _: CardId, _: u8, sg: &mut CardSet) -> Script<()> {
    let res = duel.destroy(*sg, REASON_EFFECT + REASON_MATERIAL + REASON_FUSION)? == sg.len() as i32;
    sg.clear();
    // The library ends the operation when extraop returns false.
    unported!(duel, !res);
    Ok(())
}

fn atlimit(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    i32::from(!(c.is_special_summoned(duel) && c.is_summon_location(duel, LOCATION_EXTRA)))
}

fn efilter(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    let tc = te.get_owner(duel);
    tc.is_special_summoned(duel)
        && tc.is_summon_location(duel, LOCATION_EXTRA)
        && tc != e.get_handler(duel)
        && te.is_monster_effect(duel)
        && te.is_activated(duel)
        && te.get_activate_location(duel) == LOCATION_MZONE
}
