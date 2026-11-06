// Port of proc_equip.lua: aux.AddEquipProcedure, an Equip Spell's activation that targets the monster to equip.
use engine::{Args, CardId, ConditionFn, CostFn, Duel, EffectId, Script, Value};

use crate::constant::*;
use crate::proc_workaround::CardWorkaround;

// The parameters of aux.AddEquipProcedure (p, f, eqlimit, cost, tg, op, con, prop): `P` None is nil, any player's
// monster; `f` left nil passes every monster, and `tg` and `op` left nil do nothing.
pub trait EquipProcedure {
    const P: Option<u8> = None;
    const EQLIMIT: Option<fn(&Duel, EffectId, CardId) -> i32> = None;
    const COST: Option<CostFn> = None;
    const CON: Option<ConditionFn> = None;
    const PROP: u32 = 0;

    fn f(_c: CardId, _duel: &Duel, _e: EffectId, _tp: u8) -> bool {
        true
    }

    fn tg(_duel: &mut Duel, _a: &Args, _tc: Option<CardId>) -> Script<()> {
        Ok(())
    }

    fn op(_duel: &mut Duel, _a: &Args) -> Script<()> {
        Ok(())
    }
}

pub struct Equip;

impl Equip {
    pub fn add_procedure<P: EquipProcedure>(duel: &mut Duel, c: CardId) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 1068);
        e1.set_category(duel, CATEGORY_EQUIP);
        e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
        e1.set_code(duel, EVENT_FREE_CHAIN);
        e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_CONTINUOUS_TARGET + P::PROP);
        if let Some(con) = P::CON {
            e1.set_condition(duel, con);
        }
        if let Some(cost) = P::COST {
            e1.set_cost(duel, cost);
        }
        e1.set_target(duel, Equip::target::<P>);
        e1.set_operation(duel, Equip::operation::<P>);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_EQUIP_LIMIT);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_value(duel, Value::Card(P::EQLIMIT.unwrap_or(Equip::limit::<P>)));
        c.register_effect(duel, e2);
        e1
    }

    // Auxiliary.EquipLimit(f).
    fn limit<P: EquipProcedure>(duel: &Duel, e: EffectId, c: CardId) -> i32 {
        P::f(c, duel, e, e.get_handler_player(duel)).into()
    }

    fn filter<P: EquipProcedure>(c: CardId, duel: &Duel, player: u8, e: EffectId, tp: u8) -> bool {
        (player == PLAYER_ALL || c.is_controler(duel, player)) && c.is_faceup(duel) && P::f(c, duel, e, tp)
    }

    fn target<P: EquipProcedure>(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
        let (e, tp) = (a.e, a.tp);
        let player = match P::P {
            Some(0) => tp,
            Some(1) => 1 - tp,
            _ => PLAYER_ALL,
        };
        let filter = |c, duel: &Duel| Equip::filter::<P>(c, duel, player, e, tp);
        if let Some(chkc) = chkc {
            return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel) && filter(chkc, duel));
        }
        if chk == 0 {
            return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
        let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
        P::tg(duel, a, g.and_then(|g| g.get_first()))?;
        duel.set_operation_info(0, CATEGORY_EQUIP, Some(e.get_handler(duel).into()), 1, 0, 0);
        Ok(false)
    }

    fn operation<P: EquipProcedure>(duel: &mut Duel, a: &Args) -> Script<()> {
        let e = a.e;
        if let Some(tc) = duel.get_first_target()
            && tc.is_relate_to_effect(duel, e)
            && tc.is_faceup(duel)
        {
            duel.equip(a.tp, e.get_handler(duel), tc)?;
        }
        P::op(duel, a)
    }
}
