// Prototype Sky Striker Ace - Amatsu
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 25072579;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_SKY_STRIKER_ACE], Some(lc), sumtype, tp)
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];
// In the Extra Deck, Link Summoned with Sky Striker Ace - Raye, with an opponent's Sky Striker Ace - Zero (2000 ATK, a Link
// Monster to destroy) on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (26077387, MZONE), (76072561, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, chngcon);
    e1.set_target(duel, chngtg);
    e1.set_operation(duel, chngop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn chngcon(duel: &Duel, a: &Args) -> bool {
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    let atk: i64 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_ATTACK);
    a.rp == 1 - a.tp && atk >= 2000 && loc == LOCATION_MZONE
}

fn chngfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SKY_STRIKER_ACE]) && c.is_link_monster(duel)
}

fn chngtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(chngfilter, a.rp, 0, LOCATION_MZONE, 1, None))
}

fn chngop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.change_target_card(a.ev as u8, CardSet::default());
    duel.change_chain_operation(a.ev as u8, repop);
    Ok(())
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, chngfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let (bc1, bc2) = duel.get_battle_monster(a.tp);
    bc1 == Some(a.e.get_handler(duel)) && bc2.is_some()
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_effect_target(duel, e)
        && (c.is_controler(duel, 1 - tp) || (c.is_set_card(duel, &[SET_SKY_STRIKER_ACE]) && c.is_faceup(duel)))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| desfilter(c, duel, e, tp), tp, LOCATION_MZONE, LOCATION_ONFIELD, None);
    let check = aux::dpcheck(|c, duel| i64::from(c.get_controler(duel)));
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &check));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &check, tp, HINTMSG_DESTROY)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(tg), 2, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tg) = duel.get_target_cards(a.e)
        && !tg.is_empty()
    {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}
