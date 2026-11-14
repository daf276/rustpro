// Graydle Slime
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 20056760;

pub const LISTED_SERIES: &[u16] = &[SET_GRAYDLE];
// In the hand and Graveyard to Special Summon, with Graydle Slime Jr. on the field to destroy and in the Graveyard to revive.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (80250319, MZONE), (80250319, MZONE), (80250319, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg1);
    e1.set_operation(duel, spop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_condition(duel, spcon2);
    e2.set_target(duel, sptg2);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP_DEFENSE>);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_GRAYDLE])
}

fn sptg1(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        if ft < -1 {
            return Ok(false);
        }
        return Ok(e.get_handler(duel).is_can_be_special_summoned(duel, e, 1, tp, false, false)
            && duel.is_existing_target(filter, tp, LOCATION_ONFIELD, 0, 2, None)
            && (ft > 0 || duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, (1 - ft) as usize, None)));
    }
    let g = if ft != 0 {
        let location = if ft < 0 { LOCATION_MZONE } else { LOCATION_ONFIELD };
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        duel.select_target(tp, filter, tp, location, 0, 2, 2, None)?.unwrap_or_default()
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let mut g = duel
            .select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?
            .unwrap_or_default();
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g2 = duel.select_target(tp, filter, tp, LOCATION_ONFIELD, 0, 1, 1, g.get_first().map(CardSet::from))?;
        g.merge(g2.unwrap_or_default());
        g
    };
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 2, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if duel.destroy(g, REASON_EFFECT)? != 0 {
        if !c.is_relate_to_effect(duel, a.e) {
            return Ok(());
        }
        duel.special_summon(c, 1, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn spcon2(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SPECIAL + 1
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GRAYDLE]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
