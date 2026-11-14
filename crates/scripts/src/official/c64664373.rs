// Metaltronus
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 64664373;

// In hand, with Lacrima the Crimson Tears on the opponent's field to target and in the Deck to summon (same name, to banish
// both), and Fiendsmith Engraver in the Deck as another LIGHT Fiend.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (28803166, MZONE | OPPONENT),
    (28803166, MZONE | OPPONENT),
    (28803166, DECK),
    (28803166, DECK),
    (60764609, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DISABLE + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && !c.is_type(duel, TYPE_TOKEN)
        && c.is_negatable_monster(duel)
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, c),
            tp,
            LOCATION_HAND | LOCATION_DECK | LOCATION_EXTRA,
            0,
            1,
            None,
        )
}

// `tc` stands for the race, attribute and ATK the Lua passes.
fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, tc: CardId) -> bool {
    let ct = usize::from(c.is_race(duel, tc.get_race(duel)))
        + usize::from(c.is_attribute(duel, tc.get_attribute(duel)))
        + usize::from(c.is_attack(duel, &[tc.get_attack(duel)]));
    ct >= 2
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && ((c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, None, c) > 0)
            || (!c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count(tp, LOCATION_MZONE) > 0))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && cfilter(chkc, duel, e, tp));
    }
    let filter = |c, duel: &Duel| cfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let tc = duel
        .select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?
        .and_then(|g| g.get_first());
    let locations = (LOCATION_HAND | LOCATION_DECK | LOCATION_EXTRA) as i32;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, locations);
    duel.set_operation_info(0, CATEGORY_DISABLE, tc.map(Into::into), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 2, tp, 0);
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        let capture = tc.map_or(0, |tc| tc.index() as i64 + 1);
        duel.set_chain_limit(|duel, e, _, _, tc| e.get_handler(duel).index() as i64 + 1 != tc, capture);
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel
        .get_first_target()
        .filter(|tc| tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel))
    else {
        return Ok(());
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp, tc);
    let sc = duel
        .select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first();
    if let Some(sc) = sc
        && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        let c = e.get_handler(duel);
        sc.negate_effects(duel, c, 0, false);
        if duel.special_summon_complete()? == 0 {
            return Ok(());
        }
        tc.negate_effects(duel, c, 0, false);
        duel.adjust_instantly(Some(tc));
        if sc.is_code(duel, tc.get_code(duel))
            && sc.is_able_to_remove_full(duel, Some(tp), POS_FACEDOWN, REASON_EFFECT)
            && tc.is_able_to_remove_full(duel, Some(tp), POS_FACEDOWN, REASON_EFFECT)
            && duel.select_yes_no(tp, aux::stringid(ID, 1))?
        {
            duel.break_effect()?;
            duel.remove(CardSet::from_iter([tc, sc]), POS_FACEDOWN, REASON_EFFECT)?;
        }
    }
    Ok(())
}
