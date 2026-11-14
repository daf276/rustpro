// Machina Citadel
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 87074380;

pub const LISTED_NAMES: &[u32] = &[ID];
// On the field with Machina Gearframe to destroy against the opponent's Sangan, and in the GY for when it or Raigeki
// destroys Gearframe.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (42940404, MZONE),
    (42940404, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_unsummonable(duel);
    c.add_must_be_special_summoned_by_card_effect(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        let eg = a.eg.unwrap_or_default();
        !eg.contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| spconfilter(c, duel, a.tp), 1, None)
    });
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn weaker(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_faceup(duel) && c.is_attack_below(duel, atk)
}

fn desfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let atk = c.get_attack(duel);
    c.is_race(duel, RACE_MACHINE)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(|c, duel| weaker(c, duel, atk), tp, 0, LOCATION_MZONE, 1, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && desfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| desfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |c, duel| desfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let Some(tc) = g.and_then(|g| g.get_first()) else {
        return Ok(false);
    };
    let atk = tc.get_attack(duel);
    let mut dg = duel.get_matching_group(|c, duel| weaker(c, duel, atk), tp, 0, LOCATION_MZONE, None);
    dg.insert(tc);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), dg.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_race(duel, RACE_MACHINE) && tc.is_faceup(duel) {
        let atk = tc.get_attack(duel);
        let mut g = duel.get_matching_group(|c, duel| weaker(c, duel, atk), a.tp, 0, LOCATION_MZONE, None);
        if !g.is_empty() {
            g.insert(tc);
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_attribute_on_field(duel, ATTRIBUTE_EARTH)
        && c.is_previous_race_on_field(duel, RACE_MACHINE)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && !c.is_previous_code_on_field(duel, &[ID])
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
}
