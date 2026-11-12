// Turbo Cannon
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 13574687;
const TURBO_ROCKET: u32 = 6142213;

pub const LISTED_NAMES: &[u32] = &[TURBO_ROCKET];
pub const MATERIAL: &[u32] = &[TURBO_ROCKET];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[TURBO_ROCKET])
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Turbo Rocket (Level 2 Tuner) and Key Mace (Level 1).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (TURBO_ROCKET, MZONE),
    (TURBO_ROCKET, MZONE),
    (1929294, MZONE),
    (1929294, MZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

// The core rounds the Lua's `atk/2` where it reads it as an integer.
fn half_attack(duel: &Duel, c: CardId) -> i32 {
    (c.get_attack(duel) + 1) / 2
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| c.is_faceup(duel), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |c, duel| c.is_faceup(duel), tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    let tc = g.and_then(|g| g.get_first());
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(false) };
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, tc.get_controler(duel), half_attack(duel, tc));
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        let dam = half_attack(duel, tc);
        let p = tc.get_controler(duel);
        if duel.destroy(tc, REASON_EFFECT)? != 0 {
            duel.damage(p, dam as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
