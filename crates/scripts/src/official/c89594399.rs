// Reptilianne Coatl
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 89594399;

pub const LISTED_SERIES: &[u16] = &[SET_REPTILIANNE];
// In the hand, with Reptilianne Viper (DARK Reptile) on the field and a 0 ATK Viper of the opponent's, Reptilianne Lamia to
// summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (42303365, MZONE),
    (42303365, MZONE | OPPONENT),
    (42303365, MZONE | OPPONENT),
    (57647597, HAND),
    (57647597, HAND),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_NONTUNER);
    e2.set_value(duel, Value::Card(|duel, _, sc| i32::from(sc.is_race(duel, RACE_REPTILE))));
    c.register_effect(duel, e2);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_race(duel, RACE_REPTILE)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(spconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_REPTILIANNE]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? < 1 {
        return Ok(());
    }
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let og = duel.get_matching_group(aux::faceup_filter(|c, duel| c.is_attack(duel, &[0])), tp, 0, LOCATION_MZONE, None);
    if ft == 0 || og.is_empty() {
        return Ok(());
    }
    let mut ct = (og.len() as i32).min(ft);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ct = 1;
    }
    let sg = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, None);
    if ct > 0 && !sg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let ssg = sg.select(duel, tp, 1, ct as u8, None)?;
        if !ssg.is_empty() {
            duel.special_summon(ssg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
