// Ultimate Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 71143015;
const CARD_BLUEEYES_U_DRAGON: u32 = 23995346;

pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON, CARD_BLUEEYES_U_DRAGON];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.lists_code_as_material(duel, &[CARD_BLUEEYES_W_DRAGON, CARD_BLUEEYES_U_DRAGON])
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_able_to_deck(duel)
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = aux::necro_valley_filter(|c, duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_deck));
        (
            Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None)),
            Some((fcheck, e.get_handler(duel))),
        )
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::shuffle_material);

    fn stage2(duel: &mut Duel, _: EffectId, _: Option<CardId>, tp: u8, mg: Option<CardSet>, chk: i32) -> Script<()> {
        if chk != 0 {
            return Ok(());
        }
        let g = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_ONFIELD, None);
        let mc = mg.map_or(0, |mg| mg.filter_count(duel, desfilter, None));
        if !g.is_empty() && mc > 0 && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let sg = g.select(duel, tp, 1, mc as u8, None)?;
            if !sg.is_empty() {
                duel.break_effect()?;
                duel.destroy(sg, REASON_EFFECT)?;
            }
        }
        Ok(())
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            let tp = a.tp;
            duel.set_operation_info(
                0,
                CATEGORY_TODECK,
                None,
                0,
                tp,
                (LOCATION_HAND | LOCATION_MZONE | LOCATION_GRAVE) as i32,
            );
            duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 0, 1 - tp, LOCATION_ONFIELD as i32);
        }
        Ok(true)
    }
}

// Blue-Eyes Ultimate Dragon in the Extra Deck, Fusion Summoned by shuffling two Blue-Eyes White Dragons from the field and the
// Graveyard and a Blue-Eyes White Dragon from the hand, with a monster of the opponent's to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_BLUEEYES_U_DRAGON, EXTRA),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
    (CARD_BLUEEYES_W_DRAGON, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(CARD_BLUEEYES_U_DRAGON)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    c.register_effect(duel, e1);
}

// sumtype is that of the Fusion Summon effect, which the check is only called for.
fn fcheck(duel: &Duel, _: CardId, tp: u8, sg: CardSet, fc: CardId) -> bool {
    sg.is_exists(
        duel,
        |c, duel| forcedmatfilter(c, duel, fc, u64::from(SUMMON_TYPE_FUSION) | MATERIAL_FUSION, tp),
        1,
        None,
    )
}

fn forcedmatfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    (c.is_code(duel, CARD_BLUEEYES_W_DRAGON) || c.is_code(duel, CARD_BLUEEYES_U_DRAGON))
        && c.is_summon_code(duel, Some(fc), sumtype, tp, &Fusion::material(duel, fc.get_original_code(duel)))
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    let code = c.get_previous_code_on_field(duel);
    (code == CARD_BLUEEYES_W_DRAGON || code == CARD_BLUEEYES_U_DRAGON) && c.is_previous_location(duel, LOCATION_ONFIELD)
}
