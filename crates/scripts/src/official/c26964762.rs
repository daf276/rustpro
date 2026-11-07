// Destiny HERO - Dark Angel
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 26964762;

// In the hand with Destiny HERO monsters (Malicious, Denier) in the GY, a Normal Spell in each Deck, and a Spell to
// activate while it is on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (9411399, GRAVE),
    (9411399, GRAVE),
    (16605586, GRAVE),
    (16605586, GRAVE),
    (CARD_POLYMERIZATION, HAND),
    (ID, MZONE),
    (55144522, DECK),
    (55144522, DECK | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_FIELD);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e3.set_cost(duel, deckcost);
    e3.set_target(duel, decktg);
    e3.set_operation(duel, deckop);
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(|c, duel| c.is_set_card(duel, &[SET_DESTINY_HERO]), a.tp, LOCATION_GRAVE, 0, 3, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DESTINY_HERO])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, 1 - tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.special_summon(tc, 0, tp, 1 - tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if a.rp == a.tp && re.is_spell_effect(duel) {
        let rc = re.get_handler(duel);
        if duel.negate_effect(a.ev as u8) && rc.is_relate_to_effect(duel, re) {
            duel.destroy(rc, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DESTINY_HERO]) && c.is_able_to_remove_as_cost(duel)
}

fn deckcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(Cost::self_banish(duel, a, 0)? && duel.is_existing_matching_card(cfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g = duel.select_matching_card(tp, cfilter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    g.insert(c);
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn decktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk != 0
        || (duel.get_field_group_count(tp, 0, LOCATION_DECK) > 0
            && duel.is_existing_matching_card(CardId::is_normal_spell, tp, LOCATION_DECK, 0, 1, None)))
}

fn deckop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let g1 = duel.select_matching_card(tp, CardId::is_normal_spell, tp, LOCATION_DECK, 0, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, 1 - tp, aux::stringid(ID, 3));
    let g2 = duel.select_matching_card(1 - tp, CardId::is_normal_spell, 1 - tp, LOCATION_DECK, 0, 1, 1, None)?;
    for (p, g) in [(tp, g1), (1 - tp, g2)] {
        if let Some(tc) = g.get_first() {
            duel.shuffle_deck(p)?;
            duel.move_sequence(tc, 0)?;
            duel.confirm_decktop(p, 1)?;
        }
    }
    Ok(())
}
