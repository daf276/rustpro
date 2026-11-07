// Number F0: Utopic Draco Future
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 26973555;

pub const LISTED_NAMES: &[u32] = &[65305468];

// In the Extra Deck, Xyz Summoned with three Evilswarm Exciton Knights.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (46772449, MZONE),
    (46772449, MZONE),
    (46772449, MZONE),
    (46772449, MZONE),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(xyzfilter);
    const LV: Option<u32> = None;
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn exchk(duel: &Duel, g: CardSet, _: u8, _: CardId) -> bool {
        g.get_class_count(duel, |c, duel| i64::from(c.get_rank(duel))) == 1
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_NEGATE + CATEGORY_CONTROL);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, discon);
    e4.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e4.set_target(duel, distg);
    e4.set_operation(duel, disop);
    c.register_effect(duel, e4);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    c.is_faceup(duel) && c.is_summon_code(duel, Some(lc), SUMMON_TYPE_XYZ as u64, tp, &[65305468])
}

fn xyzfilter(c: CardId, duel: &Duel, xyz: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, TYPE_XYZ, Some(xyz), sumtype, tp) && !c.is_set_card(duel, &[SET_NUMBER])
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.ep == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) && duel.is_chain_negatable(a.ev as u8)
}

fn controllable(duel: &Duel, re: EffectId) -> bool {
    let rc = re.get_handler(duel);
    rc.is_on_field(duel) && rc.is_relate_to_effect(duel, re) && rc.is_able_to_change_controler(duel)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if a.re.is_some_and(|re| controllable(duel, re)) {
        duel.set_operation_info(0, CATEGORY_CONTROL, a.eg, 1, 0, 0);
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && controllable(duel, re) {
        duel.break_effect()?;
        duel.get_control(re.get_handler(duel), a.tp)?;
    }
    Ok(())
}
