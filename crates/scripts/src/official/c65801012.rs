// Cynet Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 65801012;
const DIPLEXER_CHIMERA: u32 = 78084378;

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(extraop);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_CYBERSE)
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        if duel.get_field_group_count(tp, LOCATION_EMZONE, 0) == 0
            && duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_none()
        {
            let eg = duel.get_matching_group(exfilter0, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
            if !eg.is_empty() {
                return (Some(eg), Some((fcheck, e.get_handler(duel))));
            }
        }
        (None, None)
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 0, a.tp, LOCATION_GRAVE as i32);
        Ok(false)
    }
}

// In the hand, with Cyberse Normal Monsters and Link Spider on the field and in the Graveyard as materials for Diplexer Chimera.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (DIPLEXER_CHIMERA, EXTRA),
    (24154052, MZONE),
    (24154052, HAND),
    (98978921, MZONE),
    (98978921, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(DIPLEXER_CHIMERA)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Fusion::register_summon_eff::<Params>(duel, c);
}

fn fcheck(duel: &Duel, _: CardId, _: u8, sg: CardSet, _: CardId) -> bool {
    sg.filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None).len() <= 1
}

fn exfilter0(c: CardId, duel: &Duel) -> bool {
    c.is_link_monster(duel) && c.is_race(duel, RACE_CYBERSE) && c.is_able_to_remove(duel)
}

fn extraop(duel: &mut Duel, _: EffectId, _: CardId, _: u8, sg: &mut CardSet) -> Script<()> {
    let rg = sg.filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
    if !rg.is_empty() {
        duel.remove(rg, POS_FACEUP, REASON_EFFECT | REASON_MATERIAL | REASON_FUSION)?;
        *sg = sg.difference(rg);
    }
    Ok(())
}
