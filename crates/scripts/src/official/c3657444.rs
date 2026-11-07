// Cyber Valley
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 3657444;

// Twice on the field with Cyber Dragons on the field, in the hand and in the Graveyard, and attackers of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (CARD_CYBER_DRAGON, MZONE),
    (CARD_CYBER_DRAGON, HAND),
    (CARD_CYBER_DRAGON, GRAVE),
    (CARD_CYBER_DRAGON, MZONE | OPPONENT),
    (CARD_CYBER_DRAGON, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e1.set_cost(duel, Cost::self_banish);
    e1.set_target(duel, drawtg1);
    e1.set_operation(duel, drawop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE + CATEGORY_DRAW);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, drawtg2);
    e2.set_operation(duel, drawop2);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_REMOVE + CATEGORY_TODECK);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target(duel, tdtg);
    e3.set_operation(duel, tdop);
    c.register_effect(duel, e3);
}

fn drawtg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_player_can_draw(a.tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, a.tp, 1);
    Ok(false)
}

fn drawop1(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.draw(a.tp, 1, REASON_EFFECT)? > 0 {
        duel.break_effect()?;
        duel.skip_phase(1 - a.tp, PHASE_BATTLE, RESET_PHASE | PHASE_BATTLE_STEP, 1);
    }
    Ok(())
}

fn drawtg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp)
            && chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_faceup(duel)
            && chkc.is_able_to_remove(duel));
    }
    let filter = aux::faceup_filter(CardId::is_able_to_remove);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel)
            && duel.is_player_can_draw(tp, 2)
            && duel.is_existing_target(&filter, tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, a.e.get_description(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g = duel
        .select_target(tp, &filter, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?
        .unwrap_or_default();
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 2, 0, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    Ok(false)
}

fn drawop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_facedown(duel) || !c.is_relate_to_effect(duel, e) || !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if duel.remove(CardSet::from_iter([c, tc]), POS_FACEUP, REASON_EFFECT)? == 2 {
        duel.break_effect()?;
        duel.draw(tp, 2, REASON_EFFECT)?;
    }
    Ok(())
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_deck, tp, LOCATION_GRAVE, 0, 1, None)
            && duel.is_existing_matching_card(CardId::is_able_to_remove, tp, LOCATION_HAND, 0, 1, None)
            && c.is_able_to_remove(duel));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, a.e.get_description(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    duel.select_target(tp, CardId::is_able_to_deck, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 2, 0, (LOCATION_MZONE | LOCATION_HAND) as i32);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut hg = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if hg.is_empty() {
        return Ok(());
    }
    hg.insert(c);
    if duel.remove(hg, POS_FACEUP, REASON_EFFECT)? == 2
        && let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.break_effect()?;
        duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, tc)?;
    }
    Ok(())
}
