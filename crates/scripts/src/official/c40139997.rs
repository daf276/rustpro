// Ruddy Rose Dragon
use crate::prelude::*;

const ID: u32 = 40139997;

pub const LISTED_NAMES: &[u32] = &[CARD_BLACK_ROSE_DRAGON];

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
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, condition);
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel, sc: CardId) -> bool {
    (c.is_race(duel, RACE_PLANT) && c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), SUMMON_TYPE_SYNCHRO.into(), PLAYER_NONE))
        || c.is_code(duel, CARD_BLACK_ROSE_DRAGON)
}

fn removable(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(removable, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None));
    }
    let g = duel.get_matching_group(removable, tp, LOCATION_GRAVE, LOCATION_GRAVE, None);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, 0, 0);
    let c = a.e.get_handler(duel);
    if c.get_material(duel).is_exists(duel, |mc, duel| matfilter(mc, duel, c), 1, None) {
        let g2 = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
        duel.set_possible_operation_info(0, CATEGORY_DESTROY, Some(g2), g2.len() as u8, 0, 0);
    }
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(removable, tp, LOCATION_GRAVE, LOCATION_GRAVE, None);
    if g.is_empty() {
        return Ok(());
    }
    duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    let c = a.e.get_handler(duel);
    let g2 = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    if !g2.is_empty()
        && c.get_material(duel).is_exists(duel, |mc, duel| matfilter(mc, duel, c), 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        duel.destroy(g2, REASON_EFFECT)?;
    }
    Ok(())
}

fn condition(duel: &Duel, a: &Args) -> bool {
    if a.tp == a.ep || a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) || !duel.is_chain_negatable(a.ev as u8) {
        return false;
    }
    duel.get_operation_info(a.ev as u8, CATEGORY_DESTROY)
        .is_some_and(|info| info.op_cards.is_some() || info.op_count > 0)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    let locations = LOCATION_EXTRA | LOCATION_GRAVE;
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, locations as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_BLACK_ROSE_DRAGON)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && ((c.is_location(duel, LOCATION_GRAVE) && duel.get_location_count(tp, LOCATION_MZONE) > 0)
            || (c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, None, c) > 0))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.negate_activation(a.ev as u8) {
        return Ok(());
    }
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.get_matching_group(filter, tp, LOCATION_GRAVE | LOCATION_EXTRA, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
