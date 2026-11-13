// Flying Mary, the Wandering Ghost Ship
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT, REMOVED};

const ID: u32 = 95784714;

// In the Extra Deck, Link Summoned from Skull Servants (Zombies), with Shadow Ghoul (Level 5 Zombie) in the Graveyard, banished and on the
// field, and Sangan on the opponent's field to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (32274490, MZONE),
    (32274490, MZONE),
    (32274490, MZONE),
    (30778711, GRAVE),
    (30778711, GRAVE),
    (30778711, REMOVED),
    (30778711, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_ZOMBIE, Some(lc), sumtype, tp)
    }
}

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
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_continuous_spell(duel));
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_above(duel, 5)
        && c.is_race(duel, RACE_ZOMBIE)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && c.is_faceup(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.get_mzone_count(tp, Some(e.get_handler(duel).into())) > 0
            && duel.is_existing_target(filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || !duel.move_to_field(c, tp, tp, LOCATION_SZONE, POS_FACEUP, true)? {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_CHANGE_TYPE);
    e1.set_value(duel, (TYPE_SPELL | TYPE_CONTINUOUS) as i32);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
    c.register_effect(duel, e1);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn destgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_level_above(duel, 5)
        && c.is_race(duel, RACE_ZOMBIE)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(
            aux::faceup_filter(|oc, duel| oc.is_attack_below(duel, c.get_attack(duel))),
            tp,
            0,
            LOCATION_MZONE,
            1,
            None,
        )
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && destgfilter(chkc, duel, tp));
    }
    let filter = |c, duel: &Duel| destgfilter(c, duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let atk = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_attack(duel));
    let dg = duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_attack_below(duel, atk)),
        tp,
        0,
        LOCATION_MZONE,
        None,
    );
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || tc.is_facedown(duel) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let atk = tc.get_attack(duel);
    let g = duel.select_matching_card(
        tp,
        aux::faceup_filter(|c, duel| c.is_attack_below(duel, atk)),
        tp,
        0,
        LOCATION_MZONE,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
