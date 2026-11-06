// Port of proc_normal.lua: aux.AddNormalSummonProcedure, a Normal Summon by other tributes than the level asks for, or
// the only way to Normal Summon the card. aux.AddNormalSetProcedure needs the Set procedures of MonsterSet, which come
// with the first card that registers one; the metatable's min_tribute_req/max_tribute_req have no reader yet.
use engine::{Args, CardId, CardSet, Duel, EffectId, Script};

use crate::constant::*;
use crate::utility::aux;

// The parameters of aux.AddNormalSummonProcedure (ns, opt, min, max, val, desc, f, sumop): `f` and `sumop` left nil
// pass every card and do nothing.
pub trait NormalProcedure {
    const NS: bool;
    const OPT: bool;
    const MIN: i32 = 0;
    const MAX: i32 = 0;
    const VAL: u32 = SUMMON_TYPE_TRIBUTE;
    const DESC: Option<u64> = None;

    fn f(_c: CardId, _duel: &Duel, _tp: u8) -> bool {
        true
    }

    fn sumop(_duel: &mut Duel, _g: CardSet, _a: &Args) -> Script<()> {
        Ok(())
    }
}

pub struct Normal;

impl Normal {
    pub fn add_summon_procedure<P: NormalProcedure>(duel: &mut Duel, c: CardId) -> EffectId {
        let e1 = duel.create_effect(c);
        if let Some(desc) = P::DESC {
            e1.set_description(duel, desc);
        }
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(
            duel,
            if P::NS && P::OPT {
                EFFECT_SUMMON_PROC
            } else {
                EFFECT_LIMIT_SUMMON_PROC
            },
        );
        if P::NS {
            e1.set_condition(duel, Normal::summon_condition::<P>);
            e1.set_target(duel, Normal::summon_target::<P>);
            e1.set_operation(duel, Normal::summon_operation::<P>);
        } else {
            e1.set_condition(duel, |_, a| a.c.is_none());
        }
        e1.set_value(duel, P::VAL as i32);
        c.register_effect(duel, e1);
        e1
    }

    fn maplevel(level: u32) -> i32 {
        match level {
            5..=6 => 1,
            7.. => 2,
            _ => 0,
        }
    }

    fn tributes<P: NormalProcedure>(duel: &Duel, c: CardId, a: &Args, tp: u8) -> CardSet {
        let mut mg = duel.get_tribute_group(c);
        mg.keep_matching(duel, |mc, duel| aux::is_zone(mc, duel, a.releasable, tp), None);
        mg.keep_matching(duel, |mc, duel| P::f(mc, duel, tp), None);
        mg
    }

    fn summon_condition<P: NormalProcedure>(duel: &Duel, a: &Args) -> bool {
        let Some(c) = a.c else { return true };
        let tp = c.get_controler(duel);
        let mg = Normal::tributes::<P>(duel, c, a, tp);
        let tributes = Normal::maplevel(c.get_level(duel));
        (!P::OPT || (tributes > 0 && tributes != P::MAX))
            && i32::from(a.min) <= P::MIN
            && duel.check_tribute(c, P::MIN, P::MAX, Some(mg), tp, a.zone)
    }

    fn summon_target<P: NormalProcedure>(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
        let Some(c) = a.c else { return Ok(false) };
        let mg = Normal::tributes::<P>(duel, c, a, a.tp);
        let cancelable = duel.is_summon_cancelable();
        let g = duel.select_tribute(a.tp, c, P::MIN, P::MAX, Some(mg), a.tp, a.zone, cancelable)?;
        if g.is_empty() {
            return Ok(false);
        }
        a.e.set_label_object_group(duel, Some(g));
        Ok(true)
    }

    fn summon_operation<P: NormalProcedure>(duel: &mut Duel, a: &Args) -> Script<()> {
        let Some(c) = a.c else { return Ok(()) };
        let g = a.e.get_label_object_group(duel).unwrap_or_default();
        c.set_material(duel, Some(g));
        duel.release(g, REASON_SUMMON + REASON_MATERIAL)?;
        P::sumop(duel, g, a)
    }
}
