// S:P Little Knight
use crate::prelude::*;

const ID: u32 = 29301450;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_link_summoned(duel) && a.e.get_label(duel) == 1
    });
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_MATERIAL_CHECK);
    e2.set_value(duel, Value::MaterialCheck(valcheck));
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |_, a| a.ep == 1 - a.tp);
    e3.set_target(duel, tmprmtg);
    e3.set_operation(duel, tmprmop);
    c.register_effect(duel, e3);
}

fn removable(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let locations = LOCATION_ONFIELD | LOCATION_GRAVE;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, locations) && removable(chkc, duel));
    }
    if chk == 0 {
        e.set_label(duel, 0);
        return Ok(duel.is_existing_target(removable, tp, locations, locations, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, removable, tp, locations, locations, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(())
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let types = TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK;
    let player = e.get_handler_player(duel);
    let summon_type = SUMMON_TYPE_LINK.into();
    if c.get_material(duel)
        .is_exists(duel, |mc, duel| mc.is_type_full(duel, types, Some(c), summon_type, player), 1, None)
        && let Some(e1) = e.get_label_object(duel)
    {
        e1.set_label(duel, 1);
    }
}

fn tmprmfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_can_be_effect_target(duel, e) && removable(c, duel)
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    (sg.is_exists(duel, |c, duel| c.is_controler(duel, tp), 1, None), false)
}

fn tmprmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| tmprmfilter(c, duel, e), tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let check = |duel: &Duel, sg, _, _| rescon(duel, sg, tp);
    if chk == 0 {
        return Ok(g.len() >= 2 && aux::select_unselect_group_check(duel, g, 2, 2, &check));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &check, tp, HINTMSG_REMOVE)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(tg), 2, tp, 0);
    Ok(false)
}

fn tmprmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default().filter(duel, removable, None);
    if tg.len() == 2 {
        aux::remove_until::<aux::DefaultFieldReturnOp>(duel, tg, 0, REASON_EFFECT, PHASE_END, ID, e, tp)?;
    }
    Ok(())
}
