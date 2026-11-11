// M-X-Saber Invoker
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 4423206;

// In the Extra Deck, Xyz Summoned with two Level 3 monsters (Root Water, Twin Long Rods #2), with EARTH Level 4 Warriors (Kojikocy,
// Kanan the Swordmistress) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (39004808, MZONE),
    (29692206, MZONE),
    (1184620, DECK),
    (12829151, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_WARRIOR | RACE_BEASTWARRIOR)
        && c.is_attribute(duel, ATTRIBUTE_EARTH)
        && c.get_level(duel) == 4
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.first()
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? != 0
    {
        let handler = e.get_handler(duel);
        let fid = i64::from(handler.get_field_id(duel));
        tc.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1, fid, 0);
        let e1 = duel.create_effect(handler);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_count_limit(duel, 1, 0);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_label(duel, fid);
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_condition_mut(duel, descon);
        e1.set_operation(duel, desop);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.first());
    if tc.and_then(|tc| tc.get_flag_effect_label(duel, ID)) != Some(a.e.get_label(duel)) {
        a.e.reset(duel);
        return false;
    }
    true
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
