// PSY-Framelord Zeta
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 37192109;

// In the Extra Deck, Synchro Summoned with PSY-Framegear Alpha (a Level 1 Tuner) and PSY-Frame Driver (Level 6), with PSY-Frame cards in
// the Graveyard to add and the opponent's Special Summoned monsters (Iron Chain Dragon) to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (75425043, MZONE),
    (49036338, MZONE),
    (2810642, GRAVE),
    (2810642, GRAVE),
    (19974580, MZONE | OPPONENT),
    (19974580, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, 0);
    // The Lua's earlier SetHintTiming(0, TIMINGS_CHECK_MONSTER_E | TIMING_END_PHASE) is overwritten by this one.
    e1.set_hint_timing(duel, TIMING_SPSUMMON, TIMING_SPSUMMON);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_position(duel, POS_FACEUP_ATTACK) && c.is_able_to_remove(duel) && c.is_special_summoned(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && rmfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel) && duel.is_existing_target(rmfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g = duel
        .select_target(tp, rmfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?
        .unwrap_or_default();
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 2, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !c.is_relate_to_effect(duel, e) || !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let g = CardSet::from_iter([c, tc]);
    if duel.remove(g, 0, REASON_EFFECT | REASON_TEMPORARY)? == 0 {
        return Ok(());
    }
    let fid = c.get_field_id(duel);
    let on_standby = duel.is_turn_player(tp) && duel.is_phase(PHASE_STANDBY);
    let rct = if on_standby { 2 } else { 1 };
    let og = duel.get_operated_group();
    for oc in og {
        let turn = if oc.is_controler(duel, tp) {
            RESET_SELF_TURN
        } else {
            RESET_OPPO_TURN
        };
        let reset = RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | turn;
        oc.register_flag_effect_full(duel, ID, reset, 0, rct, i64::from(fid), 0);
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_count_limit(duel, 1, 0);
    e1.set_label(duel, i64::from(fid));
    e1.set_label_object_group(duel, Some(og));
    e1.set_condition_mut(duel, retcon);
    e1.set_operation(duel, retop);
    if on_standby {
        e1.set_reset_full(duel, RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN, 2);
        e1.set_value(duel, duel.get_turn_count());
    } else {
        e1.set_reset(duel, RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN);
        e1.set_value(duel, 0);
    }
    duel.register_effect(e1, tp);
    Ok(())
}

fn retfilter(c: CardId, duel: &Duel, fid: u32) -> bool {
    c.get_flag_effect_label(duel, ID) == Some(i64::from(fid))
}

fn retcon(duel: &mut Duel, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    if duel.is_turn_player(1 - tp) || matches!(e.get_value(duel), Value::Int(turn) if turn == duel.get_turn_count()) {
        return false;
    }
    let fid = e.get_label(duel) as u32;
    let g = e.get_label_object_group(duel).unwrap_or_default();
    if !g.is_exists(duel, |c, duel| retfilter(c, duel, fid), 1, None) {
        e.reset(duel);
        return false;
    }
    true
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let fid = e.get_label(duel) as u32;
    let g = e.get_label_object_group(duel).unwrap_or_default();
    let mut sg = g.filter(duel, |c, duel| retfilter(c, duel, fid), None);
    if sg.len() > 1
        && sg.get_class_count(duel, |c, duel| i64::from(c.get_previous_controler(duel))) == 1
        && let Some(first) = sg.get_first()
    {
        let ft = duel.get_location_count(first.get_previous_controler(duel), LOCATION_MZONE);
        if ft == 1 {
            duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 0));
            if let Some(tc) = sg.select(duel, tp, 1, 1, None)?.get_first() {
                duel.return_to_field(tc)?;
                sg.remove(tc);
            }
        }
    }
    for tc in sg {
        duel.return_to_field(tc)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PSY_FRAME]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(c.is_able_to_extra(duel) && duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let tc = duel.get_first_target();
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? != 0
        && c.is_location(duel, LOCATION_EXTRA)
        && let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
