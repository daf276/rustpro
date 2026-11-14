// Achichi @Ignister
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 15808381;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_IGNISTER];

// Copies in the hand to summon and in the Graveyard, Backup and Bururu @Ignister to add, and Cyberse attackers.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (30118811, DECK),
    (42429678, DECK),
    (3723262, MZONE),
    (3723262, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_START);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition_mut(duel, descon);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_IGNISTER])
        && c.is_monster(duel)
        && c.is_level_below(duel, 4)
        && !c.is_code(duel, ID)
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let tp = a.tp;
    let Some(mut tc) = duel.get_attacker() else { return false };
    if tc.is_controler(duel, 1 - tp) {
        let Some(bc) = duel.get_attack_target() else { return false };
        tc = bc;
    }
    if tc.is_faceup(duel) && tc.is_race(duel, RACE_CYBERSE) {
        a.e.set_label_object_group(duel, Some(tc.into()));
        true
    } else {
        false
    }
}

fn destg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    // The Lua passes an undefined global bc, so no card.
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    // The Lua destroys the stored monster, which is its own Cyberse one.
    if let Some(tc) = tc.filter(|tc| tc.is_controler(duel, a.tp) && tc.is_relate_to_battle(duel)) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
