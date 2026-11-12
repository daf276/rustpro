// Magicalibra
use crate::prelude::*;
use crate::testing::{MZONE, PZONE};

const ID: u32 = 43857222;

// In the Pendulum Zone, with monsters of Levels 3, 4 and 7 on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (36996508, MZONE), (48531733, MZONE), (5628232, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_target(duel, lvtg);
    e1.set_operation(duel, lvop);
    c.register_effect(duel, e1);
}

fn lvfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.has_level(duel) && c.is_can_be_effect_target(duel, e)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| lvfilter(c, duel, e), tp, LOCATION_MZONE, 0, None);
    let hlv = g.get_max_group(duel, |c, duel| c.get_level(duel).into()).map_or(0, |(_, hlv)| hlv);
    if chk == 0 {
        return Ok(g.len() > 1 && hlv > 1);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_LVRANK);
    let lv = duel.announce_number_range(tp, 1, (hlv - 1).min(6) as u32)?;
    let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| {
        (sg.is_exists(duel, |c, duel| c.is_level_above(duel, lv as u32 + 1), 1, None), false)
    };
    let tg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_FACEUP)?;
    duel.set_target_card(tg);
    duel.set_target_param(lv as u32);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let tg = duel.get_target_cards(a.e).unwrap_or_default().filter(duel, CardId::is_faceup, None);
    if !c.is_relate_to_effect(duel, a.e) || tg.is_empty() {
        return Ok(());
    }
    let lv: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    let reset = RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_END;
    if tg.len() == 1 {
        if let Some(tc) = tg.get_first()
            && tc.get_level(duel) > lv
        {
            tc.update_level(duel, -(lv as i32), Some(reset), Some(c));
        }
    } else if tg.len() == 2 {
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 0));
        let rd = tg
            .filter_select(duel, tp, |c, duel| c.is_level_above(duel, lv + 1), 1, 1, None)?
            .get_first();
        if let Some(rd) = rd
            && rd.update_level(duel, -(lv as i32), Some(reset), Some(c)) != 0
        {
            let mut rest = tg;
            rest.remove(rd);
            if let Some(other) = rest.get_first() {
                other.update_level(duel, lv as i32, Some(reset), Some(c));
            }
        }
    }
    Ok(())
}
