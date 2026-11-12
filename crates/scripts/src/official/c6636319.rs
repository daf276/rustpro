// Evil★Twin Ki-sikil Deal
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 6636319;

pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_KI_SIKIL], Some(lc), sumtype, tp)
    }
}

// Link Summoned from Live Twin Ki-sikil, with Live Twin Lil-la in the Deck to send.
// The draw needs the opponent to chain to a Ki-sikil or Lil-la effect, which random play rarely does, so it is not checked.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (36326160, MZONE),
    (36326160, MZONE),
    (73810864, DECK),
    (73810864, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e0.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e0.set_operation(duel, regop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| !duel.has_flag_effect(a.tp, ID));
    e1.set_cost(duel, effcost);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, sumtype, _, _, _| {
        c.is_code(duel, ID) && sumtype & SUMMON_TYPE_LINK == SUMMON_TYPE_LINK
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LIL_LA]) && c.is_monster(duel) && c.is_able_to_grave_as_cost(duel)
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, costfilter, LOCATION_DECK | LOCATION_EXTRA)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.has_flag_effect(tp, ID) {
        return Ok(());
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAIN_SOLVED);
    e1.set_condition(duel, drawcon);
    e1.set_operation(duel, drawop);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
    Ok(())
}

fn drawcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let ch = i32::from(duel.get_current_chain_full(true)) - 1;
    let Ok(ch) = u8::try_from(ch) else { return false };
    ch > 0
        && a.ep == 1 - tp
        && chain::is_triggering_player(duel, ch, tp)
        && chain::is_triggering_type(duel, ch, TYPE_MONSTER)
        && chain::is_triggering_setcode(duel, ch, &[SET_KI_SIKIL, SET_LIL_LA])
}

fn drawop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_CARD, 1 - tp, ID.into());
    duel.draw(tp, 1, REASON_EFFECT)?;
    Ok(())
}
