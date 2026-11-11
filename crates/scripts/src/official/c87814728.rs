// Magnet Warrior Sigma Minus
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 87814728;

struct Params;

impl FusionSummonParams for Params {
    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_race(duel, RACE_ROCK)
    }

    fn gc(duel: &Duel, e: EffectId, _: u8) -> Option<CardSet> {
        Some(Fusion::forced_handler(duel, e))
    }
}

// In the hand to Fusion Summon Conduction Warrior Plasma Magnum with Alpha and Beta, and on the field to negate the attack
// of the opponent's Burglar. No Level 8 Magna Warrior is ported, so the Deck send is only reported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (47247792, EXTRA),
    (99785935, HAND),
    (39256679, HAND),
    (6297941, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, atknegcon);
    e1.set_operation(duel, |duel, _| {
        duel.negate_attack()?;
        Ok(())
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_reveal);
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_description(duel, aux::stringid(ID, 2));
    e3a.set_category(duel, CATEGORY_TOGRAVE);
    e3a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3a.set_property(duel, EFFECT_FLAG_DELAY);
    e3a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3a.set_count_limit(duel, 1, (ID, 1));
    e3a.set_target(duel, tgtg);
    e3a.set_operation(duel, tgop);
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3b);
}

fn atknegcon(duel: &Duel, _: &Args) -> bool {
    let Some(bc1) = duel.get_attacker() else { return false };
    bc1.is_attribute(duel, ATTRIBUTE_EARTH)
        && duel
            .get_attack_target()
            .is_some_and(|bc2| bc2.is_attribute(duel, ATTRIBUTE_EARTH) && bc2.is_faceup(duel))
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[8]) && c.is_set_card(duel, &[SET_MAGNA_WARRIOR]) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}
