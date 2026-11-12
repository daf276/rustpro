// Number 33: Chronomaly Machu Mech
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 39139935;
pub const XYZ_NUMBER: u32 = 33;

// In the Extra Deck, Xyz Summoned with two of three Level 5 monsters, with the opponent's Slifer (ATK follows its hand) as target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (35565537, MZONE),
    (86088138, MZONE),
    (90876561, MZONE),
    (10000020, MZONE | OPPONENT),
    (10000020, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DAMAGE + CATEGORY_ATKCHANGE);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_attack(duel) != c.get_base_attack(duel)
}

fn difference(duel: &Duel, tc: CardId) -> i32 {
    let (atk, batk) = (tc.get_attack(duel), tc.get_base_attack(duel));
    if batk > atk { batk - atk } else { atk - batk }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if let Some(tc) = g.and_then(|g| g.get_first()) {
        let dam = difference(duel, tc);
        duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && tc.get_attack(duel) != tc.get_base_attack(duel) {
        let dif = difference(duel, tc);
        let dam = duel.damage(1 - tp, dif as u32, REASON_EFFECT)?;
        if dam > 0 && c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, dam as i32);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
            c.register_effect(duel, e1);
        }
    }
    Ok(())
}
