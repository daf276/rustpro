// Junk Warrior
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 60800381;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[CARD_JUNK_SYNCHRON]) || c.is_has_effect(duel, 20932152).is_some()
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const MATERIAL: &[u32] = &[CARD_JUNK_SYNCHRON];
pub const LISTED_NAMES: &[u32] = &[CARD_JUNK_SYNCHRON];
// In the Extra Deck, Synchro Summoned with Junk Synchron and Atlantean Pikeman, a Level 2 Normal Monster.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (CARD_JUNK_SYNCHRON, MZONE), (26976414, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel)) {
        return Ok(());
    }
    let g = duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_level_below(duel, 2)),
        a.tp,
        LOCATION_MZONE,
        0,
        None,
    );
    let atk = g.get_sum(duel, |c, duel| c.get_attack(duel).into());
    if atk > 0 {
        c.update_attack(duel, atk as i32, None, None, None);
    }
    Ok(())
}
