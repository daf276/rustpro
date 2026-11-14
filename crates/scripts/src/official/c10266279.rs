// Junoldo the Shadespirit Power Patron
// The script's spfilter, spcon, sptg and spop belong to no effect and are left out.
use crate::prelude::*;

const ID: u32 = 10266279;

pub const LISTED_SERIES: &[u16] = &[SET_POWER_PATRON, SET_ELFNOTE];

const JUNORA: u32 = 5914858;

pub const LISTED_NAMES: &[u32] = &[ID, JUNORA];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DRAW + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, pdescon);
    e1.set_target(duel, pdestg);
    e1.set_operation(duel, pdesop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, |duel, a, chk| Cost::replaceable(duel, a, chk, mdescost));
    e2.set_target(duel, mdestg);
    e2.set_operation(duel, mdesop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_DECK);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn pdesconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_controler(duel, tp) && c.is_set_card(duel, &[SET_POWER_PATRON, SET_ELFNOTE])
}

fn pdescon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| pdesconfilter(c, duel, a.tp), 1, None)
}

fn pdestg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 2));
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn pdesop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) && duel.destroy(c, REASON_EFFECT)? > 0 && duel.draw(tp, 2, REASON_EFFECT)? == 2 {
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn junoraspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, hc: Option<CardId>) -> bool {
    c.is_code(duel, JUNORA)
        && duel.get_location_count_from_ex(tp, tp, hc.map(CardSet::from), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_SYNCHRO, tp, false, false)
}

pub fn mdescost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let g = duel.get_decktop_group(a.tp, 3);
    if chk == 0 {
        return Ok(g.is_exists(duel, |c, duel| c.is_able_to_remove_as_cost_full(duel, POS_FACEDOWN), 3, None));
    }
    duel.disable_shuffle_check(true);
    duel.remove(g, POS_FACEDOWN, REASON_COST)?;
    Ok(false)
}

fn mdestg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        let filter = |sc, duel: &Duel| junoraspfilter(sc, duel, e, tp, Some(c));
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn mdesop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.destroy(c, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = |sc, duel: &Duel| junoraspfilter(sc, duel, e, tp, None);
    let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    sc.set_material(duel, None);
    if duel.special_summon(sc, SUMMON_TYPE_SYNCHRO, tp, tp, false, false, POS_FACEUP)? > 0 {
        sc.complete_procedure(duel);
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_EXTRA) && c.is_faceup(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_POWER_PATRON, SET_ELFNOTE]) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_EXTRA | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let filter = aux::necro_valley_filter(thfilter);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
