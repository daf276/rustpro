// Dark Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 94820406;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_FIEND)
    }

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, _: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        if let (1, Some(tc)) = (chk, tc) {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_description(duel, 3061);
            e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
            tc.register_effect(duel, e1);
        }
        Ok(())
    }
}

// Invoked Purgatrio (a Fiend Fusion Monster) in the Extra Deck, with Aleister the Invoker and the FIRE Masked HERO Furnace
// in the hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (12307878, EXTRA), (86120751, HAND), (58288218, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(12307878)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Fusion::register_summon_eff::<Params>(duel, c);
}
