// D/D/D Vice King Requiem
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, PZONE, SZONE};

const ID: u32 = 25857977;

pub const LISTED_SERIES: &[u16] = &[SET_DDD, SET_DARK_CONTRACT];
// In the Pendulum Zone with Dark Contract with the Witch to destroy; on the field with D/D Pandora as Link Materials for D/D/D Abyss
// King Gilgamesh.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, MZONE),
    (9765723, SZONE),
    (9765723, SZONE),
    (9765723, SZONE),
    (32146097, MZONE),
    (32146097, MZONE),
    (9024198, EXTRA),
    (9024198, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesGranted(aux::stringid(ID, 4)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 1, 0);
    e2.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_set_card(duel, &[SET_DDD]));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_EVENT_PLAYER);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, efcon);
    e3.set_operation(duel, efop);
    c.register_effect(duel, e3);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DARK_CONTRACT])
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && desfilter(chkc, duel));
    }
    let g = duel.get_matching_group(
        |c, duel| desfilter(c, duel) && c.is_can_be_effect_target(duel, e),
        tp,
        LOCATION_ONFIELD,
        0,
        None,
    );
    let c = e.get_handler(duel);
    let rescon = aux::chkf_mmz(tp, 1);
    if chk == 0 {
        return Ok(
            c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && aux::select_unselect_group_check(duel, g, 1, g.len(), &rescon)
        );
    }
    let sg = aux::select_unselect_group(duel, g, 1, g.len(), &rescon, tp, HINTMSG_DESTROY)?;
    duel.set_target_card(sg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sg), sg.len() as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let g = duel.get_target_cards(e).unwrap_or_default();
    if g.is_empty() {
        return Ok(());
    }
    let mut ct = duel.destroy(g, REASON_EFFECT)?;
    duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    if duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        // At Level 1 the Lua reads its global `sel`, whatever any script last left in it.
        let level_one = c.get_level(duel) == 1;
        unported!(duel, level_one);
        if level_one {
            duel.select_option(tp, &[aux::stringid(ID, 2)])?;
        } else if duel.select_option(tp, &[aux::stringid(ID, 2), aux::stringid(ID, 3)])? == 1 {
            ct = -ct;
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, ct);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn efcon(duel: &Duel, a: &Args) -> bool {
    let p = a.e.get_handler(duel);
    a.r & (REASON_FUSION | REASON_SYNCHRO | REASON_XYZ | REASON_LINK) > 0
        && p.is_previous_location(duel, LOCATION_ONFIELD)
        && p.get_reason_card(duel).is_some_and(|rc| rc.is_set_card(duel, &[SET_DDD]))
}

fn efop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(rc) = c.get_reason_card(duel) else { return Ok(()) };
    let e1 = duel.create_effect(rc);
    e1.set_description(duel, aux::stringid(ID, 4));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_RECOVER + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    rc.register_effect_full(duel, e1, true);
    let e2 = duel.create_effect(rc);
    e2.set_description(duel, aux::stringid(ID, 5));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CLIENT_HINT);
    e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    rc.register_effect_full(duel, e2, true);
    if !rc.is_type(duel, TYPE_EFFECT) {
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_ADD_TYPE);
        e3.set_value(duel, TYPE_EFFECT as i32);
        e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        rc.register_effect_full(duel, e3, true);
    }
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DARK_CONTRACT]) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_ONFIELD) && chkc.is_able_to_deck(duel));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|_, _| true, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None)
                && duel.is_existing_matching_card(tdfilter, tp, LOCATION_GRAVE | LOCATION_ONFIELD, 0, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |_, _| true, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_DESTROY, g, count, tp, 0);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_ONFIELD as i32);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 1000);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let sc = duel
        .select_matching_card(tp, tdfilter, tp, LOCATION_GRAVE | LOCATION_ONFIELD, 0, 1, 1, None)?
        .get_first();
    // The Lua errors on a nil card when the Dark Contract left in the meantime.
    unported!(duel, sc.is_none());
    let Some(sc) = sc else { return Ok(()) };
    if duel.send_to_deck(sc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_DECK | LOCATION_EXTRA)
        && let Some(tc) = tc.filter(|tc| tc.is_relate_to_effect(duel, e))
        && duel.destroy(tc, REASON_EFFECT)? > 0
    {
        duel.break_effect()?;
        duel.recover(tp, 1000, REASON_EFFECT)?;
    }
    Ok(())
}
