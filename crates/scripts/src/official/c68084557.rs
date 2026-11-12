// Power Tool Mecha Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 68084557;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// On the field and in the Extra Deck, Synchro Summoned with Flamvell Archer (a Level 3 Tuner) and Fireyarou (Level 4), with Equip Spells
// (Axe of Despair, United We Stand) in the hand to equip and equipped to move.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (54326448, MZONE),
    (71407486, MZONE),
    (40619825, HAND),
    (40619825, HAND),
    (56747793, HAND),
    (56747793, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_EQUIP);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, drcon);
    e1.set_target(duel, stock::draw_target::<1>);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_target(duel, eqtg);
    e2.set_operation(duel, eqop);
    c.register_effect(duel, e2);
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.tp)
}

fn eqfilter(c: CardId, duel: &Duel, ec: CardId) -> bool {
    c.is_type(duel, TYPE_EQUIP) && c.get_equip_target(duel).is_some_and(|target| target != ec) && c.check_equip_target(duel, ec)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_SZONE) && eqfilter(chkc, duel, c));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c1, duel| eqfilter(c1, duel, c), tp, LOCATION_SZONE, LOCATION_SZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, |c1, duel| eqfilter(c1, duel, c), tp, LOCATION_SZONE, LOCATION_SZONE, 1, 1, None)?;
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && tc.check_equip_target(duel, c) {
        duel.equip(tp, tc, c)?;
    }
    Ok(())
}
