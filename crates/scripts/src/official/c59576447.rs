// Verdict of Anubis
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT, SZONE};

const ID: u32 = 59576447;

pub const LISTED_NAMES: &[u32] = &[CARD_TEMPLE_OF_THE_KINGS];

// Copies and Temple of the Kings in the Spell & Trap Zone for the three other Spells/Traps, the opponent's Dark Hole and
// Raigeki to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (CARD_TEMPLE_OF_THE_KINGS, SZONE),
    (CARD_TEMPLE_OF_THE_KINGS, SZONE),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.ep == 1 - a.tp
        && a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && duel.is_chain_negatable(a.ev as u8)
        && duel.is_existing_matching_card(
            CardId::is_spell_trap,
            a.tp,
            LOCATION_ONFIELD,
            0,
            3,
            Some(a.e.get_handler(duel).into()),
        )
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, a.tp, 0);
        }
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !(duel.negate_activation(a.ev as u8)
        && a.re.is_some_and(|re| re.get_handler(duel).is_relate_to_effect(duel, re))
        && duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)? > 0
        && duel.is_existing_matching_card(
            |c, duel| c.is_faceup(duel) && c.is_code(duel, CARD_TEMPLE_OF_THE_KINGS),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        ))
    {
        return Ok(());
    }
    let g = duel.get_matching_group(CardId::is_monster, tp, 0, LOCATION_MZONE, None);
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        return Ok(());
    }
    duel.break_effect()?;
    if duel.destroy(g, REASON_EFFECT)? > 0 {
        let dam = duel
            .get_operated_group()
            .get_sum(duel, |c, duel| i64::from(c.get_base_attack(duel))) as f64
            / 2.0;
        duel.break_effect()?;
        duel.damage(1 - tp, dam.round() as u32, REASON_EFFECT)?;
    }
    Ok(())
}
