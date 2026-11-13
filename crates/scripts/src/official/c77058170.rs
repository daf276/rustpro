// Underclock Taker
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 77058170;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// Sangans (Effect Monsters) to Link Summon with and to weaken, and Elfnote Lucina on the opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn atkfilter(c: CardId, duel: &Duel, lg: CardSet) -> bool {
    c.is_faceup(duel) && c.get_attack(duel) > 0 && lg.is_contains(c)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let lg = e.get_handler(duel).get_linked_group(duel);
    if chkc.is_some() {
        return Ok(false);
    }
    let filter = |c, duel: &Duel| atkfilter(c, duel, lg);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SELF);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    e.set_label_object_group(duel, g);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_OPPO);
    duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let tc1 = e.get_label_object_group(duel).and_then(|g| g.get_first());
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let tc2 = g.get_first().filter(|&first| Some(first) != tc1).or_else(|| g.iter().nth(1));
    if let (Some(tc1), Some(tc2)) = (tc1, tc2)
        && tc1.is_faceup(duel)
        && tc1.is_relate_to_effect(duel, e)
        && tc2.is_faceup(duel)
        && tc2.is_relate_to_effect(duel, e)
    {
        let atk = tc1.get_attack(duel);
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -atk);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc2.register_effect(duel, e1);
    }
    Ok(())
}
