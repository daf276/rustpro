// Invoked Oceanus
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 6772168;
pub const MATERIAL_SETCODE: &[u16] = &[SET_ALEISTER];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_ALEISTER], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| {
            c.is_attribute_full(duel, ATTRIBUTE_DARK | ATTRIBUTE_WATER, Some(fc), sumtype, tp)
        }),
    ];
}

// A copy in the Graveyard for the direct attack effect, another on the field, and the opponent's monster to attack.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, MZONE), (ID, EXTRA), (43989315, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e1.set_range(duel, LOCATION_MMZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, e, c| i32::from(c != e.get_handler(duel))));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(
        duel,
        EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_RANGE + EFFECT_FLAG_IGNORE_IMMUNE,
    );
    e2.set_code(duel, EFFECT_TO_GRAVE_REDIRECT);
    e2.set_range(duel, LOCATION_EMZONE);
    e2.set_target_card(duel, rmtarget);
    e2.set_target_range(duel, LOCATION_ALL, LOCATION_ALL);
    e2.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, |duel, _| duel.is_able_to_enter_bp());
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, diratktg);
    e3.set_operation(duel, diratkop);
    c.register_effect(duel, e3);
}

fn rmtarget(duel: &Duel, e: EffectId, c: CardId) -> bool {
    let tp = e.get_handler_player(duel);
    !(c.is_owner(duel, tp) || c.is_spell_trap(duel) || c.is_location(duel, LOCATION_OVERLAY)) && duel.is_player_can_remove(tp, Some(c))
}

fn diratktgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_fusion_monster(duel) && c.is_faceup(duel) && c.is_has_effect(duel, EFFECT_DIRECT_ATTACK).is_none()
}

fn diratktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && diratktgfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(diratktgfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_APPLYTO);
    duel.select_target(tp, diratktgfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn diratkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3205);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_DIRECT_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
