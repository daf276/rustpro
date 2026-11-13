// Horoscope Sorcerer, the Stargazer Magician
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, PZONE};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 1186447;

// In the hand with a Magician Pendulum Monster (Stargazer Magician) to discard and on the field, Supreme King Gate Magician (2500
// ATK Pendulum) in the Deck and a face-up Pendulum in the Extra Deck, and the opponent's Spells to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, PZONE),
    (94415058, HAND),
    (94415058, MZONE),
    (14513273, DECK),
    (14513273, DECK),
    (83764718, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::Message(MSG_SELECT_EFFECTYN),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAIN_SOLVING);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_condition(duel, discon);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_cost(duel, |duel, a, chk| Cost::discard_full(duel, a, chk, spcostfilter, true, (1, 1)));
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, deckthtg);
    e3.set_operation(duel, deckthop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOHAND);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
}

fn disconfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_MAGICIAN]) && c.is_pendulum_monster(duel)) || c.is_set_card(duel, &[SET_ODD_EYES])) && c.is_faceup(duel)
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp
        && a.re.is_some_and(|re| re.is_spell_effect(duel))
        && duel.is_chain_disablable(a.ev as u8)
        && !duel.has_flag_effect(a.tp, ID)
        && duel.is_existing_matching_card(disconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !duel.select_effect_yes_no_full(tp, c, aux::stringid(ID, 3))? {
        return Ok(());
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    duel.hint(HINT_CARD, 0, u64::from(ID));
    if duel.negate_effect(a.ev as u8) {
        duel.break_effect()?;
        duel.destroy(c, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_MAGICIAN]) && c.is_pendulum_monster(duel)) || c.is_set_card(duel, &[SET_ODD_EYES, SET_PERFORMAPAL]))
        && c.is_monster(duel)
}

fn deckthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_pendulum_monster(duel) && c.is_attack(duel, &[2500]) && c.is_able_to_hand(duel)
}

fn deckthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, deckthfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn deckthop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, deckthfilter, LOCATION_DECK)
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_PENDULUM)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && chkc.is_type(duel, TYPE_PENDULUM) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(desfilter, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, desfilter, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn extrathfilter(c: CardId, duel: &Duel) -> bool {
    c.is_pendulum_monster(duel) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && duel.destroy(tc, REASON_EFFECT)? > 0
        && duel.is_existing_matching_card(extrathfilter, tp, LOCATION_EXTRA, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 4))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, extrathfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}
