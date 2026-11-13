// Invocation - "Chalice"
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 76334960;
const CHOICE: u8 = 0;

// Aleister the Invoker in the Graveyard to activate, in the hand and on the field to banish, and Kozaky in the Deck and on the
// opponent's field as the other material of Invoked Caliga.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (86120751, GRAVE),
    (86120751, HAND),
    (86120751, MZONE),
    (99171160, DECK),
    (99171160, DECK),
    (99171160, MZONE | OPPONENT),
    (13529466, EXTRA),
    (13529466, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(13529466)];

struct DeckParams;

impl FusionSummonParams for DeckParams {
    const EXACTCOUNT: Option<usize> = Some(2);
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::banish_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_INVOKED])
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_able_to_remove(duel)
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_remove);
        let check = |duel: &Duel, _: CardId, _: u8, sg: CardSet, _: CardId| {
            sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None) == 1
        };
        (
            Some(duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None)),
            Some((check, e.get_handler(duel))),
        )
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(
            0,
            CATEGORY_REMOVE,
            None,
            2,
            a.tp,
            (LOCATION_HAND | LOCATION_ONFIELD | LOCATION_DECK) as i32,
        );
        Ok(false)
    }
}

struct OpponentParams;

impl FusionSummonParams for OpponentParams {
    const EXACTCOUNT: Option<usize> = Some(2);
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::banish_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_INVOKED])
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_on_field(duel) && c.is_able_to_remove(duel)
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, aux::faceup_filter(CardId::is_able_to_remove));
        let check = |duel: &Duel, _: CardId, tp: u8, sg: CardSet, _: CardId| {
            sg.filter_count(duel, |c, duel| c.is_controler(duel, 1 - tp), None) == 1
        };
        (
            Some(duel.get_matching_group(filter, tp, 0, LOCATION_ONFIELD, None)),
            Some((check, e.get_handler(duel))),
        )
    }

    fn extratg(duel: &mut Duel, _: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 2, PLAYER_ALL, LOCATION_ONFIELD as i32);
        Ok(false)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    for op in [false, true] {
        Fusion::new_summon_eff_closure(duel, e1, op);
    }
    c.register_effect(duel, e1);
}

fn confilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ALEISTER]) && c.is_monster(duel) && c.is_faceup(duel)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(confilter, a.tp, LOCATION_MZONE | LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = Fusion::summon_eff_tg::<DeckParams>(duel, a, 0, None)?;
    let b2 = Fusion::summon_eff_tg::<OpponentParams>(duel, a, 0, None)?;
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let choice = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_chain_data(duel, CHOICE, choice.map_or(0, |choice| choice as i64));
    match choice {
        Some(1) => Fusion::summon_eff_tg::<DeckParams>(duel, a, 1, None)?,
        Some(2) => Fusion::summon_eff_tg::<OpponentParams>(duel, a, 1, None)?,
        _ => false,
    };
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    match a.e.get_chain_data(duel, CHOICE) {
        Some(1) => Fusion::summon_eff_op::<DeckParams>(duel, a),
        Some(2) => Fusion::summon_eff_op::<OpponentParams>(duel, a),
        _ => Ok(()),
    }
}
