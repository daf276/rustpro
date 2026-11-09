// Baxia, Brightness of the Yang Zing
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 83755611;

// In the Extra Deck, Synchro Summoned with Chiwen and Metaphys Armed Dragon, with Wyrm Normal Monsters to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (61488417, MZONE), (89189982, MZONE), (18108166, GRAVE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_WYRM, Some(sc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn matcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let ct = c
        .get_material(duel)
        .filter(duel, |c, duel| c.is_race(duel, RACE_WYRM), None)
        .get_class_count(duel, |c, duel| c.get_original_attribute(duel).into());
    e.set_label(duel, ct as i64);
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_able_to_deck(duel));
    }
    let ct = a.e.get_label_object(duel).map_or(0, |e1| e1.get_label(duel));
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| c.is_able_to_deck(duel), tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None) && ct > 0);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(
        tp,
        |c, duel| c.is_able_to_deck(duel),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        ct as u8,
        None,
    )?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_TODECK, g, count, 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| desfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, None)
                && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g1 = duel.select_target(tp, |c, duel| desfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g2 = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g1, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g2, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let target = |duel: &Duel, category| duel.get_operation_info(0, category).and_then(|info| info.op_cards?.get_first());
    unported!(
        duel,
        target(duel, CATEGORY_DESTROY).is_none() || target(duel, CATEGORY_SPECIAL_SUMMON).is_none()
    );
    let (Some(tc1), Some(tc2)) = (target(duel, CATEGORY_DESTROY), target(duel, CATEGORY_SPECIAL_SUMMON)) else {
        return Ok(());
    };
    if tc1.is_relate_to_effect(duel, a.e) && duel.destroy(tc1, REASON_EFFECT)? > 0 && tc2.is_relate_to_effect(duel, a.e) {
        duel.special_summon(tc2, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
