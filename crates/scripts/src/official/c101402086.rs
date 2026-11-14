// Archidux, the Squid Sorcerer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 101402086;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(lc), sumtype, tp)
    }
}

// On the field and in the Graveyard facing the opponent's Raigeki, with Level 2 WATER Normal Monsters (Psychic Kappa) to summon,
// and Link Summoned from them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, GRAVE),
    (ID, EXTRA),
    (7892180, HAND),
    (7892180, MZONE),
    (7892180, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, atkdefcost);
    e1.set_operation(duel, atkdefop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_CUSTOM + ID);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    e2.set_label_object_group(duel, Some(CardSet::default()));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_operation(duel, regop);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let eg = a.eg.unwrap_or_default();
    if duel.is_damage_step() || eg.is_contains(c) {
        return Ok(());
    }
    let tg = eg.filter(duel, |tc, duel| tgfilter(tc, duel, e, tp, 1 - tp, true), None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let Some(e2) = e.get_label_object(duel) else { return Ok(()) };
    let mut g = e2.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g.merge(tg);
    g.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    e2.set_label_object_group(duel, Some(g));
    duel.raise_single_event(c, EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)
}

fn atkdefcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_grave_as_cost, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_grave_as_cost, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn atkdefop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1a.set_target_range(duel, LOCATION_MZONE, 0);
    e1a.set_target_card(duel, |duel, _, c| c.is_attribute(duel, ATTRIBUTE_WATER));
    e1a.set_value(duel, 500);
    e1a.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1a, tp);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    duel.register_effect(e1b, tp);
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, opp: u8, ign_chk: bool) -> bool {
    c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_controler(duel, tp)
        && c.is_previous_attribute_on_field(duel, ATTRIBUTE_WATER)
        && c.is_link_monster(duel)
        && c.is_faceup(duel)
        && c.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED)
        && c.is_can_be_effect_target(duel, e)
        && c.is_reason(duel, REASON_EFFECT)
        && c.is_reason_player(duel, opp)
        && (ign_chk || {
            let link = c.get_link(duel);
            duel.is_existing_matching_card(
                |sc, duel| spfilter(sc, duel, e, tp, link),
                tp,
                LOCATION_HAND | LOCATION_GRAVE,
                0,
                1,
                None,
            )
        })
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, link: u32) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_level_below(duel, link) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let opp = 1 - tp;
    let g = e
        .get_label_object_group(duel)
        .unwrap_or_default()
        .filter(duel, |c, duel| tgfilter(c, duel, e, tp, opp, false), None);
    if let Some(chkc) = chkc {
        return Ok(g.is_contains(chkc) && tgfilter(chkc, duel, e, tp, opp, false));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && !g.is_empty());
    }
    let tc = if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
        g.select(duel, tp, 1, 1, None)?.get_first()
    } else {
        g.get_first()
    };
    if let Some(tc) = tc {
        duel.set_target_card(tc);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let link = tc.get_link(duel);
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, link));
        let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
