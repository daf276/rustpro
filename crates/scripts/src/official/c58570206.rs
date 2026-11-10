// Heavy Polymerization
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 58570206;

struct Params;

impl FusionSummonParams for Params {
    const MINCOUNT: Option<usize> = Some(3);

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let eg = duel.get_matching_group(CardId::is_able_to_remove, tp, LOCATION_EXTRA, 0, None);
        match eg.is_empty() {
            true => (None, None),
            false => (Some(eg), Some((fcheck, e.get_handler(duel)))),
        }
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(extraop);

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, a.tp, LOCATION_EXTRA as i32);
        }
        Ok(true)
    }

    fn stage2(duel: &mut Duel, e: EffectId, _: Option<CardId>, tp: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        if chk == 0 {
            let ct = e.get_label(duel);
            if ct > 0 {
                let lp = duel.get_lp(tp);
                duel.set_lp(tp, lp - ct as i32);
            }
        }
        Ok(())
    }
}

fn extraop(duel: &mut Duel, e: EffectId, _: CardId, _: u8, sg: &mut CardSet) -> Script<()> {
    let rg = sg.filter(duel, |c, duel| c.is_location(duel, LOCATION_EXTRA), None);
    if rg.is_empty() {
        e.set_label(duel, 0);
        return Ok(());
    }
    duel.remove(rg, POS_FACEUP, REASON_EFFECT | REASON_MATERIAL | REASON_FUSION)?;
    let attack = rg.get_sum(duel, |c, duel| i64::from(c.get_attack(duel)));
    e.set_label(duel, attack);
    *sg = sg.difference(rg);
    Ok(())
}

fn fcheck(duel: &Duel, _: CardId, tp: u8, sg: CardSet, _: CardId) -> bool {
    sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_EXTRA), None) <= duel.get_field_group_count(tp, 0, LOCATION_MZONE)
}

// Cyber Eternity Dragon from a Cyber Dragon and two Cyber End Dragons banished from the Extra Deck, which the opponent's two
// monsters allow.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (82315403, EXTRA),
    (1546123, EXTRA),
    (1546123, EXTRA),
    (70095154, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(82315403)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Fusion::register_summon_eff::<Params>(duel, c);
}
