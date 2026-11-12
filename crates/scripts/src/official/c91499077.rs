// Gagaga Samurai
use super::c88942504::operation as atop;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 91499077;

// In the Extra Deck, Xyz Summoned with two Level 4 monsters (Hyosube, Water Omotics), and opponent's attackers (Elfnote
// Lucina) to redirect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2118022, MZONE),
    (2118022, MZONE),
    (2483611, MZONE),
    (2483611, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, _| duel.is_able_to_enter_bp());
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, attg);
    e1.set_operation(duel, atop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, cbcon);
    e2.set_operation(duel, cbop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_GAGAGA]) && c.get_effect_count(duel, EFFECT_EXTRA_ATTACK) == 0
}

fn attg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn cbcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let bt = a.eg.and_then(|eg| eg.get_first());
    unported!(duel, bt.is_none());
    bt.is_some_and(|bt| a.r != REASON_REPLACE && bt != c && bt.is_controler(duel, a.tp) && c.is_position(duel, POS_ATTACK))
}

fn cbop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && duel.change_position(c, POS_FACEUP_DEFENSE)? != 0 {
        let at = duel.get_attacker();
        unported!(duel, at.is_none());
        if let Some(at) = at
            && at.can_attack(duel)
            && !at.is_immune_to_effect(duel, e)
            && !c.is_immune_to_effect(duel, e)
        {
            duel.calculate_damage(at, Some(c))?;
        }
    }
    Ok(())
}
