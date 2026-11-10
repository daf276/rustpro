// Doomcaliber Knight
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 78700060;

// Sangan (a monster effect when sent to the Graveyard) on the opponent's side, attacked by random play.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (26202165, MZONE | OPPONENT), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_F);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.re.is_some_and(|re| re.is_monster_effect(duel)));
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let Some(re) = a.re else { return Ok(false) };
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    let rc = re.get_handler(duel);
    if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if u32::from(duel.get_current_chain()) != a.ev + 1 {
        return Ok(());
    }
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
