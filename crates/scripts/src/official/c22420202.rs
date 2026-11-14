// Drytron Delta Altais
use super::c97148796::{register, sptg};
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 22420202;

pub const LISTED_SERIES: &[u16] = &[SET_DRYTRON];
// Drytron monsters to Tribute, and a Ritual Monster and Ritual Spell in hand to reveal.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (97148796, MZONE),
    (96026108, HAND),
    (69815951, HAND),
    (22398665, HAND),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register::<ID>(duel, c, CATEGORY_SPECIAL_SUMMON + CATEGORY_DRAW, sptg::<CATEGORY_DRAW, 1>, spop);
}

fn drfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_ritual_monster(duel) || c.is_ritual_spell(duel)) && !c.is_public(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? > 0 {
        let g = duel.get_matching_group(drfilter, tp, LOCATION_HAND, 0, None);
        if !g.is_empty() && duel.is_player_can_draw(tp, 1) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.break_effect()?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.confirm_cards(1 - tp, sg)?;
            duel.shuffle_hand(tp)?;
            duel.draw(tp, 1, REASON_EFFECT)?;
        }
    }
    Ok(())
}
