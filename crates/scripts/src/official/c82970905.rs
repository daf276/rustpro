// Father Grizzly
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE, OPPONENT};

const ID: u32 = 82970905;

// Copies on the field and in the Graveyard, Level 4 monsters with 1400 ATK (Water Omotics) in the Deck, and the opponent's strong
// attacker and Sangan (a monster effect to negate) with a Normal Monster on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, GRAVE),
    (2483611, DECK),
    (2483611, DECK),
    (2483611, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.is_monster_effect(duel))
            && a.ep == 1 - a.tp
            && duel.is_chain_negatable(a.ev as u8)
            && duel.is_existing_matching_card(aux::faceup_filter(CardId::is_non_effect_monster), a.tp, LOCATION_MZONE, 0, 1, None)
    });
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
}

fn normalspconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[4]) && c.is_attack(duel, &[1400])
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, normal_chk: bool) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && (normalspconfilter(c, duel) || (normal_chk && c.is_type(duel, TYPE_NORMAL)))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let normal_chk = duel.is_existing_matching_card(normalspconfilter, tp, LOCATION_GRAVE, 0, 2, None);
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, normal_chk), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let normal_chk = duel.is_existing_matching_card(normalspconfilter, tp, LOCATION_GRAVE, 0, 2, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, normal_chk), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, tp, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, tp, 0);
        }
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8) && a.re.is_some_and(|re| re.get_handler(duel).is_relate_to_effect(duel, re)) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
