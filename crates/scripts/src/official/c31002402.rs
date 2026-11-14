// Dogmatikalamity
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 31002402;

struct Params;

impl RitualParams for Params {
    const FORCEDSELECTION: Option<ForcedSelection> = Some(ritcheck);

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_DOGMATIKA])
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8) -> Option<CardSet> {
        Some(duel.get_matching_group(|c, duel| c.has_level(duel) && c.is_able_to_grave(duel), tp, LOCATION_EXTRA, 0, None))
    }

    fn extraop(duel: &mut Duel, mat: CardSet, _: EffectId, _: u8, _: CardId) -> Script<()> {
        let extra_mat = mat.filter(duel, |c, duel| c.is_location(duel, LOCATION_EXTRA), None);
        if !extra_mat.is_empty() {
            duel.send_to_grave(extra_mat, REASON_EFFECT | REASON_MATERIAL | REASON_RITUAL)?;
        } else {
            duel.release_ritual_material(mat)?;
        }
        Ok(())
    }

    fn stage2(duel: &mut Duel, _: CardSet, e: EffectId, tp: u8, _: CardId) -> Script<()> {
        if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
            return Ok(());
        }
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 0));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_target_range(duel, 1, 0);
        e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA));
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
        Ok(())
    }
}

pub const LISTED_SERIES: &[u16] = &[SET_DOGMATIKA];
// White Knight of Dogmatika in the hand to summon, with Level 8 Extra Deck monsters (Road Warrior, Colossal Fighter) to send.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (40352445, HAND), (40352445, HAND), (2322421, EXTRA), (23693634, EXTRA)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(40352445)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Ritual::create_proc::<Params, RITPROC_EQUAL>(duel, c);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    c.register_effect(duel, e1);
}

fn ritcheck(duel: &Duel, _: EffectId, _: u8, g: CardSet, _: CardId) -> (bool, bool) {
    let extra_ct = g.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_EXTRA), None);
    (extra_ct == 0 || g.len() == 1, extra_ct > 1 || (extra_ct == 1 && g.len() > 1))
}
