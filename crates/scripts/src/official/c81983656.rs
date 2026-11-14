// Blackwing Tamer - Obsidian Hawk Joe
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 81983656;

pub const LISTED_SERIES: &[u16] = &[SET_BLACKWING];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_BLACKWING], Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_set_card_full(duel, &[SET_BLACKWING], Some(sc), sumtype, tp)
    }
}

// On the field and in the Extra Deck (Synchro Summoned from Gale, a Level 3 Tuner, and Zephyros, Level 4) with Blackwing monsters
// in the Graveyard, against Elfnote Lucina attacking and Mage Power targeting from the opponent's hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (2009101, MZONE),
    (2009101, MZONE),
    (14785765, MZONE),
    (14785765, MZONE),
    (11613567, GRAVE),
    (75498415, GRAVE),
    (13597785, MZONE | OPPONENT),
    (83746708, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.r != REASON_REPLACE);
    e2.set_target(duel, cbtg);
    e2.set_operation(duel, cbop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, cecon);
    e3.set_target(duel, cetg);
    e3.set_operation(duel, ceop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_above(duel, 5) && c.is_race(duel, RACE_WINGEDBEAST) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn cbfilter(c: CardId, duel: &Duel, at: CardSet) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_BLACKWING]) && at.is_contains(c)
}

fn cbtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(attacker) = duel.get_attacker() else { return Ok(false) };
    let (at, _) = attacker.get_attackable_target(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && cbfilter(chkc, duel, at));
    }
    let exception = Some(a.e.get_handler(duel).into());
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| cbfilter(c, duel, at), tp, LOCATION_MZONE, 0, 1, exception));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| cbfilter(c, duel, at), tp, LOCATION_MZONE, 0, 1, 1, exception)?;
    Ok(false)
}

fn cbop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && let Some(attacker) = duel.get_attacker()
        && tc.is_relate_to_effect(duel, a.e)
        && !attacker.is_immune_to_effect(duel, a.e)
    {
        duel.change_attack_target(Some(tc), false);
    }
    Ok(())
}

fn cecon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if a.e == re || a.rp == a.tp || !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let g: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    g.is_some_and(|g| g.len() == 1 && g.get_first() == Some(a.e.get_handler(duel)))
}

fn cefilter(c: CardId, duel: &Duel, ct: u8) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_BLACKWING]) && duel.check_mut(|duel| duel.check_chain_target(ct, c))
}

fn cetg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (tp, ev) = (a.tp, a.ev as u8);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && cefilter(chkc, duel, ev));
    }
    let exception = Some(a.e.get_handler(duel).into());
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| cefilter(c, duel, ev), tp, LOCATION_MZONE, 0, 1, exception));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| cefilter(c, duel, ev), tp, LOCATION_MZONE, 0, 1, 1, exception)?;
    Ok(false)
}

fn ceop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.change_target_card(a.ev as u8, tc.into());
    }
    Ok(())
}
