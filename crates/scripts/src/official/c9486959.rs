// Onibimaru Soul Sweeper
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 9486959;

// Twice in the Extra Deck, Xyz Summoned with Level 2 Normal Monsters, with the opponent's Sangans to banish and its Raigeki to
// destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (756652, MZONE),
    (15401633, MZONE),
    (64511793, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 1));
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, tdcon);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EVENT_LEAVE_FIELD_P);
    e3.set_operation(duel, matpreop);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_remove(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.remove(tc, 0, REASON_EFFECT | REASON_TEMPORARY)? > 0 {
        tc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_count_limit(duel, 1, 0);
        e1.set_condition(duel, retcon);
        e1.set_operation(duel, retop);
        e1.set_reset(duel, RESET_PHASE | PHASE_END | RESET_OPPO_TURN);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn retcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp)
        && a.e
            .get_label_object_group(duel)
            .and_then(|g| g.get_first())
            .is_some_and(|tc| tc.get_flag_effect(duel, ID) > 0)
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_SELECTMSG, 1 - a.tp, HINTMSG_ZONE);
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.return_to_field(tc)?;
    }
    Ok(())
}

fn tdcon(duel: &Duel, a: &Args) -> bool {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    a.rp == 1 - tp && c.is_previous_controler(duel, tp) && c.is_previous_location(duel, LOCATION_ONFIELD)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_monster(duel) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = a.e.get_label_object_group(duel).map_or(0, |g| g.len());
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_target(tdfilter, tp, LOCATION_REMOVED, LOCATION_REMOVED, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let tg = duel.select_target(tp, tdfilter, tp, LOCATION_REMOVED, LOCATION_REMOVED, 1, ct as u8, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, tg, tg.map_or(0, |tg| tg.len()) as u8, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}

fn matpreop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = a.e.get_handler(duel).get_overlay_group(duel);
    if let Some(e2) = a.e.get_label_object(duel) {
        e2.set_label_object_group(duel, Some(g));
    }
    Ok(())
}
