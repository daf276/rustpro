// Port of proc_persistent.lua.
use engine::{Args, CardId, CardSet, ConditionFn, CostFn, Duel, EffectId, OperationFn, Script};

use crate::constant::*;
use crate::proc_workaround::CardWorkaround;

// The parameters of aux.AddPersistentProcedure: `P` None is nil, any monster; `tg_check` is tg called with chk 0.
pub trait PersistentProcedure {
    const P: Option<u8> = None;
    const CATEGORY: u64 = 0;
    const PROPERTY: u32 = 0;
    const HINT1: Option<u32> = None;
    const HINT2: Option<u32> = None;
    const CON: Option<ConditionFn> = None;
    const COST: Option<CostFn> = None;
    const OP: Option<OperationFn> = None;
    const ANYPOS: bool = false;

    fn f(_c: CardId, _duel: &Duel, _e: EffectId, _tp: u8) -> bool {
        true
    }

    fn tg_check(_duel: &Duel, _a: &Args, _tc: CardId) -> bool {
        true
    }

    fn tg(_duel: &mut Duel, _a: &Args, _tc: CardId) -> Script<()> {
        Ok(())
    }
}

pub struct Persistent;

impl Persistent {
    pub fn add_procedure<P: PersistentProcedure>(duel: &mut Duel, c: CardId) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 1068);
        if P::CATEGORY != 0 {
            e1.set_category(duel, P::CATEGORY);
        }
        e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
        e1.set_code(duel, EVENT_FREE_CHAIN);
        match (P::HINT1, P::HINT2) {
            (None, None) => {}
            (Some(hint1), Some(hint2)) if hint1 == hint2 => e1.set_hint_timing(duel, hint1, hint1),
            (hint1, hint2) => e1.set_hint_timing(duel, hint1.unwrap_or(0), hint2.unwrap_or(0)),
        }
        e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + P::PROPERTY);
        if let Some(con) = P::CON {
            e1.set_condition(duel, con);
        }
        if let Some(cost) = P::COST {
            e1.set_cost(duel, cost);
        }
        e1.set_target(duel, Persistent::target::<P>);
        if let Some(op) = P::OP {
            e1.set_operation(duel, op);
        }
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_range(duel, LOCATION_SZONE);
        e2.set_code(duel, EVENT_CHAIN_SOLVED);
        e2.set_label_object(duel, Some(e1));
        e2.set_condition(duel, |duel, a| a.re == a.e.get_label_object(duel));
        e2.set_promptless_operation(duel, Persistent::tg_op::<P>);
        c.register_effect(duel, e2);
    }

    // `player` is the p of the procedure resolved against tp; `tg` the tg check, which only the existence check passes.
    fn filter<P: PersistentProcedure>(c: CardId, duel: &Duel, player: u8, a: &Args, tg: bool) -> bool {
        (player == PLAYER_ALL || c.is_controler(duel, player)) && P::f(c, duel, a.e, a.tp) && (!tg || P::tg_check(duel, a, c))
    }

    fn target<P: PersistentProcedure>(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
        let tp = a.tp;
        let player = match P::P {
            Some(0) => tp,
            Some(1) => 1 - tp,
            _ => PLAYER_ALL,
        };
        if let Some(chkc) = chkc {
            return Ok(chkc.is_location(duel, LOCATION_MZONE)
                && chkc.is_faceup(duel)
                && Persistent::filter::<P>(chkc, duel, player, a, false));
        }
        if chk == 0 {
            let filter = |c, duel: &Duel| Persistent::filter::<P>(c, duel, player, a, true);
            return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
        let filter = |c, duel: &Duel| Persistent::filter::<P>(c, duel, player, a, false);
        let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
        if let Some(tc) = g.and_then(|g| g.get_first()) {
            P::tg(duel, a, tc)?;
        }
        Ok(false)
    }

    fn tg_op<P: PersistentProcedure>(duel: &mut Duel, a: &Args) -> Script<()> {
        let c = a.e.get_handler(duel);
        let Some(re) = a.re else { return Ok(()) };
        let tc = duel
            .get_chain_info::<Option<CardSet>>(a.ev as u8, CHAININFO_TARGET_CARDS)
            .and_then(|g| g.get_first());
        if let Some(tc) = tc
            && c.is_relate_to_effect(duel, re)
            && (P::ANYPOS || tc.is_faceup(duel))
            && tc.is_relate_to_effect(duel, re)
        {
            c.set_card_target(duel, tc);
            c.create_relation(duel, tc, RESET_EVENT + RESETS_STANDARD);
        }
        Ok(())
    }

    // aux.PersistentTargetFilter.
    pub fn target_filter(duel: &Duel, e: EffectId, c: CardId) -> bool {
        e.get_handler(duel).is_has_card_target(duel, c)
    }
}
