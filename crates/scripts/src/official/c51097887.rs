// Tri-Brigade Airborne Assault
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 51097887;

// In the hand with Burglar and Niwatori on the field to target, and ones of the other race in the Deck to Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (6297941, MZONE),
    (7805359, MZONE),
    (6297941, DECK),
    (7805359, DECK),
    (7805359, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, atk: i32, race: u64) -> bool {
    c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB)
        && c.is_attack_below(duel, atk)
        && !c.is_race(duel, race)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn tgtfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB)
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, c.get_attack(duel), c.get_race(duel)),
            tp,
            LOCATION_DECK,
            0,
            1,
            None,
        )
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && tgtfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| tgtfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, |c, duel| tgtfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 0, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e0.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e0.set_reset(duel, RESET_PHASE | PHASE_END);
    e0.set_target_range(duel, 1, 0);
    e0.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_type(duel, TYPE_LINK)
    });
    duel.register_effect(e0, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if duel.get_location_count(tp, LOCATION_MZONE) < 1 || !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let (atk, race) = (tc.get_attack(duel), tc.get_race(duel));
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, atk, race), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?
    {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            sc.register_effect_full(duel, e1, true);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}
