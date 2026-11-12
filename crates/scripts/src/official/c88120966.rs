// Number 15: Gimmick Puppet Giant Grinder
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 88120966;
pub const XYZ_NUMBER: u32 = 15;

// In the Extra Deck, Xyz Summoned with Level 8 Normal Monsters, with Xyz Monsters of the opponent's (Number 106 and Number 81)
// to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (10000100, MZONE),
    (10949074, MZONE),
    (23923758, MZONE),
    (63746411, MZONE | OPPONENT),
    (49032236, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 2, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, _| duel.is_phase(PHASE_MAIN1));
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.get_summon_type(duel) & SUMMON_TYPE_SPECIAL == SUMMON_TYPE_SPECIAL
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && duel.destroy(tc, REASON_EFFECT)? != 0 && tc.is_type(duel, TYPE_XYZ) {
        duel.break_effect()?;
        let atk = tc.get_base_attack(duel).max(0);
        duel.damage(1 - a.tp, atk as u32, REASON_EFFECT)?;
    }
    Ok(())
}
