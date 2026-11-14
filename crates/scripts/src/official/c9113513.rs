// Ostinato
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 9113513;

struct Params;

impl FusionSummonParams for Params {
    const EXACTCOUNT: Option<usize> = Some(2);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_MELODIOUS])
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        Fusion::in_hand_mat(c, duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_grave);
        (Some(duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None)), None)
    }

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, tp: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        let Some(tc) = tc else { return Ok(()) };
        if chk == 0 {
            tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        }
        if chk == 1 {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
            e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
            e1.set_code(duel, EVENT_PHASE + PHASE_END);
            e1.set_count_limit(duel, 1, 0);
            e1.set_label_object_group(duel, Some(tc.into()));
            e1.set_condition_mut(duel, descon);
            e1.set_operation(duel, desop);
            e1.set_reset(duel, RESET_PHASE | PHASE_END);
            duel.register_effect(e1, tp);
        }
        Ok(())
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 2, a.tp, LOCATION_DECK as i32);
        duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, a.tp, LOCATION_GRAVE as i32);
        Ok(false)
    }
}

// In the hand with Schuberta and Bacha (two "Melodious" monsters) in the Extra Deck and "Melodious" monsters in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (57594700, EXTRA),
    (56208713, EXTRA),
    (76990617, DECK),
    (40502912, DECK),
    (79757784, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(57594700)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_MZONE, 0) == 0);
    c.register_effect(duel, e1);
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    if tc.is_some_and(|tc| tc.get_flag_effect(duel, ID) != 0) {
        return true;
    }
    a.e.reset(duel);
    false
}

fn mgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, fusc: CardId, mg: CardSet) -> bool {
    c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_GRAVE)
        && c.get_reason(duel) & (REASON_FUSION | REASON_MATERIAL) == REASON_FUSION | REASON_MATERIAL
        && c.get_reason_card(duel) == Some(fusc)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && fusc.check_fusion_material(duel, Some(mg), Some(c.into()), u64::from(PLAYER_NONE))
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    let mg = tc.get_material(duel);
    let sumtype = tc.get_summon_type(duel);
    if duel.destroy(tc, REASON_EFFECT)? != 0
        && sumtype & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION
        && !mg.is_empty()
        && duel.get_location_count(tp, LOCATION_MZONE) >= mg.len() as i32
        && mg.is_exists(
            duel,
            aux::necro_valley_filter(|c, duel| mgfilter(c, duel, e, tp, tc, mg)),
            mg.len(),
            None,
        )
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.special_summon(mg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
