// Steel Star Regulator
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 32986898;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;
    const MAX: Option<usize> = Some(3);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        !c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned from Cyberse Normal Monsters and an Xyz Monster (Gaming Gamer GG), with weak monsters of the
// opponent's to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (47226949, MZONE),
    (47226949, MZONE),
    (47226949, MZONE),
    (15778492, MZONE),
    (24154052, MZONE | OPPONENT),
    (24154052, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn mfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_type(duel, TYPE_XYZ)
}

fn matcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let g = c.get_material(duel).filter(duel, |mc, duel| !mc.is_type(duel, TYPE_LINK), None);
    let atk =
        g.get_sum(duel, |c, duel| i64::from(c.get_original_level(duel))) + g.get_sum(duel, |c, duel| i64::from(c.get_original_rank(duel)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, (atk * 100) as i32);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD));
    c.register_effect(duel, e1);
    if g.is_exists(duel, mfilter, 1, None) {
        let reset = RESET_EVENT | (RESETS_STANDARD & !(RESET_TOFIELD | RESET_LEAVE | RESET_TEMP_REMOVE));
        c.register_flag_effect_full(duel, ID, reset, EFFECT_FLAG_CLIENT_HINT, 1, 0, aux::stringid(ID, 1));
    }
}

fn desfilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    !c.is_type(duel, TYPE_LINK) && c.get_attack(duel) <= atk && c.is_faceup(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let atk = a.e.get_handler(duel).get_attack(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && desfilter(chkc, duel, atk));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| desfilter(c, duel, atk), tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |c, duel| desfilter(c, duel, atk), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.destroy(tc, REASON_EFFECT)? > 0 && e.get_handler(duel).get_flag_effect(duel, ID) > 0 {
        duel.break_effect()?;
        let atk = (f64::from(tc.get_base_attack(duel)) / 2.0).round().max(0.0) as u32;
        duel.damage(1 - tp, atk, REASON_EFFECT)?;
    }
    Ok(())
}
