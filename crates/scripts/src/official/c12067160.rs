// Gorgon of Zilofthonia
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 12067160;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with two Sangans (Effect Monsters), with monsters of the opponent to be pointed to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
// No activated effect.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE,
    );
    e0.set_code(duel, EFFECT_FORCE_MZONE);
    e0.set_range(duel, LOCATION_MZONE);
    e0.set_target_range(duel, 1, 0);
    e0.set_value(
        duel,
        Value::Zones(|duel, e, _, _, r| {
            if r == LOCATION_REASON_CONTROL {
                -1
            } else {
                !(e.get_handler(duel).get_linked_zone(duel) as i32)
            }
        }),
    );
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, e, c| e.get_handler(duel).get_linked_group(duel).contains(c));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, discon);
    e2.set_operation(duel, |duel, a| {
        duel.hint(HINT_CARD, 0, ID.into());
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(
        duel,
        Value::Card(|duel, e, _| {
            let linked = e.get_handler(duel).get_linked_group(duel);
            linked
                .filter(duel, CardId::is_faceup, None)
                .get_sum(duel, |c, duel| i64::from(c.get_base_attack(duel))) as i32
        }),
    );
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_value(duel, 1);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).get_linked_group_count(duel) == 0);
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e5.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel)));
    c.register_effect(duel, e5);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let Some(re) = a.re else { return false };
    let rc = re.get_handler(duel);
    re.is_monster_effect(duel)
        && rc.is_relate_to_effect(duel, re)
        && c.get_linked_group(duel).contains(rc)
        && !c.is_status(duel, STATUS_BATTLE_DESTROYED)
}
