// Fossil Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 59419719;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_FOSSIL])
    }

    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let (own, opponent) = match duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_none() {
            true => (LOCATION_GRAVE, LOCATION_GRAVE),
            false => (LOCATION_ONFIELD, 0),
        };
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_remove);
        (Some(duel.get_matching_group(filter, tp, own, opponent, None)), None)
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::banish_material);

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, tp: u8, mg: Option<CardSet>, chk: i32) -> Script<()> {
        let (Some(tc), Some(sg)) = (tc, mg) else { return Ok(()) };
        if chk != 3 {
            return Ok(());
        }
        let mats = sg.filter(duel, |c, duel| c.is_previous_location(duel, LOCATION_GRAVE), None);
        if mats.is_exists(duel, |c, duel| c.is_previous_controler(duel, tp), 1, None)
            && mats.is_exists(duel, |c, duel| c.is_previous_controler(duel, 1 - tp), 1, None)
        {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_description(duel, 3003);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
            e1.set_property(
                duel,
                EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT,
            );
            e1.set_range(duel, LOCATION_MZONE);
            e1.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel)));
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect_full(duel, e1, true);
        }
        Ok(())
    }

    fn extratg(duel: &mut Duel, _: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, PLAYER_EITHER, LOCATION_GRAVE as i32);
        Ok(false)
    }
}

// In the hand with the two Fossil monsters in the Extra Deck, and Stone Ogre Grotto in each Graveyard as materials (with Raigeki
// from the opponent destroying a Fossil monster, for the copy in the Graveyard to return).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (59531356, EXTRA),
    (83656563, EXTRA),
    (59531356, MZONE),
    (15023985, GRAVE),
    (15023985, GRAVE),
    (15023985, GRAVE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummonedCode(59531356),
    Check::SpecialSummonedCode(83656563),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn thcfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && c.is_set_card(duel, &[SET_FOSSIL])
        && c.is_type(duel, TYPE_FUSION)
        && c.get_previous_controler(duel) == tp
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_position(duel, POS_FACEUP)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| thcfilter(c, duel, a.tp), 1, None)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, c)?;
    }
    Ok(())
}
