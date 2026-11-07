// Number 75: Bamboozling Gossip Shadow
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 71166481;

// In the Extra Deck, Xyz Summoned with Petit Angel and Mystical Sheep #2, other Numbers to attach to, and the opponent's
// monsters with effects.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (38142739, MZONE),
    (83464209, MZONE),
    (38142739, MZONE),
    (84013237, MZONE),
    (2061963, MZONE),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, chcon);
    e1.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e1.set_target(duel, chtg);
    e1.set_operation(duel, chop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, xtg);
    e2.set_operation(duel, xop);
    c.register_effect(duel, e2);
}

fn chcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_monster_effect(duel)) && a.rp == 1 - a.tp
}

fn chtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_player_can_draw(a.tp, 1) && duel.is_player_can_draw(1 - a.tp, 1))
}

fn chop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.change_target_card(a.ev as u8, CardSet::default());
    duel.change_chain_operation(a.ev as u8, repop);
    Ok(())
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.draw(a.tp, 1, REASON_EFFECT)?;
    duel.draw(1 - a.tp, 1, REASON_EFFECT)?;
    Ok(())
}

fn xfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && c.is_set_card(duel, &[SET_NUMBER])
}

fn xtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && xfilter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(xfilter, tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, xfilter, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    Ok(false)
}

fn xop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && !tc.is_immune_to_effect(duel, e)
    {
        duel.overlay(tc, c)?;
    }
    Ok(())
}
