// Sunavalon Melias
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 44478599;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_PLANT, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, g: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        g.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp), 1, None)
    }
}

pub const LISTED_NAMES: &[u32] = &[27520594];
// On the field and in the Extra Deck, Link Summoned with Sunavalon Dryas and Sunseed Genius Loci, with Genius Loci in the
// Graveyard to summon and Sunvine Link Monsters on the field to point at.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (93896655, MZONE),
    (27520594, MZONE),
    (27520594, GRAVE),
    (65563871, MZONE),
    (65563871, MZONE),
    (91557476, MZONE),
    (28168762, MZONE),
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
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_IGNORE_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, _| duel.is_able_to_enter_bp());
    e3.set_target(duel, attg);
    e3.set_operation(duel, atop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, 27520594) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, e, tp), LOCATION_GRAVE)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn atfilter(c: CardId, duel: &Duel, lg: CardSet) -> bool {
    c.is_set_card(duel, &[SET_SUNVINE]) && c.is_link_monster(duel) && lg.is_contains(c)
}

fn valfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SUNAVALON]) && c.is_link_monster(duel)
}

fn attg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && atfilter(chkc, duel, lg));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| atfilter(c, duel, lg), tp, LOCATION_MZONE, 0, 1, None)
                && duel.get_matching_group_count(valfilter, tp, LOCATION_MZONE, 0, None) > 0,
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, |c, duel| atfilter(c, duel, lg), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let ct = duel.get_matching_group_count(valfilter, tp, LOCATION_MZONE, 0, None);
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    if ct > 0 {
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_value(duel, ct as i32 - 1);
    } else {
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    }
    tc.register_effect(duel, e1);
    tc.register_flag_effect_full(
        duel,
        ID,
        RESETS_STANDARD_PHASE_END,
        EFFECT_FLAG_CLIENT_HINT,
        1,
        0,
        aux::stringid(ID, 2),
    );
    Ok(())
}
