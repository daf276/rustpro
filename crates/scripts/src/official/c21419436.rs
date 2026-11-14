// Unchained Soul Rage Abominator
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 21419436;

pub const LISTED_SERIES: &[u16] = &[SET_UNCHAINED];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_race_full(duel, RACE_FIEND, Some(lc), sumtype, tp), 1, None)
    }
}

// In the Extra Deck, Link Summoned from two Fiends on the field, and in the Graveyard with Unchained Abomination (Link-4) on the
// field, with the opponent's Aruha to summon and Unchained Soul of Rage to activate a monster effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (26236560, MZONE),
    (31588572, MZONE),
    (29479265, MZONE),
    (26236560, HAND | OPPONENT),
    (67680512, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_DESTROY + CATEGORY_DISABLE);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1a.set_code(duel, EVENT_CUSTOM + ID);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_count_limit(duel, 1, (ID, 0));
    e1a.set_target(duel, distg);
    e1a.set_operation(duel, disop);
    e1a.set_label_object_group(duel, Some(CardSet::default()));
    c.register_effect(duel, e1a);
    let e1b = duel.create_effect(c);
    e1b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1b.set_range(duel, LOCATION_MZONE);
    e1b.set_label_object(duel, Some(e1a));
    e1b.set_operation(duel, regop);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, descon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn disfilter(c: CardId, duel: &Duel, e: EffectId, opp: u8) -> bool {
    c.is_summon_player(duel, opp)
        && c.is_negatable_monster(duel)
        && c.is_can_be_effect_target(duel, e)
        && c.is_location(duel, LOCATION_MZONE)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_damage_step() {
        return Ok(());
    }
    let tg = a.eg.unwrap_or_default().filter(duel, |c, duel| disfilter(c, duel, e, 1 - tp), None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let Some(e1a) = e.get_label_object(duel) else { return Ok(()) };
    let mut g = e1a.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g.merge(tg);
    g.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    e1a.set_label_object_group(duel, Some(g));
    duel.raise_single_event(e.get_handler(duel), EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let opp = 1 - tp;
    let g = e
        .get_label_object_group(duel)
        .unwrap_or_default()
        .filter(duel, |c, duel| disfilter(c, duel, e, opp), None);
    if let Some(chkc) = chkc {
        return Ok(g.is_contains(chkc) && disfilter(chkc, duel, e, opp));
    }
    let dg = duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_race(duel, RACE_FIEND)),
        tp,
        LOCATION_MZONE,
        0,
        None,
    );
    if chk == 0 {
        return Ok(!dg.is_empty() && !g.is_empty());
    }
    let tc = if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
        g.select(duel, tp, 1, 1, None)?
    } else {
        g
    };
    duel.set_target_card(tc);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(tc), 1, tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(
        tp,
        aux::faceup_filter(|c, duel| c.is_race(duel, RACE_FIEND)),
        tp,
        LOCATION_MZONE,
        0,
        1,
        1,
        None,
    )?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    if duel.destroy(g, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
    {
        tc.negate_effects(duel, e.get_handler(duel), RESET_PHASE | PHASE_END, false);
    }
    Ok(())
}

fn desconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_link_above(duel, 4) && c.is_set_card(duel, &[SET_UNCHAINED]) && c.is_faceup(duel)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    re.is_monster_effect(duel)
        && loc == LOCATION_MZONE
        && re.get_handler(duel).is_relate_to_effect(duel, re)
        && duel.is_existing_matching_card(desconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    rc.create_effect_relation(duel, a.e);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(rc.into()), 1, a.tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if rc.is_relate_to_effect(duel, re) {
        duel.destroy(rc, REASON_EFFECT)?;
    }
    Ok(())
}
