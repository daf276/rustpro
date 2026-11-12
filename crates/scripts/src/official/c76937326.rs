// Daigusto Laplampilica
use super::c10755984::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 76937326;

// In the Extra Deck, Synchro Summoned with Angel Trumpeter (a Level 4 Tuner) and Gusto Griffin (Level 2), with Gusto monsters in the Deck
// to Summon and Daigusto Falcos in the Extra Deck to Synchro Summon with them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (87979586, MZONE),
    (42082363, MZONE),
    (42082363, MZONE),
    (336369, DECK),
    (336369, DECK),
    (42082363, DECK),
    (42082363, DECK),
    (34109611, EXTRA),
    (34109611, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(34109611)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, tgtg);
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel)
}

fn matfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GUSTO]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn filter(c: CardId, duel: &Duel, mg: Option<CardSet>, tp: u8, chk: bool) -> bool {
    c.is_set_card(duel, &[SET_GUSTO])
        && c.is_type(duel, TYPE_SYNCHRO)
        && (!chk || duel.get_location_count_from_ex(tp, tp, mg, c) > 0)
        && mg.is_none_or(|mg| c.is_synchro_summonable_full(duel, None, Some(mg), mg.len() as u16, mg.len() as u16))
}

fn rescon(exg: CardSet) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, mg, c| {
        let (_, stop) = aux::dncheck(duel, sg, mg, c);
        let (min, max) = (sg.len() as u16, sg.len() as u16);
        let res = sg.get_class_count(duel, |c, duel| i64::from(c.get_location(duel))) == sg.len()
            && exg.is_exists(
                duel,
                |sc, duel| sc.is_synchro_summonable_full(duel, None, Some(sg), min, max),
                1,
                None,
            );
        (res, stop)
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let exg = duel.get_matching_group(|c, duel| filter(c, duel, None, tp, false), tp, LOCATION_EXTRA, 0, None);
    let cancelcon = rescon(exg);
    let mg = duel.get_matching_group(|c, duel| matfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, None);
    let ft = duel.get_location_count(tp, LOCATION_MZONE).min(2);
    if chk == 0 {
        return Ok(ft > 1
            && duel.is_player_can_special_summon_count(tp, 2)
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && aux::select_unselect_group_check(duel, mg, 1, 2, &cancelcon));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 3, 0, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_attribute(duel, ATTRIBUTE_WIND));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let exg = duel.get_matching_group(|c, duel| filter(c, duel, None, tp, false), tp, LOCATION_EXTRA, 0, None);
    let cancelcon = rescon(exg);
    let mg = duel.get_matching_group(|c, duel| matfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, None);
    let g = aux::select_unselect_group_full(duel, mg, 1, 2, &cancelcon, tp, HINTMSG_SPSUMMON, Some(&cancelcon), false)?;
    if duel.get_location_count(tp, LOCATION_MZONE) < g.len() as i32
        || g.is_empty()
        || (duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() && g.len() > 1)
    {
        return Ok(());
    }
    for tc in g {
        duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        tc.register_effect(duel, e2);
    }
    duel.special_summon_complete()?;
    duel.break_effect()?;
    let syng = duel.get_matching_group(|c, duel| filter(c, duel, Some(g), tp, true), tp, LOCATION_EXTRA, 0, None);
    if !syng.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(c) = syng.select(duel, tp, 1, 1, None)?.get_first() {
            duel.synchro_summon_full(tp, c, None, Some(g), g.len() as u16, g.len() as u16)?;
        }
    }
    Ok(())
}

fn tgtg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.is_set_card(duel, &[SET_GUSTO]) && c.is_type(duel, TYPE_SYNCHRO) && c != e.get_handler(duel)
}
