// Clear Wing Synchro Dragon of the Four Heavenly Dragons
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 54603525;

// Synchro Summoned from Ash Blossom & Joyous Spring and Mystical Elf, with Turbo Synchron (WIND Tuner) in the hand to summon on
// the opponent's Raigeki and Doron for Mist Bird Clausolas, and the opponent's Sangan to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (14558127, MZONE),
    (15025844, MZONE),
    (756652, MZONE),
    (67270095, HAND),
    (67270095, HAND),
    (15240238, EXTRA),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.ep == 1 - a.tp);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn effect_monster(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_effect_monster(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(effect_monster, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(effect_monster, tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, effect_monster, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    let Some(tc) = g.first() else { return Ok(()) };
    duel.hint_selection(g);
    let c = a.e.get_handler(duel);
    let atk = tc.get_base_attack(duel);
    if duel.destroy(g, REASON_EFFECT)? > 0 && c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) && atk > 0 {
        c.update_attack(duel, atk, None, None, None);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4)
        && c.is_attribute(duel, ATTRIBUTE_WIND)
        && c.is_type(duel, TYPE_TUNER)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn synchrofilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WIND) && c.is_synchro_summonable(duel, None, None)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
        tp,
        LOCATION_HAND | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.first()
        && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        sc.negate_effects(duel, e.get_handler(duel), 0, false);
    }
    if duel.special_summon_complete()? > 0
        && duel.is_existing_matching_card(synchrofilter, tp, LOCATION_EXTRA, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, synchrofilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if let Some(synchro) = g.first() {
            duel.synchro_summon(tp, synchro, None, None)?;
        }
    }
    Ok(())
}
