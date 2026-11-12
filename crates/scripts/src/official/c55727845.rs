// Number 96: Dark Mist
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 55727845;
pub const XYZ_NUMBER: u32 = 96;

// In the Extra Deck, Xyz Summoned with Level 2 Normal Monsters, with the opponent's Elfnote Lucina and Ash Blossom to attack into it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (756652, MZONE),
    (2830619, MZONE),
    (3985011, MZONE),
    (4920010, MZONE),
    (13597785, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        let at = duel.get_attack_target();
        return Ok(
            ((at.is_some_and(|at| at.is_faceup(duel)) && duel.get_attacker() == Some(c)) || at == Some(c))
                && !c.is_status(duel, STATUS_CHAINING),
        );
    }
    if let Some(tc) = c.get_battle_target(duel) {
        duel.set_target_card(tc);
    }
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && !tc.is_immune_to_effect(duel, e)
    {
        // Lua halves with a float, which the core rounds.
        let atk = (f64::from(tc.get_attack(duel)) / 2.0).round() as i32;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, atk);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_range(duel, LOCATION_MZONE);
        e2.set_code(duel, EFFECT_UPDATE_ATTACK);
        e2.set_value(duel, atk);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e2);
    }
    Ok(())
}
