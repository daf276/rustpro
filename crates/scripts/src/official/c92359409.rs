// Superdreadnought Rail Cannon Gustav Rocket
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 92359409;
const GUSTAV_MAX: u32 = 56910167;

pub const LISTED_NAMES: &[u32] = &[ID, GUSTAV_MAX];

// Darklord Desire (Level 10) as the Xyz Materials, or Gustav Max with a card to discard, and an opponent's Sangan and Ash Blossom to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (55690251, MZONE),
    (55690251, MZONE),
    (55690251, MZONE),
    (GUSTAV_MAX, MZONE),
    (2118022, HAND),
    (26202165, MZONE | OPPONENT),
    (14558127, HAND | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(10);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> =
        Some(|c, duel, tp, xyzc| c.is_summon_code(duel, Some(xyzc), u64::from(SUMMON_TYPE_XYZ), tp, &[GUSTAV_MAX]) && c.is_faceup(duel));
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        !duel.has_flag_effect(tp, ID) && duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
        let g = duel.get_matching_group(CardId::is_discardable, tp, LOCATION_HAND, 0, None);
        let cancelable = Xyz::proc_cancellable(duel);
        let Some(sc) = g.select_unselect(duel, Some(CardSet::default()), tp, false, cancelable, 1, 1)? else {
            return Ok(false);
        };
        if duel.send_to_grave(sc, REASON_DISCARD | REASON_COST)? > 0 {
            duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
            return Ok(true);
        }
        Ok(false)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, discon);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_PHASE | PHASE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e2.set_target(duel, detachtg);
    e2.set_operation(duel, detachop);
    c.register_effect(duel, e2);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && a.e.get_handler(duel).get_overlay_count(duel) > 0
        && duel.is_chain_disablable(a.ev as u8)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, tp, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, tp, 0);
            duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1000);
        }
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_effect(a.ev as u8)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
        && duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)? > 0
    {
        duel.break_effect()?;
        duel.damage(1 - a.tp, 1000, REASON_EFFECT)?;
    }
    Ok(())
}

fn detachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    if c.get_overlay_count(duel) == 0 {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, a.tp, 0);
    }
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn detachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let b1 = c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT);
    let op = if b1 {
        duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (true, aux::stringid(ID, 4))])?
            .unwrap_or(2)
    } else {
        2
    };
    if op == 1 {
        c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)?;
    } else {
        duel.destroy(c, REASON_EFFECT)?;
    }
    Ok(())
}
