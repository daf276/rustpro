// Full Armored Utopic Ray Lancer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 1269512;

// In the Extra Deck, Xyz Summoned with three Level 4 Normal Monsters (Vorse Raider), or over Gagaga Cowboy (Rank 4) by discarding a
// Spell, with the opponent's monsters in Attack Position to negate and to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (14898066, MZONE),
    (14898066, MZONE),
    (14898066, MZONE),
    (12014404, MZONE),
    (12580477, HAND),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(|c, duel, tp, xyzc| {
        c.is_faceup(duel) && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), u64::from(SUMMON_TYPE_XYZ), tp) && c.is_rank_below(duel, 4)
    });
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        !duel.has_flag_effect(tp, ID) && duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, None)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
        let g = duel.get_matching_group(cfilter, tp, LOCATION_HAND, 0, None);
        let cancelable = Xyz::proc_cancellable(duel);
        let Some(tc) = g.select_unselect(duel, Some(CardSet::default()), tp, false, cancelable, 1, 1)? else {
            return Ok(false);
        };
        duel.send_to_grave(tc, REASON_DISCARD | REASON_COST)?;
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(duel, -500);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE_START);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_DESTROYING);
    e3.set_condition(duel, exatkcon);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_operation(duel, |duel, _| {
        duel.chain_attack();
        Ok(())
    });
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_discardable(duel)
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attack_pos(duel) && c.is_negatable_monster(duel)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(disfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(disfilter, a.tp, 0, LOCATION_MZONE, None);
    let c = a.e.get_handler(duel);
    for tc in g {
        tc.negate_effects(duel, c, 0, false);
    }
    Ok(())
}

fn exatkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.get_attacker() == Some(c) && duel.check_mut(|duel| c.can_chain_attack(duel)) && aux::bdocon(duel, a)
}
