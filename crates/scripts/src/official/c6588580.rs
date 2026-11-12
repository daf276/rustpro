// Thermal Genex
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 6588580;
const GENEX_CONTROLLER: u32 = 68505803;

pub const MATERIAL: &[u32] = &[GENEX_CONTROLLER];
pub const LISTED_NAMES: &[u32] = &[GENEX_CONTROLLER];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[GENEX_CONTROLLER])
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_attribute_full(duel, ATTRIBUTE_FIRE, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned from Genex Controller (a Level 3 Tuner) and Garoozis (FIRE, Level 5), against the
// opponent's Psychic Kappa to destroy by battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (68505803, MZONE),
    (68505803, MZONE),
    (14977074, MZONE),
    (14977074, MZONE),
    (7892180, MZONE | OPPONENT),
    (7892180, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, _, c| {
            duel.get_matching_group_count(
                |c, duel| c.is_attribute(duel, ATTRIBUTE_FIRE),
                c.get_controler(duel),
                LOCATION_GRAVE,
                0,
                None,
            ) as i32
                * 200
        }),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_condition(duel, damcon);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_relate_to_battle(duel) && c.get_battle_target(duel).is_some_and(|bc| bc.is_monster(duel))
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 0);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d = duel.get_matching_group_count(|c, duel| c.is_set_card(duel, &[SET_GENEX]), tp, LOCATION_GRAVE, 0, None) as u32 * 200;
    duel.damage(p, d, REASON_EFFECT)?;
    Ok(())
}
