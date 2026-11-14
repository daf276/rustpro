// Archfiend Emperor
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT, PZONE, SZONE};

const ID: u32 = 48469380;
const THRONE: u32 = 63679166;

pub const LISTED_NAMES: &[u32] = &[THRONE];

// A copy in a Pendulum Zone with face-up Spells to destroy, which sends it to the Extra Deck face-up, and Throne of the Archfiends
// with Level 4 Archfiend Soldiers in hand to Ritual Summon it from there to the Extra Monster Zone.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, HAND),
    (THRONE, HAND),
    (49881766, HAND),
    (49881766, HAND),
    (15259703, SZONE),
    (15259703, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::SpecialSummoned,
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Pendulum::add_procedure(duel, c);
    c.add_must_be_ritual_summoned(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_range(duel, LOCATION_EMZONE);
    e2.set_value(
        duel,
        Value::Effect(|duel, e, te| te.is_monster_effect(duel) && te.get_owner(duel) != e.get_owner(duel)),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_ritual_summoned(duel) && c.is_summon_location(duel, LOCATION_EXTRA)
    });
    e3.set_target(duel, rmtg);
    e3.set_operation(duel, rmop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_EMZONE);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn faceup_spell_trap(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_spell_trap(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_spell_trap(duel) && chkc.is_faceup(duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(faceup_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let mut g = duel
        .select_target(tp, faceup_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?
        .unwrap_or_default();
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 2, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
    {
        duel.destroy(CardSet::from_iter([c, tc]), REASON_EFFECT)?;
    }
    Ok(())
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, a.tp, LOCATION_MMZONE, LOCATION_MMZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, PLAYER_ALL, LOCATION_MMZONE as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.tp, LOCATION_MMZONE, LOCATION_MMZONE);
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_FIEND) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let location = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED;
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), location)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let location = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED;
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), location, POS_FACEUP)
}
