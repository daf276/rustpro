// Drytron Gamma Eltanin
use super::c97148796::{register, sptg};
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, MZONE};

const ID: u32 = 60037599;

pub const LISTED_SERIES: &[u16] = &[SET_DRYTRON];
pub const LISTED_NAMES: &[u32] = &[ID];
// Drytron monsters to Tribute, and others in the Graveyard to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (97148796, MZONE),
    (96026108, HAND),
    (97148796, GRAVE),
    (96026108, GRAVE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register::<ID>(
        duel,
        c,
        CATEGORY_SPECIAL_SUMMON,
        sptg::<CATEGORY_SPECIAL_SUMMON, { LOCATION_GRAVE as i32 }>,
        spop,
    );
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DRYTRON])
        && c.is_monster(duel)
        && c.is_attack(duel, &[2000])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && !c.is_code(duel, ID)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? > 0 {
        let g = duel.get_matching_group(
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
            tp,
            LOCATION_GRAVE,
            0,
            None,
        );
        if !g.is_empty() && duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.break_effect()?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
