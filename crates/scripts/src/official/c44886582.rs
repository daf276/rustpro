// Super Duper Polymerization
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 44886582;

struct Params;

impl FusionSummonParams for Params {
    const EXACTCOUNT: Option<usize> = Some(2);

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_on_field(duel) && c.is_able_to_grave(duel)
    }

    // The forced materials, the target (still the chain's first target) and the Special Summoned monster kept in the label object.
    fn gc(duel: &Duel, e: EffectId, _: u8) -> Option<CardSet> {
        let mut gc = CardSet::from(duel.get_first_target()?);
        gc.merge(e.get_label_object_group(duel)?);
        Some(gc)
    }

    fn extrafil(duel: &Duel, _: EffectId, _: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        (duel.get_first_target().map(CardSet::from), None)
    }
}

// Destiny HERO - Malicious as the target, and Destiny HERO - Dystopia and Dusktopia in the Extra Deck to summon, which then
// Fusion Summon each other with it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (9411399, MZONE), (90579153, EXTRA), (93657021, EXTRA)];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(93657021)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, Cost::pay_lp::<2000>);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_type(duel, TYPE_EFFECT)
        && c.has_level(duel)
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, tc: CardId) -> bool {
    c.is_race(duel, tc.get_race(duel))
        && c.is_attribute(duel, tc.get_attribute(duel))
        && c.has_level(duel)
        && !c.is_level(duel, &[tc.get_level(duel)])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel, e, tp));
    }
    let filter = |c, duel: &Duel| filter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_MZONE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        duel.set_chain_limit(|_, _, _, _, _| false, 0);
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if !tc.is_relate_to_effect(duel, e) || tc.is_facedown(duel) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sc = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, tc), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(sc) = sc.get_first() else {
        return Ok(());
    };
    if duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        sc.negate_effects(duel, e.get_handler(duel), 0, false);
        duel.break_effect()?;
        e.set_label_object_group(duel, Some(sc.into()));
        Fusion::new_summon_eff_closure(duel, e, false);
        let b1 = Fusion::summon_eff_tg::<Params>(duel, a, 0, None)?;
        let b2 = sc.is_able_to_grave(duel);
        match duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])? {
            Some(1) => {
                Fusion::new_summon_eff_closure(duel, e, true);
                Fusion::summon_eff_op::<Params>(duel, a)?;
            }
            Some(2) => {
                duel.send_to_grave(sc, REASON_EFFECT)?;
            }
            _ => {}
        }
    }
    Ok(())
}
