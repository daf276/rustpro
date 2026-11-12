// Ally of Justice Field Marshal
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 69461394;

// In the Extra Deck, Synchro Summoned with Stinger the Electric Expeller (a Level 3 Tuner) and two Niwatori (Level 3), to attack the
// opponent's Key Maces, which are often face-down, with copies on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (57649113, MZONE),
    (7805359, MZONE),
    (7805359, MZONE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_condition(duel, condition);
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let Some(dg) = a.eg.and_then(|eg| eg.get_first()) else {
        return false;
    };
    a.eg.is_some_and(|eg| eg.len() == 1)
        && dg.is_location(duel, LOCATION_GRAVE)
        && dg.is_reason(duel, REASON_BATTLE)
        && dg.get_battle_position(duel) == POS_FACEDOWN_DEFENSE
        && dg.get_reason_card(duel) == Some(a.e.get_handler(duel))
}
