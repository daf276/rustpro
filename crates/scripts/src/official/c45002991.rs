// Crusadia Equimax
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 45002991;

pub const LISTED_SERIES: &[u16] = &[SET_WORLD_LEGACY, SET_CRUSADIA];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        lcheck(duel, sg, lc, sumtype, tp)
    }
}

// Copies in the Extra Deck, Link Summoned from Crusadia Arboria and Mekk-Knight Crusadia Avramax (a Link Monster), with Ash Blossom to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (91646304, MZONE),
    (91646304, MZONE),
    (21887175, MZONE),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, atklimit);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_count_limit(duel, 1, 0);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_cost(duel, discost);
    e3.set_target(duel, distg);
    e3.set_operation(duel, disop);
    c.register_effect(duel, e3);
}

pub fn lcheck(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
    sg.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp), 1, None)
}

// Shared with Crusadia Regulex and Crusadia Spatha.
pub fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let g = e.get_handler(duel).get_linked_group(duel).filter(duel, CardId::is_faceup, None);
    g.get_sum(duel, |c, duel| c.get_base_attack(duel).into()) as i32
}

// Shared with Crusadia Regulex and Crusadia Spatha.
pub fn atklimit(duel: &Duel, e: EffectId, c: CardId) -> bool {
    e.get_handler(duel).get_linked_group(duel).is_contains(c)
}

fn disfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_negatable(duel) && c.is_can_be_effect_target(duel, e)
}

fn cfilter(c: CardId, duel: &Duel, lg: CardSet) -> bool {
    c.is_set_card(duel, &[SET_WORLD_LEGACY, SET_CRUSADIA]) && lg.is_contains(c)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_matching_group(|c, duel| disfilter(c, duel, e), tp, 0, LOCATION_ONFIELD, None);
    let lg = e.get_handler(duel).get_linked_group(duel);
    let check = aux::release_check_target(tg);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |c, duel| cfilter(c, duel, lg), (1, 1), false, Some(&check), None));
    }
    let g = duel.select_release_group_cost(tp, |c, duel| cfilter(c, duel, lg), (1, 1), false, Some(&check), None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel) && chkc.is_negatable(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !((tc.is_faceup(duel) && !tc.is_disabled(duel)) || tc.is_type(duel, TYPE_TRAPMONSTER)) || !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.negate_related_chain(tc, RESET_TURN_SET);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EFFECT_DISABLE_EFFECT);
    e2.set_value(duel, RESET_TURN_SET as i32);
    e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e2);
    if tc.is_type(duel, TYPE_TRAPMONSTER) {
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
        e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e3);
    }
    Ok(())
}
