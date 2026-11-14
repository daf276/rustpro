// Necro Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE};

const RUBY: u32 = 76614340;

// Gem-Knight Garnets in the Graveyard to banish for Gem-Knight Ruby.
pub const SUPPORT: &[(u32, u16)] = &[(91731841, GRAVE), (91731841, GRAVE), (RUBY, EXTRA), (RUBY, EXTRA)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(RUBY)];

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(extraop);

    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, |c, duel| matfilter(c, duel, tp));
        (
            Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE | LOCATION_MZONE, 0, None)),
            None,
        )
    }

    fn stage2(duel: &mut Duel, e: EffectId, fc: Option<CardId>, _: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        if let (1, Some(fc)) = (chk, fc) {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_description(duel, 3206);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_CANNOT_ATTACK);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            fc.register_effect_full(duel, e1, true);
        }
        Ok(())
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, a.tp, LOCATION_GRAVE as i32);
        }
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn matfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_remove_full(duel, Some(tp), POS_FACEDOWN, REASON_EFFECT) && aux::sp_elim_filter(c, duel, false, false)
}

fn extraop(duel: &mut Duel, _: EffectId, _: CardId, tp: u8, sg: &mut CardSet) -> Script<()> {
    let (faceup, facedown) = sg.split(duel, CardId::is_faceup, None);
    if !faceup.is_empty() {
        duel.hint_selection(faceup);
    }
    if !facedown.is_empty() {
        duel.confirm_cards(1 - tp, facedown)?;
    }
    duel.remove(*sg, POS_FACEDOWN, REASON_EFFECT | REASON_MATERIAL | REASON_FUSION)?;
    sg.clear();
    Ok(())
}
