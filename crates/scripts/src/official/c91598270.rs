// SolSolfachord Gracia
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, PZONE};

const ID: u32 = 91598270;

pub const LISTED_SERIES: &[u16] = &[SET_SOLFACHORD];
// Solfachord Happiness in the Deck to add, and DoSolfachord Cutia (an even Scale) in the Pendulum Zone.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (93481594, DECK),
    (93481594, DECK),
    (55226153, PZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_operation(duel, sucop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_condition(duel, btcon);
    e4.set_operation(duel, btop);
    c.register_effect(duel, e4);
}

fn sucfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SOLFACHORD]) && c.is_type(duel, TYPE_PENDULUM) && c.is_controler(duel, tp) && c.is_pendulum_summoned(duel)
}

fn sucop(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.eg
        .is_some_and(|eg| eg.is_exists(duel, |c, duel| sucfilter(c, duel, a.tp), 1, None))
    {
        duel.set_chain_limit_till_chain_end(
            |duel, e, rp, tp, _| tp == rp || (e.is_spell_trap_effect(duel) && !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE)),
            0,
        );
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SOLFACHORD]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn btcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker()
        .is_some_and(|ac| ac.is_controler(duel, a.tp) && ac.is_type(duel, TYPE_PENDULUM) && ac.is_set_card(duel, &[SET_SOLFACHORD]))
}

fn btop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 0, 1);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            aux::faceup_filter(CardId::is_even_scale),
            a.e.get_handler_player(duel),
            LOCATION_PZONE,
            0,
            1,
            None,
        )
    });
    e1.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel)));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
    c.register_effect(duel, e1);
    Ok(())
}
