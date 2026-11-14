// Goblin Bikers Gone Bonkers
use crate::prelude::*;

const ID: u32 = 46014517;

pub const LISTED_SERIES: &[u16] = &[SET_GOBLIN];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, attachtg);
    e1.set_operation(duel, attachop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let attachable = |tc: CardId, duel: &Duel| tc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, 1 - tp)
            && attachable(chkc, duel)
            && chkc.is_faceup(duel));
    }
    let ct = duel.get_matching_group_count(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_GOBLIN])),
        tp,
        LOCATION_MZONE,
        0,
        None,
    );
    if chk == 0 {
        return Ok(c.is_type(duel, TYPE_XYZ)
            && ct > 0
            && duel.check_remove_overlay_card(tp, 1, 0, 1, REASON_EFFECT)
            && duel.is_existing_target(aux::faceup_filter(attachable), tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
    duel.select_target(tp, aux::faceup_filter(attachable), tp, 0, LOCATION_MZONE, 1, ct as u8, None)?;
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.remove_overlay_card(tp, 1, 0, 1, 1, REASON_EFFECT)? != 1 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    let mut tg = duel.get_target_cards(e).unwrap_or_default();
    tg.remove_matching(duel, |tc, duel| tc.is_immune_to_effect(duel, e), None);
    if !tg.is_empty() {
        duel.overlay_full(c, tg, true)?;
    }
    Ok(())
}

fn tgctfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GOBLIN]) && c.is_type(duel, TYPE_XYZ) && c.is_faceup(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_grave(duel));
    }
    let ct = duel.get_matching_group_count(tgctfilter, tp, LOCATION_MZONE, 0, None);
    if chk == 0 {
        return Ok(ct > 0
            && duel.check_remove_overlay_card(tp, 1, 0, 3, REASON_EFFECT)
            && duel.is_existing_target(CardId::is_able_to_grave, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, CardId::is_able_to_grave, tp, 0, LOCATION_ONFIELD, 1, ct as u8, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, count, tp, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.remove_overlay_card(a.tp, 1, 0, 3, 3, REASON_EFFECT)? != 3 {
        return Ok(());
    }
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_grave(tg, REASON_EFFECT)?;
    }
    Ok(())
}
