// Turbo-Tainted Hot Rod GT19
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 16769305;

// Face-down on the field to be flipped by the opponent's Elfnote Lucina, with Bunilla (Level 1) of either player as the
// Synchro material with it for Armory Arm (Level 4).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (69380702, MZONE),
    (69380702, MZONE | OPPONENT),
    (29071332, EXTRA),
    (29071332, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e0.set_code(duel, EVENT_FLIP);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_operation(duel, flipop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, lvltg);
    e1.set_operation(duel, lvlop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        duel.is_main_phase() && a.e.get_handler(duel).has_flag_effect(duel, ID)
    });
    e2.set_target(duel, syncsumtg);
    e2.set_operation(duel, syncsumop);
    c.register_effect(duel, e2);
}

fn flipop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
    Ok(())
}

fn lvltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_LVRANK);
    let level = u64::from(c.get_level(duel));
    let lv = duel.announce_number(tp, &(1..=9).filter(|&n| n != level).collect::<Vec<_>>())?;
    duel.set_target_param(lv as u32);
    duel.set_operation_info(0, CATEGORY_LVCHANGE, Some(c.into()), 1, tp, lv as i32);
    Ok(false)
}

fn lvlop(duel: &mut Duel, a: &Args) -> Script<()> {
    let lv: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) && lv != c.get_level(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, lv as i32);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn synchro_summonable(duel: &Duel, tp: u8, mg: CardSet) -> bool {
    duel.is_existing_matching_card(
        |sc, duel| sc.is_synchro_summonable(duel, None, Some(mg)),
        tp,
        LOCATION_EXTRA,
        0,
        1,
        None,
    )
}

fn filter(c: CardId, duel: &Duel, this: CardId, tp: u8) -> bool {
    if !(c.is_faceup(duel) && c.has_level(duel) && c.is_can_be_synchro_material_full(duel, None, None, None)) {
        return false;
    }
    let mg = CardSet::from_iter([this, c]);
    if c.is_controler(duel, tp) {
        return synchro_summonable(duel, tp, mg);
    }
    duel.check_mut(|duel| {
        let e1 = duel.create_effect(this);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SYNCHRO_MATERIAL);
        c.register_effect_full(duel, e1, true);
        let res = synchro_summonable(duel, tp, mg);
        e1.reset(duel);
        res
    })
}

fn syncsumtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel, c, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(
            |tc, duel| filter(tc, duel, c, tp),
            tp,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            Some(c.into()),
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(
        tp,
        |tc, duel| filter(tc, duel, c, tp),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        1,
        Some(c.into()),
    )?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn syncsumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e)) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    if tc.is_controler(duel, 1 - tp) {
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SYNCHRO_MATERIAL);
        tc.register_effect_full(duel, e1, true);
    }
    let mg = CardSet::from_iter([c, tc]);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |sc, duel| sc.is_synchro_summonable(duel, None, Some(mg)),
        tp,
        LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first() {
        if tc.is_controler(duel, 1 - tp) {
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
            e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
            e2.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
            e2.set_operation(duel, regop);
            e2.set_label_object(duel, Some(e1));
            sc.register_effect_full(duel, e2, true);
            let e3 = e2.clone_effect(duel);
            e3.set_code(duel, EVENT_SPSUMMON_NEGATED);
            sc.register_effect_full(duel, e3, true);
        }
        duel.synchro_summon(tp, sc, None, Some(mg))?;
    } else {
        e1.reset(duel);
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(e1) = a.e.get_label_object(duel) {
        e1.reset(duel);
    }
    a.e.reset(duel);
    Ok(())
}
