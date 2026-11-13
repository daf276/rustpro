// Ra'ten, the Heavenly General
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 30163008;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(99);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        lcheck(duel, sg, lc, sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned from Warriors, with Level 4 or lower Warriors in hand to Special Summon and the opponent's monsters to
// destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (91152256, MZONE),
    (48305365, MZONE),
    (11321183, MZONE),
    (91152257, HAND),
    (48305365, HAND),
    (26202165, MZONE | OPPONENT),
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
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_PHASE + PHASE_BATTLE_START);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
}

fn lcheck(duel: &Duel, g: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
    aux::check_same_property(duel, g, &|c, duel| c.get_race_full(duel, Some(lc), sumtype, tp)).0
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8, lg: CardSet, zone: u32) -> bool {
    c.is_faceup(duel)
        && lg.is_contains(c)
        && duel.is_existing_matching_card(
            |sc, duel| spfilter2(sc, duel, e, tp, c.get_race(duel), zone),
            tp,
            LOCATION_HAND,
            0,
            1,
            None,
        )
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8, rc: u64, zone: u32) -> bool {
    c.is_level_below(duel, 4)
        && c.is_race(duel, rc)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let lg = e.get_handler(duel).get_linked_group(duel);
    let zone = e.get_handler(duel).get_linked_zone_full(duel, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && spfilter1(chkc, duel, e, tp, lg, zone));
    }
    if chk == 0 {
        return Ok(
            duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zone) > 0
                && duel.is_existing_target(
                    |c, duel| spfilter1(c, duel, e, tp, lg, zone),
                    tp,
                    LOCATION_MZONE,
                    LOCATION_MZONE,
                    1,
                    None,
                ),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(
        tp,
        |c, duel| spfilter1(c, duel, e, tp, lg, zone),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tg) = duel.get_first_target() else { return Ok(()) };
    if !c.is_relate_to_effect(duel, e) || !tg.is_relate_to_effect(duel, e) || tg.is_facedown(duel) {
        return Ok(());
    }
    let zone = c.get_linked_zone_full(duel, tp);
    if duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zone) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let rc = tg.get_race(duel);
    let g = duel.select_matching_card(tp, |c, duel| spfilter2(c, duel, e, tp, rc, zone), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.special_summon_full(tc, 0, tp, tp, false, false, POS_FACEUP, zone)?;
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}
