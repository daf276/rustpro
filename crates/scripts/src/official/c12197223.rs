// Shiina, Twin Tempests of Celestial Thunder
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 12197223;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand with a WIND monster, Flying Kamakiri #2, on the field, and Raigeki in the opponent's hand to chain to.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (3134241, MZONE), (12580477, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE > 0));
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_attribute(duel, ATTRIBUTE_WIND)),
            a.tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn rthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn spell_trap_to_hand(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(re) = a.re else { return Ok(false) };
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && ((re.is_monster_effect(duel) && duel.is_existing_matching_card(rthfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None))
                || (re.is_spell_trap_effect(duel)
                    && duel.is_existing_matching_card(spell_trap_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None))));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, PLAYER_ALL, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(re) = a.re else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let mut g = CardSet::default();
        if re.is_monster_effect(duel) {
            g = duel.get_matching_group(rthfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
        } else if re.is_spell_trap_effect(duel) {
            g = duel.get_matching_group(spell_trap_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
        }
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        }
    }
    Ok(())
}
