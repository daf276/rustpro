// Cyberload Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, REMOVED};

const ID: u32 = 55704856;

pub const LISTED_SERIES: &[u16] = &[SET_CYBER_DRAGON];

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::shuffle_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.lists_archetype_as_material(duel, &[SET_CYBER_DRAGON])
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_on_field(duel) && c.is_able_to_deck(duel)
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, |c, duel| c.is_able_to_deck(duel) && c.is_faceup(duel));
        (
            Some(duel.get_matching_group(filter, tp, LOCATION_REMOVED, 0, None)),
            Some((fcheck, e.get_handler(duel))),
        )
    }

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, tp: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        if chk == 1
            && let Some(tc) = tc
        {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_FIELD);
            e1.set_code(duel, EFFECT_CANNOT_ATTACK);
            e1.set_target_range(duel, LOCATION_MZONE, 0);
            let field_id = tc.get_field_id(duel);
            e1.set_label(duel, i64::from(field_id));
            e1.set_target_card(duel, |duel, e, c| e.get_label(duel) != i64::from(c.get_field_id(duel)));
            e1.set_reset(duel, RESET_PHASE | PHASE_END);
            duel.register_effect(e1, tp);
        }
        Ok(())
    }
}

// Cyber Twin Dragon in the Extra Deck, Fusion Summoned from a Cyber Dragon on the field and a banished one.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (74157028, EXTRA),
    (74157028, EXTRA),
    (CARD_CYBER_DRAGON, MZONE),
    (CARD_CYBER_DRAGON, REMOVED),
    (CARD_CYBER_DRAGON, REMOVED),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(74157028)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    c.register_effect(duel, e1);
}

fn fcheck(duel: &Duel, _: CardId, tp: u8, sg: CardSet, fc: CardId) -> bool {
    sg.is_exists(
        duel,
        |c, duel| c.is_set_card_full(duel, &[SET_CYBER_DRAGON], Some(fc), SUMMON_TYPE_FUSION.into(), tp),
        1,
        None,
    )
}
