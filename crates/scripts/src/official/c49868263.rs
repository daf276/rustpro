// Ryu Senshi
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 49868263;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(75953262), FusionMaterial::Code(67957315)];
}

// In the Extra Deck and on the field, Fusion Summoned by Polymerization with Warrior Dai Grepher and Spirit Ryu, with the
// opponent's Trap Hole and Mirror Force to negate and Mage Power to target it. The negation of a targeting Spell is not checked.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (75953262, HAND),
    (67957315, HAND),
    (4206964, SZONE | OPPONENT),
    (44095762, SZONE | OPPONENT),
    (83746708, HAND | OPPONENT),
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
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_operation(duel, disop2);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_DISABLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_SZONE, LOCATION_SZONE);
    e3.set_target_card(duel, distg2);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_SELF_DESTROY);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target_range(duel, LOCATION_SZONE, LOCATION_SZONE);
    e4.set_target_card(duel, distg2);
    c.register_effect(duel, e4);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    if a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    a.re.is_some_and(|re| {
        re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.get_active_type(duel) == TYPE_TRAP && duel.is_chain_disablable(a.ev as u8)
    })
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.negate_effect(a.ev as u8);
    Ok(())
}

fn disop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if re.is_spell_effect(duel) && re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        let g: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
        if g.is_some_and(|g| g.is_contains(a.e.get_handler(duel)))
            && duel.negate_effect(a.ev as u8)
            && re.get_handler(duel).is_relate_to_effect(duel, re)
        {
            duel.destroy(re.get_handler(duel), REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn distg2(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.get_card_target_count(duel) > 0 && c.is_spell(duel) && c.get_card_target(duel).is_contains(e.get_handler(duel))
}
