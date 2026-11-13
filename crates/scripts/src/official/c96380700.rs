// Talkback Lancer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 96380700;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        matfilter(c, duel, lc, sumtype, tp)
    }
}

// On the field with Protrons (Level 1 Cyberse) to release and Code Talkers in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (24154052, MZONE),
    (24154052, MZONE),
    (53413628, GRAVE),
    (53413628, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn matfilter(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_level_below(duel, 2) && c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_race(duel, RACE_CYBERSE)
        && c.is_monster(duel)
        && duel.get_mzone_count_full(tp, Some(c.into()), tp, LOCATION_REASON_TOFIELD, zone) > 0
        && duel.is_existing_target(|tc, duel| spfilter(tc, duel, e, tp, c), tp, LOCATION_GRAVE, 0, 1, None)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let zone = c.get_linked_zone_full(duel, tp);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |rc, duel| cfilter(rc, duel, e, tp, zone), (1, 1), false, None, Some(c.into())));
    }
    let g = duel.select_release_group_cost(tp, |rc, duel| cfilter(rc, duel, e, tp, zone), (1, 1), false, None, Some(c.into()))?;
    duel.release(g, REASON_COST)?;
    e.set_label_object_group(duel, g.get_first().map(Into::into));
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, rc: CardId) -> bool {
    c.is_set_card(duel, &[SET_CODE_TALKER])
        && c.is_monster(duel)
        && !c.is_original_code_rule(duel, rc.get_original_code_rule(duel))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let cc = e.get_label_object_group(duel).and_then(|g| g.get_first());
    if let Some(chkc) = chkc {
        let Some(cc) = cc else { return Ok(false) };
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp, cc));
    }
    if chk == 0 {
        return Ok(true);
    }
    let Some(cc) = cc else { return Ok(false) };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp, cc), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let zone = e.get_handler(duel).get_linked_zone_full(duel, tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) {
        duel.special_summon_full(tc, 0, tp, tp, false, false, POS_FACEUP, zone)?;
    }
    Ok(())
}
