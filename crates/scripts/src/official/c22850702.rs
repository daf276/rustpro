// Chaos Angel
use crate::prelude::*;

const ID: u32 = 22850702;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_attribute_full(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK, Some(sc), sumtype, tp)
    }

    fn sub1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK, Some(sc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_label(duel, ATTRIBUTE_LIGHT.into());
    e2.set_condition(duel, immbttlcon);
    e2.set_target_card(duel, |duel, _, c| c.is_type(duel, TYPE_SYNCHRO));
    e2.set_value(duel, Value::Effect(immval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_label(duel, ATTRIBUTE_DARK.into());
    e3.set_condition(duel, immbttlcon);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_MATERIAL_CHECK);
    e4.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e4);
}

fn immval(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    if !te.is_activated(duel) {
        return false;
    }
    let trig_p = duel.get_chain_info::<Option<u8>>(0, CHAININFO_TRIGGERING_PLAYER);
    let trig_typ = duel.get_chain_info::<Option<u32>>(0, CHAININFO_TRIGGERING_TYPE);
    trig_p == Some(1 - e.get_handler_player(duel)) && trig_typ.is_some_and(|trig_typ| trig_typ & TYPE_MONSTER > 0)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let removable = |c: CardId, duel: &Duel| c.is_able_to_remove(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && removable(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(removable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, removable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn immbttlcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.has_flag_effect(duel, ID) && c.get_flag_effect_label(duel, ID).unwrap_or(0) & a.e.get_label(duel) > 0
}

fn valcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let attr = c
        .get_material(duel)
        .get_bitwise_or(duel, |c, duel| c.get_original_attribute(duel).into()) as u32
        & (ATTRIBUTE_LIGHT | ATTRIBUTE_DARK);
    if attr == 0 {
        return;
    }
    let index = match attr {
        ATTRIBUTE_LIGHT => 1,
        ATTRIBUTE_DARK => 2,
        _ => 3,
    };
    let reset = RESET_EVENT | (RESETS_STANDARD & !(RESET_TOFIELD | RESET_LEAVE | RESET_TEMP_REMOVE));
    c.register_flag_effect_full(duel, ID, reset, EFFECT_FLAG_CLIENT_HINT, 1, attr.into(), aux::stringid(ID, index));
}
