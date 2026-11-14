// Appliancer Celtopus
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 78225596;

pub const LISTED_SERIES: &[u16] = &[SET_APPLIANCER];
// Link Summoned from two Appliancer monsters, with Appliancer Link monsters to co-link with, opponent's monsters to fight and Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (78447174, MZONE),
    (78447174, MZONE),
    (78447174, MZONE),
    (4729591, MZONE),
    (81769387, MZONE),
    (78447174, HAND),
    (78447174, DECK),
    (3507053, EXTRA),
    (13597785, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
// A co-link needs Celtopus Link Summoned into the Extra Monster Zone with another Appliancer Link Monster in the zone it points to; random play rarely gets it.
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_APPLIANCER], Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_linked(duel));
    e1.set_value(duel, Value::Card(aux::imval1));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_TRIGGER_F + EFFECT_TYPE_FIELD);
    e3.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, atkcon);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_LEAVE_FIELD_P);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_condition(duel, checkcon);
    e4.set_operation(duel, |duel, a| {
        a.e.get_handler(duel)
            .register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_CHAIN, 0, 1);
        Ok(())
    });
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_DRAW);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e5.set_code(duel, EVENT_DESTROYED);
    e5.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) > 0);
    e5.set_target(duel, stock::draw_target::<1>);
    e5.set_operation(duel, stock::draw);
    c.register_effect(duel, e5);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let (Some(mut atk), Some(mut def)) = (duel.get_attacker(), duel.get_attack_target()) else {
        return false;
    };
    if atk.is_controler(duel, 1 - tp) {
        (atk, def) = (def, atk);
    }
    atk.is_controler(duel, tp)
        && atk.is_link_monster(duel)
        && atk.is_set_card(duel, &[SET_APPLIANCER])
        && atk != c
        && def.is_controler(duel, 1 - tp)
        && atk.get_mutual_linked_group(duel).is_contains(c)
        && c.get_mutual_linked_group(duel).is_contains(atk)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let atkct = c.get_mutual_linked_group(duel).filter(duel, CardId::is_monster, None).len();
    let Some(attacker) = duel.get_attacker() else { return Ok(()) };
    let atk = if attacker.is_controler(duel, 1 - tp) {
        duel.get_attack_target()
    } else {
        Some(attacker)
    };
    if let Some(atk) = atk
        && atk.is_relate_to_battle(duel)
        && atk.is_faceup(duel)
        && atk.is_controler(duel, tp)
        && atkct > 0
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE_CAL);
        e1.set_value(duel, atkct as i32 * 1000);
        atk.register_effect(duel, e1);
    }
    Ok(())
}

fn checkfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let hc = e.get_handler(duel);
    c.is_set_card(duel, &[SET_APPLIANCER])
        && c.is_link_monster(duel)
        && c.is_controler(duel, tp)
        && c.is_reason(duel, REASON_DESTROY)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && !(c.get_mutual_linked_group(duel).is_contains(hc) && hc.get_mutual_linked_group(duel).is_contains(c))
}

fn checkcon(duel: &Duel, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    let (c, eg) = (e.get_handler(duel), a.eg.unwrap_or_default());
    !eg.is_contains(c) && eg.is_exists(duel, |tc, duel| checkfilter(tc, duel, e, tp), 1, Some(c.into()))
}
