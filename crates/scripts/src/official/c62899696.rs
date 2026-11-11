// Speedroid Maliciousmagnet
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 62899696;

// Clear Wing Synchro Dragon (Level 7) in the Extra Deck to Synchro Summon with the Level 6 Frostosaurus of the opponent.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (82044279, EXTRA),
    (6631034, MZONE | OPPONENT),
    (6631034, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1)), Check::SpecialSummonedCode(82044279)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) == 0);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(a.tp) && duel.is_main_phase());
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

fn filter(tc: CardId, duel: &Duel, c: CardId, tp: u8) -> bool {
    if !tc.is_faceup(duel) || !tc.is_can_be_synchro_material_full(duel, None, None, None) {
        return false;
    }
    duel.check_mut(|duel| {
        c.register_flag_effect(duel, ID, 0, 0, 1);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SYNCHRO_MATERIAL);
        tc.register_effect_full(duel, e1, true);
        let mg = CardSet::from_iter([c, tc]);
        let res = duel.is_existing_matching_card(|sc, duel| synfilter(sc, duel, mg), tp, LOCATION_EXTRA, 0, 1, None);
        c.reset_flag_effect(duel, ID);
        e1.reset(duel);
        res
    })
}

fn synfilter(c: CardId, duel: &Duel, mg: CardSet) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WIND) && c.is_synchro_summonable(duel, None, Some(mg))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let target = |tc, duel: &Duel| filter(tc, duel, c, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && target(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(target, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, target, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    if let Some(tc) = tc
        && c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        c.register_flag_effect(duel, ID, 0, 0, 1);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SYNCHRO_MATERIAL);
        tc.register_effect_full(duel, e1, true);
        let mg = CardSet::from_iter([c, tc]);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |sc, duel| synfilter(sc, duel, mg), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first() {
            let e3 = duel.create_effect(c);
            e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
            e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
            e3.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
            e3.set_operation(duel, regop);
            e3.set_label_object(duel, Some(e1));
            sc.register_effect_full(duel, e3, true);
            let e4 = e3.clone_effect(duel);
            e4.set_code(duel, EVENT_SPSUMMON_NEGATED);
            sc.register_effect_full(duel, e4, true);
            duel.synchro_summon(tp, sc, None, Some(mg))?;
        } else {
            c.reset_flag_effect(duel, ID);
            e1.reset(duel);
        }
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    e.get_owner(duel).reset_flag_effect(duel, ID);
    if let Some(e1) = e.get_label_object(duel) {
        e1.reset(duel);
    }
    e.reset(duel);
    Ok(())
}
