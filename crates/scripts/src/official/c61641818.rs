// Virtual World Dragon - Longlong
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 61641818;

// On the field with Xyz Import to attach Sangan, Eldeen to Xyz Summon it, and the opponent's Ash Blossom to activate from the hand
// next to a face-up Sangan of another Attribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
    (14558127, HAND | OPPONENT),
    (6367785, MZONE),
    (6367785, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;

    fn exchk(duel: &Duel, g: CardSet, _: u8, _: CardId) -> bool {
        g.get_class_count(duel, |c, duel| c.get_race(duel) as i64) == 1
            && g.get_class_count(duel, |c, duel| i64::from(c.get_attribute(duel))) == 1
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) > 0);
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_NEGATE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, discon);
    e2.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let Some(re) = a.re else { return false };
    if a.rp == tp
        || a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        || !re.is_monster_effect(duel)
        || !duel.is_chain_negatable(a.ev as u8)
    {
        return false;
    }
    let g = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    if g.is_empty() {
        return false;
    }
    let attribute = re.get_handler(duel).get_attribute(duel);
    !g.is_exists(duel, |c, duel| c.is_attribute(duel, attribute), 1, None)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.negate_activation(a.ev as u8);
    Ok(())
}
