// Dark Balter the Terrible
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 80071763;
const POSSESSED_DARK_SOUL: u32 = 52860176;
const FRONTIER_WISEMAN: u32 = 38742075;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(POSSESSED_DARK_SOUL), FusionMaterial::Code(FRONTIER_WISEMAN)];
}

// In the Extra Deck, Fusion Summoned by Polymerization from its materials, and on the field against the opponent's Raigeki
// and Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (POSSESSED_DARK_SOUL, MZONE),
    (FRONTIER_WISEMAN, MZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, discon);
    e1.set_cost(duel, Cost::pay_lp::<1000>);
    e1.set_target(duel, distg);
    e1.set_operation(duel, |duel, a| {
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_BATTLED);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_operation(duel, disop2);
    c.register_effect(duel, e2);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    if a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.get_active_type(duel) == TYPE_SPELL)
        && duel.is_chain_disablable(a.ev as u8)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    Ok(false)
}

fn disop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(bc) = c.get_battle_target(duel) else { return Ok(()) };
    if bc.is_status(duel, STATUS_BATTLE_DESTROYED) && !c.is_status(duel, STATUS_BATTLE_DESTROYED) {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e = duel.create_effect(c);
            e.set_type(duel, EFFECT_TYPE_SINGLE);
            e.set_code(duel, code);
            e.set_reset(duel, RESET_EVENT | RESETS_STANDARD_EXC_GRAVE);
            bc.register_effect(duel, e);
        }
    }
    Ok(())
}
