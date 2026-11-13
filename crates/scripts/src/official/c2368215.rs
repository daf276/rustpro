// Abyss Actor - Hyper Director
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, PZONE};

const ID: u32 = 2368215;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_ABYSS_ACTOR], Some(lc), sumtype, tp) && c.is_type_full(duel, TYPE_PENDULUM, Some(lc), sumtype, tp)
    }
}

// On the field and in the Extra Deck, Link Summoned from Abyss Actor Pendulum Monsters (Extras), with one in the Pendulum Zone and
// Trendy Understudy in the Deck to place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (88412339, MZONE),
    (88412339, MZONE),
    (88412339, PZONE),
    (39024589, DECK),
    (39024589, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

// The Lua passes the code in the exception slot of IsExistingMatchingCard, where a number reads as nil, so pfilter gets no code.
fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.is_existing_matching_card(|pc, duel| pfilter(pc, duel, None), tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, None)
}

fn pfilter(c: CardId, duel: &Duel, code: Option<u32>) -> bool {
    c.is_set_card(duel, &[SET_ABYSS_ACTOR])
        && c.is_type(duel, TYPE_PENDULUM)
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_EXTRA))
        && !c.is_forbidden(duel)
        && !code.is_some_and(|code| c.is_code(duel, code))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_PZONE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_PZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_PZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? != 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
        let code = tc.get_code(duel);
        let g = duel.select_matching_card(
            tp,
            |c, duel| pfilter(c, duel, Some(code)),
            tp,
            LOCATION_DECK | LOCATION_EXTRA,
            0,
            1,
            1,
            None,
        )?;
        if let Some(pc) = g.get_first() {
            duel.break_effect()?;
            duel.move_to_field(pc, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
        }
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_set_card(duel, &[SET_ABYSS_ACTOR]));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_SUMMON);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    e3.set_target_range(duel, 1, 0);
    duel.register_effect(e3, tp);
    Ok(())
}
