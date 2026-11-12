// Naturia Beast
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 33198837;

// In the Extra Deck, Synchro Summoned with Naturia Vein (Level 1 EARTH Tuner) and Celtic Guardian (Level 4), with the opponent's
// Dark Hole in the hand to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (16940215, MZONE),
    (91152256, MZONE),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_EARTH, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| {
            c.is_attribute_full(duel, ATTRIBUTE_EARTH, Some(sc), sumtype, tp)
        })
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, discon);
    e1.set_cost(duel, discost);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.re
            .is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.is_spell_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck_as_cost(tp, 2));
    }
    duel.discard_deck(tp, 2, REASON_COST)?;
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if !c.is_faceup(duel) || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
