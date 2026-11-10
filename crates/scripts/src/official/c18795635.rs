// GMX Applied Experiment #55
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 18795635;

// GMX Researcher Selande and Dinosaur Normal Monsters (Trakodon, Little D) in the Deck to excavate; no GMX Fusion Monster is
// ported yet, so the Fusion Summon is not reached.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (20049870, DECK), (20049870, DECK), (42348802, DECK), (42625254, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_GMX])
    }

    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    fn extrafil(duel: &Duel, e: EffectId, _: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        (e.get_label_object_group(duel), None)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn gmxfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GMX])
        && c.is_monster(duel)
        && duel.is_existing_matching_card(|c, duel| c.is_race(duel, RACE_DINOSAUR), tp, LOCATION_DECK, 0, 1, Some(c.into()))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| gmxfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let deck_count = duel.get_field_group_count(tp, LOCATION_DECK, 0);
    if deck_count == 0 {
        return Ok(());
    }
    let gmx_g = duel.get_matching_group(|c, duel| gmxfilter(c, duel, tp), tp, LOCATION_DECK, 0, None);
    let dino_g = duel.get_matching_group(|c, duel| c.is_race(duel, RACE_DINOSAUR), tp, LOCATION_DECK, 0, None);
    let sequence = |c: CardId, duel: &Duel| i64::from(c.get_sequence(duel));
    let top_of = |g: &CardSet, duel: &Duel| {
        g.get_max_group(duel, sequence)
            .and_then(|(g, _)| g.get_first())
            .map(|c| c.get_sequence(duel))
    };
    let excav_count = match (top_of(&gmx_g, duel), top_of(&dino_g, duel)) {
        (Some(gmx), Some(dino)) => deck_count - gmx.min(dino) as usize,
        _ => deck_count,
    };
    duel.confirm_decktop(tp, excav_count)?;
    let excav_g = duel.get_decktop_group(tp, excav_count);
    duel.raise_event(excav_g, EVENT_CUSTOM + 1595137, Some(e), REASON_EFFECT, tp, tp, excav_count as u32)?;
    let lp = duel.get_lp(tp) - excav_count as i32 * 400;
    duel.set_lp(tp, lp);
    if !gmx_g.is_empty() && !dino_g.is_empty() {
        let excav_monsters = duel.get_decktop_group(tp, excav_count).filter(duel, CardId::is_monster, None);
        e.set_label_object_group(duel, Some(excav_monsters));
        Fusion::new_summon_eff_closure(duel, e, false);
        if Fusion::summon_eff_tg::<Params>(duel, a, 0, None)? && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.break_effect()?;
            Fusion::new_summon_eff_closure(duel, e, true);
            Fusion::summon_eff_op::<Params>(duel, a)?;
        }
    }
    duel.shuffle_deck(tp)?;
    Ok(())
}
