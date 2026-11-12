// Trickstar Colchica
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 298846;

pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_TRICKSTAR], Some(lc), sumtype, tp) && !c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp)
    }
}

// In the Graveyard, with Trickstar Links on the field to be destroyed in battle by the opponent's attacker.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, GRAVE),
    (77307161, MZONE),
    (77307161, MZONE),
    (14365823, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property_full(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, damcond);
    e1.set_cost(duel, Cost::self_banish);
    e1.set_target(duel, damtg);
    e1.set_operation(duel, damop);
    c.register_effect(duel, e1);
}

fn damconfilter(c: CardId, duel: &Duel, tp: u8, ct: usize) -> bool {
    let bc = c.get_battle_target(duel);
    (c.is_previous_controler(duel, tp) && c.is_previous_set_card(duel, &[SET_TRICKSTAR]))
        || (ct == 1 && bc.is_some_and(|bc| bc.is_controler(duel, tp) && bc.is_set_card(duel, &[SET_TRICKSTAR])))
}

fn damcond(duel: &Duel, a: &Args) -> bool {
    let (tp, eg) = (a.tp, a.eg.unwrap_or_default());
    eg.is_exists(duel, |c, duel| damconfilter(c, duel, tp, eg.len()), 1, None) && !eg.is_contains(a.e.get_handler(duel))
}

fn damtgfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_can_be_effect_target(duel, e) && c.get_base_attack(duel) > 0
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let eg = a.eg.unwrap_or_default();
    if let Some(chkc) = chkc {
        return Ok(eg.is_contains(chkc) && damtgfilter(chkc, duel, e));
    }
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| damtgfilter(c, duel, e), 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let tc = if eg.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
        eg.filter_select(duel, tp, |c, duel| damtgfilter(c, duel, e), 1, 1, None)?
            .get_first()
    } else {
        eg.get_first()
    };
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(false) };
    duel.set_target_card(tc);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, tc.get_base_attack(duel));
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.get_base_attack(duel) > 0
    {
        duel.damage(1 - a.tp, tc.get_base_attack(duel) as u32, REASON_EFFECT)?;
    }
    Ok(())
}
