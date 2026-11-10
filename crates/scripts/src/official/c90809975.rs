// Toadally Awesome
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 90809975;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_AQUA, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with Level 2 Aqua Normal Monsters (Psychic Kappa, Zarigun, Hitodenchak) and Aqua monsters
// to send, Frogs in the Deck, a Water monster in the Graveyard, and the opponent's Spells to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (7892180, MZONE),
    (10598400, MZONE),
    (46718686, MZONE),
    (7892180, MZONE),
    (7892180, HAND),
    (9126351, DECK),
    (46239604, DECK),
    (68638985, GRAVE),
    (55144522, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_FIELD);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, negcost);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 3));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_FROG]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if a.ep == a.tp || a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    (re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)) && duel.is_chain_negatable(a.ev as u8)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_AQUA) && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel)) && c.is_able_to_grave_as_cost(duel)
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, cfilter, tp, LOCATION_MZONE | LOCATION_HAND, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let e = a.e;
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    let Some(re) = a.re else { return Ok(false) };
    let dc = re.get_handler(duel);
    if dc.is_relate_to_effect(duel, re) {
        if dc.is_destructable(duel) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
        }
        let cat = e.get_category(duel);
        if dc.is_monster_card(duel) {
            e.set_category(duel, cat | CATEGORY_SPECIAL_SUMMON);
            duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(dc.into()), 1, 0, LOCATION_GRAVE as i32);
        } else {
            e.set_category(duel, cat & !CATEGORY_SPECIAL_SUMMON);
        }
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if !duel.negate_activation(a.ev as u8) {
        return Ok(());
    }
    if rc.is_relate_to_effect(duel, re)
        && duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)? != 0
        && !rc.is_location(duel, LOCATION_HAND | LOCATION_DECK)
        && aux::nvfilter(rc, duel)
    {
        let zone_free = if rc.is_location(duel, LOCATION_EXTRA) {
            duel.get_location_count_from_ex(tp, tp, None, None::<CardId>) > 0
        } else {
            duel.get_location_count(tp, LOCATION_MZONE) > 0
        };
        if rc.is_monster(duel)
            && zone_free
            && rc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, 0xff)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.break_effect()?;
            duel.special_summon(rc, 0, tp, tp, false, false, POS_FACEDOWN_DEFENSE)?;
            duel.confirm_cards(1 - tp, rc)?;
        } else if (rc.is_type(duel, TYPE_FIELD) || duel.get_location_count(tp, LOCATION_SZONE) > 0)
            && rc.is_ssetable(duel)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.break_effect()?;
            duel.sset(tp, rc)?;
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
