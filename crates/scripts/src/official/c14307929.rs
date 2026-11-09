// Diabell, Queen of the White Forest
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 14307929;

// In the Extra Deck, Synchro Summoned with Dragunity Knight - Luin (Tuner) and Souls of the Forgotten, with Infinite Impermanence to add and send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (12496261, MZONE),
    (4920010, MZONE),
    (10045474, GRAVE),
    (10045474, HAND),
    (12496261, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        let e = a.e;
        e.get_handler(duel).is_synchro_summoned(duel) && e.get_label(duel) == 1
    });
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE);
    e1a.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1a.set_value(duel, Value::MaterialCheck(matcheck));
    e1a.set_label_object(duel, Some(e1));
    c.register_effect(duel, e1a);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.ep == 1 - a.tp);
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn matfilter(c: CardId, duel: &Duel, sc: CardId, tp: u8) -> bool {
    let sumtype = SUMMON_TYPE_SYNCHRO as u64 | MATERIAL_SYNCHRO;
    c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp) && c.is_type_full(duel, TYPE_TUNER, Some(sc), sumtype, tp)
}

fn matcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let mg = c.get_material(duel);
    let tp = e.get_handler_player(duel);
    let label = i64::from(!mg.is_empty() && mg.is_exists(duel, |mc, duel| matfilter(mc, duel, c, tp), 1, None));
    if let Some(e1) = e.get_label_object(duel) {
        e1.set_label(duel, label);
    }
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_spell_trap(duel)
        && c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, Some(c)),
            tp,
            LOCATION_EXTRA | LOCATION_GRAVE | LOCATION_REMOVED,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: Option<CardId>) -> bool {
    if !(c.is_type(duel, TYPE_TUNER)
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_level_below(duel, 7)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false))
    {
        return false;
    }
    if c.is_location(duel, LOCATION_EXTRA) {
        duel.get_location_count_from_ex(tp, tp, mc.map(CardSet::from), c) > 0
    } else {
        duel.get_mzone_count(tp, mc.map(CardSet::from)) > 0
    }
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| spcostfilter(c, duel, e, tp),
            tp,
            LOCATION_HAND | LOCATION_ONFIELD,
            0,
            1,
            None,
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spcostfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_ONFIELD,
        0,
        1,
        1,
        None,
    )?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        tp,
        (LOCATION_EXTRA | LOCATION_GRAVE | LOCATION_REMOVED) as i32,
    );
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, None)),
        tp,
        LOCATION_EXTRA | LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
