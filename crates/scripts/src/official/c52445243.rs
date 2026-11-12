// Tri-Edge Master
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 52445243;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Water Spirit (Level 1 Tuner) and Shapesnatch (Level 5).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (487395, MZONE),
    (487395, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        a.e.get_label(duel) > 0 && a.e.get_handler(duel).is_synchro_summoned(duel)
    });
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE);
    e1a.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1a.set_label_object(duel, Some(e1));
    e1a.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e1a);
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let lb = e.get_label(duel);
    let g = (lb & 0x1 > 0)
        .then(|| duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(e.get_handler(duel).into())));
    let dr = lb & 0x2 > 0;
    if chk == 0 {
        return Ok(g.is_none_or(|g| !g.is_empty()) && (!dr || duel.is_player_can_draw(tp, 0)));
    }
    if let Some(g) = g {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    }
    if dr {
        if lb == 0x2 {
            duel.set_target_player(tp);
            duel.set_target_param(1);
        }
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    }
    let index = match lb {
        0x1 => 1,
        0x2 => 2,
        0x4 => 3,
        _ => 4,
    };
    duel.hint(HINT_OPSELECTED, 1 - tp, aux::stringid(ID, index));
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let lb = e.get_label(duel);
    let mut breakeff = false;
    if lb & 0x1 > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?;
        if !g.is_empty() {
            breakeff = true;
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    if lb & 0x2 > 0 {
        let (p, d) = if lb == 0x2 {
            (
                duel.get_chain_info(0, CHAININFO_TARGET_PLAYER),
                duel.get_chain_info(0, CHAININFO_TARGET_PARAM),
            )
        } else {
            (tp, 1)
        };
        if duel.is_player_can_draw(p, 0) {
            if breakeff {
                duel.break_effect()?;
            }
            breakeff = true;
            duel.draw(p, d, REASON_EFFECT)?;
        }
    }
    if lb & 0x4 > 0 && c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        if breakeff {
            duel.break_effect()?;
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
        e1.set_code(duel, EFFECT_ADD_TYPE);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_value(duel, TYPE_TUNER as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn matcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let g = c.get_material(duel);
    let Some(e1) = e.get_label_object(duel) else { return };
    e1.set_label(duel, 0);
    if g.len() > 2 {
        e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DRAW);
        e1.set_label(duel, 0x1 | 0x2 | 0x4);
    } else if g.len() == 2 {
        let mut levels = g.into_iter().map(|mc| mc.get_synchro_level(duel, c));
        let (Some(a), Some(b)) = (levels.next(), levels.next()) else {
            return;
        };
        if a + b != 6 {
            return;
        }
        let lv = a.min(b);
        if lv == 1 {
            e1.set_category(duel, CATEGORY_DESTROY);
        } else if lv == 2 {
            e1.set_category(duel, CATEGORY_DRAW);
        }
        // Lua's 1<<(lv-1) is 0 for Level 0.
        e1.set_label(duel, (1_i64 << lv) >> 1);
    }
}
