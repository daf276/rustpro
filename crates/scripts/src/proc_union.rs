// Port of proc_union.lua.
use engine::{Args, CardId, Duel, EffectId, Script, Value};

use crate::constant::*;
use crate::proc_workaround::CardWorkaround;

// The parameters of aux.AddUnionProcedure: `OLD_PROTECT` None takes OLD_EQUIP, which also stands for mt.old_union.
pub trait UnionProcedure {
    const OLD_EQUIP: bool = false;
    const OLD_PROTECT: Option<bool> = None;

    fn f(_c: CardId, _duel: &Duel) -> bool {
        true
    }
}

pub struct Union;

impl Union {
    // The reset of the flag (the union's original code) that allows one equip or unequip per turn.
    const FLAG_RESET: u32 = RESET_EVENT + (RESETS_STANDARD - RESET_TOFIELD - RESET_LEAVE) + RESET_PHASE + PHASE_END;

    pub fn add_procedure<P: UnionProcedure>(duel: &mut Duel, c: CardId) {
        let old_protect = P::OLD_PROTECT.unwrap_or(P::OLD_EQUIP);
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 1068);
        e1.set_category(duel, CATEGORY_EQUIP);
        e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        e1.set_type(duel, EFFECT_TYPE_IGNITION);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_target(duel, Union::target::<P>);
        e1.set_operation(duel, Union::operation::<P>);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_description(duel, 2);
        e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        e2.set_type(duel, EFFECT_TYPE_IGNITION);
        e2.set_range(duel, LOCATION_SZONE);
        e2.set_condition(duel, if P::OLD_EQUIP { Union::is_union_state } else { Union::is_equipped });
        e2.set_target(duel, Union::sum_target::<P>);
        e2.set_operation(duel, Union::sum_operation::<P>);
        c.register_effect(duel, e2);
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_EQUIP);
        e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e3.set_code(duel, EFFECT_DESTROY_SUBSTITUTE);
        e3.set_condition(duel, if old_protect { Union::is_union_state } else { Union::is_equipped });
        let replace: fn(&Duel, EffectId, Option<EffectId>, u32, u8) -> bool = if old_protect {
            |_, _, _, r, _| r & REASON_BATTLE != 0
        } else {
            |_, _, _, r, _| r & REASON_BATTLE != 0 || r & REASON_EFFECT != 0
        };
        e3.set_value(duel, Value::Reason(replace));
        c.register_effect(duel, e3);
        let e4 = duel.create_effect(c);
        e4.set_type(duel, EFFECT_TYPE_SINGLE);
        e4.set_code(duel, EFFECT_UNION_LIMIT);
        e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e4.set_value(duel, Value::Card(Union::limit::<P>));
        c.register_effect(duel, e4);
    }

    fn is_equipped(duel: &Duel, a: &Args) -> bool {
        a.e.get_handler(duel).get_equip_target(duel).is_some()
    }

    // Auxiliary.UnionFilter(c, f, oldrule).
    fn filter<P: UnionProcedure>(c: CardId, duel: &Duel) -> bool {
        let (ct1, ct2) = c.get_union_count(duel);
        c.is_faceup(duel) && P::f(c, duel) && if P::OLD_EQUIP { ct1 == 0 } else { ct2 == 0 }
    }

    // The Lua checks the union itself rather than chkc.
    fn target<P: UnionProcedure>(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
        let (c, tp) = (a.e.get_handler(duel), a.tp);
        let code = c.get_original_code(duel);
        if let Some(chkc) = chkc {
            return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && Union::filter::<P>(c, duel));
        }
        if chk == 0 {
            return Ok(c.get_flag_effect(duel, code) == 0
                && duel.get_location_count(tp, LOCATION_SZONE) > 0
                && duel.is_existing_target(Union::filter::<P>, tp, LOCATION_MZONE, 0, 1, Some(c.into())));
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
        let g = duel.select_target(tp, Union::filter::<P>, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
        duel.set_operation_info(0, CATEGORY_EQUIP, g, 1, 0, 0);
        c.register_flag_effect(duel, code, Union::FLAG_RESET, 0, 1);
        Ok(false)
    }

    fn operation<P: UnionProcedure>(duel: &mut Duel, a: &Args) -> Script<()> {
        let (e, tp) = (a.e, a.tp);
        let c = e.get_handler(duel);
        if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
            return Ok(());
        }
        let Some(tc) = duel.get_first_target() else { return Ok(()) };
        if !tc.is_relate_to_effect(duel, e) || !P::f(tc, duel) {
            duel.send_to_grave(c, REASON_EFFECT)?;
            return Ok(());
        }
        if !duel.equip_full(tp, c, tc, false, false)? {
            return Ok(());
        }
        Union::set_union_state(duel, c, P::OLD_EQUIP);
        Ok(())
    }

    fn position<P: UnionProcedure>() -> u32 {
        if P::OLD_EQUIP { POS_FACEUP_ATTACK } else { POS_FACEUP }
    }

    fn sum_target<P: UnionProcedure>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
        let (e, tp) = (a.e, a.tp);
        let c = e.get_handler(duel);
        let code = c.get_original_code(duel);
        if chk == 0 {
            return Ok(c.get_flag_effect(duel, code) == 0
                && duel.get_location_count(tp, LOCATION_MZONE) > 0
                && c.is_can_be_special_summoned_full(duel, e, 0, tp, true, false, Union::position::<P>(), tp, 0xff));
        }
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
        c.register_flag_effect(duel, code, Union::FLAG_RESET, 0, 1);
        Ok(false)
    }

    fn sum_operation<P: UnionProcedure>(duel: &mut Duel, a: &Args) -> Script<()> {
        let (e, tp) = (a.e, a.tp);
        let c = e.get_handler(duel);
        if !c.is_relate_to_effect(duel, e) {
            return Ok(());
        }
        let pos = Union::position::<P>();
        if duel.special_summon(c, 0, tp, tp, true, false, pos)? == 0
            && duel.get_location_count(tp, LOCATION_MZONE) <= 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, true, false, pos, tp, 0xff)
        {
            duel.send_to_grave(c, REASON_RULE)?;
        }
        Ok(())
    }

    // Auxiliary.UnionLimit(f).
    fn limit<P: UnionProcedure>(duel: &Duel, e: EffectId, c: CardId) -> i32 {
        (P::f(c, duel) || e.get_handler(duel).get_equip_target(duel) == Some(c)).into()
    }

    pub fn is_union_state(duel: &Duel, a: &Args) -> bool {
        a.e.get_handler(duel).is_has_effect(duel, EFFECT_UNION_STATUS).is_some()
    }

    // Auxiliary.SetUnionState(c), given the metatable's old_union of c.
    pub fn set_union_state(duel: &mut Duel, c: CardId, old_union: bool) {
        let limit = c.get_card_effect(duel, EFFECT_UNION_LIMIT).first().map(|&te| te.get_value(duel));
        let e0 = duel.create_effect(c);
        e0.set_type(duel, EFFECT_TYPE_SINGLE);
        e0.set_code(duel, EFFECT_EQUIP_LIMIT);
        e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e0.set_value(duel, limit.unwrap_or_default());
        e0.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
        c.register_effect(duel, e0);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UNION_STATUS);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
        c.register_effect(duel, e1);
        if old_union {
            let e2 = e1.clone_effect(duel);
            e2.set_code(duel, EFFECT_OLDUNION_STATUS);
            c.register_effect(duel, e2);
        }
    }
}
