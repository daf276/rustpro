// Crimson Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 63436931;

pub const LISTED_NAMES: &[u32] = &[CARD_CRIMSON_DRAGON];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Ash Blossom & Joyous Spring and Shapesnatch, then on the field to be returned for
// Stardust Dragon with Red Dragon Archfiend (both Level 8 Dragon Synchro Monsters). The search needs a Spell or Trap that mentions
// "The Crimson Dragon", none of which is ported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (14558127, MZONE),
    (4035199, MZONE),
    (ID, MZONE),
    (70902743, MZONE | OPPONENT),
    (44508094, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.lists_code(duel, CARD_CRIMSON_DRAGON) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lvl: u32, mc: Option<CardId>) -> bool {
    c.is_race(duel, RACE_DRAGON)
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_level(duel, &[lvl])
        && duel.get_location_count_from_ex(tp, tp, mc.map(Into::into), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_SYNCHRO, tp, false, false)
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_faceup(duel)
        && c.is_level_above(duel, 7)
        && c.is_type(duel, TYPE_SYNCHRO)
        && !c.is_code(duel, CARD_CRIMSON_DRAGON)
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, c.get_level(duel), Some(mc)),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        )
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let filter = |tc, duel: &Duel| tgfilter(tc, duel, e, tp, c);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc != c && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(c.is_able_to_extra(duel) && duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_TOEXTRA, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e)
        && duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_EXTRA))
    {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let lvl = tc.get_level(duel);
        // The Lua passes no excluded card here, so its mc is nil.
        let g = duel.select_matching_card(
            tp,
            |sc, duel| spfilter(sc, duel, e, tp, lvl, None),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            1,
            None,
        )?;
        let Some(sc) = g.get_first() else { return Ok(()) };
        sc.set_material(duel, None);
        if duel.special_summon(sc, SUMMON_TYPE_SYNCHRO, tp, tp, false, false, POS_FACEUP)? > 0 {
            sc.complete_procedure(duel);
        }
    }
    Ok(())
}
