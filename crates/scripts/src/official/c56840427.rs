// Number C39: Utopia Ray
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 56840427;
pub const XYZ_NUMBER: u32 = 39;

pub const LISTED_NAMES: &[u32] = &[84013237];

// On the field with Xyz Import to attach the opponent's monsters, and in the Extra Deck to Xyz Summon over a Number 39: Utopia, with
// Solemn Judgments halving the owner's Life Points against the opponent's Dark Hole and Raigekis.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (84013237, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (41420027, SZONE),
    (41420027, SZONE),
    (41420027, SZONE),
    (26202165, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, xyz, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(xyz), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| duel.get_lp(a.tp) <= 1000);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    c.is_faceup(duel) && c.is_summon_code(duel, Some(lc), SUMMON_TYPE_XYZ as u64, tp, &[84013237])
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(CardId::is_faceup, a.tp, 0, LOCATION_MZONE, 1, None))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_lp(tp) > 1000 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel)) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 500);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    if let Some(sc) = duel
        .select_matching_card(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, None)?
        .get_first()
    {
        duel.hint_selection(sc);
        let e2 = e1.clone_effect(duel);
        e2.set_value(duel, -1000);
        sc.register_effect(duel, e2);
    }
    Ok(())
}
