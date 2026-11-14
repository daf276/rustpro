// Branded in Red
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 82738008;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];
pub const LISTED_SERIES: &[u16] = &[SET_DESPIA];

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::banish_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_level_above(duel, 8)
    }

    // Fusion.InHandMat(aux.LimitParams(Card.IsAbleToRemove)).
    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_location(duel, LOCATION_HAND) && c.is_able_to_remove(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_remove);
        (Some(duel.get_matching_group(filter, tp, LOCATION_MZONE, 0, None)), None)
    }

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, _: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        if let (1, Some(tc)) = (chk, tc) {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_description(duel, 3207);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e1);
        }
        Ok(())
    }
}

// Fallen of Albaz and Aluber the Jester of Despia in the Graveyard to add and on the field as materials for Despian Quaeritis.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_ALBAZ, GRAVE),
    (62962630, GRAVE),
    (CARD_ALBAZ, MZONE),
    (62962630, MZONE),
    (62962630, HAND),
    (72272462, EXTRA),
    (72272462, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(72272462)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(
        duel,
        CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON + CATEGORY_TOHAND + CATEGORY_REMOVE,
    );
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel) && ((c.is_set_card(duel, &[SET_DESPIA]) && c.is_monster(duel)) || c.is_code(duel, CARD_ALBAZ))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    if duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? == 0 || !tc.is_location(duel, LOCATION_HAND) {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, tc)?;
    if Fusion::summon_eff_tg::<Params>(duel, a, 0, None)? && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.break_effect()?;
        Fusion::summon_eff_op::<Params>(duel, a)?;
    }
    Ok(())
}
