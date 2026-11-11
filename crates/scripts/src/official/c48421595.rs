// Necro Synchron
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 48421595;
const FLEUR_SYNCHRON: u32 = 19642774;

pub const LISTED_NAMES: &[u32] = &[FLEUR_SYNCHRON];

// On the field with Kojikocy (Level 4 Normal Monster) for Stardust Charge Warrior (WIND) in the Extra Deck, and Level 1
// Plant Normal Monsters in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (64880894, EXTRA),
    (64880894, EXTRA),
    (64880894, EXTRA),
    (13193642, DECK),
    (13193642, DECK),
    (27520594, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e1.set_value(duel, FLEUR_SYNCHRON as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, lvtg);
    e2.set_operation(duel, lvop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn lvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.has_level(duel)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && lvfilter(chkc, duel) && c != chkc);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(lvfilter, tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, lvfilter, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, a.e)
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, 2);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_PLANT)
        && c.is_level(duel, &[1])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE)
        && (a.r & REASON_SYNCHRO) == REASON_SYNCHRO
        && c.get_reason_card(duel).is_some_and(|sync| sync.is_attribute(duel, ATTRIBUTE_WIND))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let tc = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !tc.is_empty() {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
