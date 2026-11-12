// Heroic Champion - Gandiva
use crate::prelude::*;
use crate::testing::MZONE;

const ID: u32 = 48009503;

pub struct Procedure<const LV: u32, const CT: usize>;

impl<const LV: u32, const CT: usize> XyzProcedure for Procedure<LV, CT> {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_WARRIOR, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(LV);
    const CT: usize = CT;
}

pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure<4, 2>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: Option<EffectId>, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_controler(duel, 1 - tp) && c.is_level_below(duel, 4) && e.is_none_or(|e| c.is_relate_to_effect(duel, e))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, eg) = (a.tp, a.eg.unwrap_or_default());
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| filter(c, duel, None, tp), 1, None));
    }
    duel.set_target_card(eg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(eg), eg.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| filter(c, duel, Some(e), tp), None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
