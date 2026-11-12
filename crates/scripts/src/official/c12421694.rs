// Sky Striker Ace - Kaina
use engine::constants::MSG_RECOVER;

use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 12421694;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_SKY_STRIKER_ACE], Some(lc), sumtype, tp) && c.is_attribute_except(duel, ATTRIBUTE_EARTH)
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];
// In the Extra Deck, Link Summoned with Sky Striker Ace - Raye (DARK), with an opponent's monster to target and Sky Striker Mobilize -
// Engage! in the hand (it needs an empty Main Monster Zone and a Sky Striker in the Deck) to be activated.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26077387, MZONE),
    (26077387, DECK),
    (26077387, DECK),
    (89631139, MZONE | OPPONENT),
    (63166095, HAND),
    (63166095, HAND),
    (63166095, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_RECOVER)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.set_sp_summon_once(duel, ID);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, natg);
    e1.set_operation(duel, naop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_operation(duel, aux::chainreg);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_CHAIN_SOLVED);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, 1));
    e3.set_operation(duel, lpop);
    c.register_effect(duel, e3);
}

fn natg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn naop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) {
        let ct = if duel.is_turn_player(tp) { 2 } else { 1 };
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3206);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END, ct);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn lpop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if re.is_spell_effect(duel) && a.rp == a.tp && re.get_handler(duel).is_set_card(duel, &[SET_SKY_STRIKER]) {
        duel.hint(HINT_CARD, 0, ID.into());
        duel.recover(a.tp, 100, REASON_EFFECT)?;
    }
    Ok(())
}
