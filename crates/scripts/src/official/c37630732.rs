// Power Bond
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 37630732;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_MACHINE)
    }

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, tp: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        let (c, Some(tc), 1) = (e.get_handler(duel), tc, chk) else {
            return Ok(());
        };
        let attack = tc.get_base_attack(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, attack);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect_full(duel, e1, true);
        if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
            e2.set_code(duel, EVENT_PHASE + PHASE_END);
            e2.set_count_limit(duel, 1, 0);
            e2.set_label(duel, attack.into());
            e2.set_reset(duel, RESET_PHASE | PHASE_END);
            e2.set_operation(duel, damop);
            duel.register_effect(e2, tp);
        }
        Ok(())
    }
}

// Cyber Twin Dragon in the Extra Deck to Fusion Summon from two Cyber Dragons.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (74157028, EXTRA), (70095154, MZONE), (70095154, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(74157028)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    c.register_effect(duel, e1);
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let amount = a.e.get_label(duel) as u32;
    duel.damage(a.tp, amount, REASON_EFFECT)?;
    Ok(())
}
