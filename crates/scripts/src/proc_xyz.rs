// Port of proc_xyz.lua: Xyz.AddProcedure with the condition, target and operation of the Xyz Summon and of the
// alternative material (alterf). The materials effects make possible (EFFECT_XYZ_MATERIAL on the opponent's monsters,
// EFFECT_XYZ_MAT_FROM_GRAVE, EFFECT_SPELL_XYZ_MAT, EFFECT_ORICHALCUM_CHAIN, EFFECT_DOUBLE_XYZ_MATERIAL, equips with
// EFFECT_EQUIP_SPELL_XYZ_MAT) and the restrictions of EFFECT_XYZ_MAT_RESTRICTION, EFFECT_STAR_SERAPH_SOVEREIGNTY and
// EFFECT_SATELLARKNIGHT_CAPELLA come with the cards that register them; until then the material counts equal the
// selected cards. The metatable fields the procedure sets (xyz_filter, xyz_parameters, minxyzct, maxxyzct) are read
// only by cards that are not ported.
use engine::{Args, CardId, CardSet, Duel, EffectId, Script};

use crate::constant::*;
use crate::utility::aux;

pub struct Xyz;

// f(c, xyz, sumtype, tp) and alterf(c, tp, xyzc).
pub type XyzFilter = fn(CardId, &Duel, CardId, u64, u8) -> bool;
pub type AlterFilter = fn(CardId, &Duel, u8, CardId) -> bool;

// The parameters of Xyz.AddProcedure (f, lv, ct, alterf, desc, maxct, op, mustbemat, exchk): a nil filter, level or
// alternative material is None, a nil maxct is ct, and op and exchk left nil pass.
pub trait XyzProcedure {
    const F: Option<XyzFilter> = None;
    const LV: Option<u32>;
    const CT: usize;
    const MAXCT: usize = Self::CT;
    const ALTERF: Option<AlterFilter> = None;
    const DESC: u64 = 0;
    const MUSTBEMAT: bool = false;

    // op(e, tp, 0, mc).
    fn op_check(_duel: &Duel, _e: EffectId, _tp: u8, _mc: CardId) -> bool {
        true
    }

    // op(e, tp, 1, mc).
    fn op(_duel: &mut Duel, _e: EffectId, _tp: u8, _mc: CardId) -> Script<bool> {
        Ok(true)
    }

    fn exchk(_duel: &Duel, _matg: CardSet, _tp: u8, _xyz: CardId) -> bool {
        true
    }
}

const XYZ_MATERIAL: u64 = SUMMON_TYPE_XYZ as u64 | MATERIAL_XYZ;

// What the recursive checks share: the Xyz Monster and its controller, the materials it must use, and the forced
// minimum and maximum (0 for nil).
struct Search {
    xyz: CardId,
    tp: u8,
    mustg: CardSet,
    min: usize,
    max: usize,
}

impl Xyz {
    pub const INFINITE_MATS: usize = usize::MAX;

    pub fn add_procedure<P: XyzProcedure>(duel: &mut Duel, c: CardId) {
        let chk1 = duel.create_effect(c);
        chk1.set_type(duel, EFFECT_TYPE_SINGLE);
        chk1.set_property(
            duel,
            EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_SET_AVAILABLE,
        );
        chk1.set_code(duel, 946);
        chk1.set_condition(duel, Xyz::condition::<P>);
        chk1.set_target(duel, Xyz::target::<P>);
        chk1.set_operation(duel, Xyz::operation);
        c.register_effect(duel, chk1);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_description(duel, 1173);
        e1.set_code(duel, EFFECT_SPSUMMON_PROC);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e1.set_range(duel, LOCATION_EXTRA);
        e1.set_condition(duel, Xyz::condition::<P>);
        e1.set_target(duel, Xyz::target::<P>);
        e1.set_operation(duel, Xyz::operation);
        e1.set_value(duel, SUMMON_TYPE_XYZ as i32);
        e1.set_label_object(duel, Some(chk1));
        c.register_effect(duel, e1);
        if P::ALTERF.is_some() {
            let chk2 = chk1.clone_effect(duel);
            chk2.set_description(duel, P::DESC);
            chk2.set_condition(duel, Xyz::condition2::<P>);
            chk2.set_target(duel, Xyz::target2::<P>);
            chk2.set_operation(duel, Xyz::operation2);
            c.register_effect(duel, chk2);
            let e2 = e1.clone_effect(duel);
            e2.set_description(duel, P::DESC);
            e2.set_condition(duel, Xyz::condition2::<P>);
            e2.set_target(duel, Xyz::target2::<P>);
            e2.set_operation(duel, Xyz::operation2);
            c.register_effect(duel, e2);
        }
    }

    // Xyz.EffectXyzMaterialChk: whether an EFFECT_XYZ_MATERIAL effect lets the opponent's monster be a material.
    fn effect_xyz_material_chk(c: CardId, duel: &Duel) -> bool {
        unported!(duel, c.is_has_effect(duel, EFFECT_XYZ_MATERIAL).is_some());
        false
    }

    fn alter_filter<P: XyzProcedure>(c: CardId, duel: &Duel, alterf: AlterFilter, xyzc: CardId, e: EffectId, tp: u8) -> bool {
        if !alterf(c, duel, tp, xyzc)
            || !c.is_can_be_xyz_material(duel, Some(xyzc), tp, REASON_XYZ | REASON_MATERIAL)
            || (c.is_controler(duel, 1 - tp) && !Xyz::effect_xyz_material_chk(c, duel))
            || !P::op_check(duel, e, tp, c)
        {
            return false;
        }
        if xyzc.is_location(duel, LOCATION_EXTRA) {
            duel.get_location_count_from_ex(tp, tp, Some(c.into()), xyzc) > 0
        } else {
            duel.get_mzone_count(tp, Some(c.into())) > 0
        }
    }

    fn mat_filter2<P: XyzProcedure>(c: CardId, duel: &Duel, xyz: CardId, tp: u8) -> bool {
        P::F.is_none_or(|f| f(c, duel, xyz, XYZ_MATERIAL, tp))
            && P::LV.is_none_or(|lv| c.is_xyz_level(duel, xyz, lv))
            && c.is_can_be_xyz_material(duel, Some(xyz), tp, REASON_XYZ | REASON_MATERIAL)
    }

    fn get_materials(duel: &Duel, tp: u8) -> CardSet {
        let usable = |c: CardId, duel: &Duel| {
            if c.is_location(duel, LOCATION_GRAVE) {
                let from_grave = c.is_has_effect(duel, EFFECT_XYZ_MAT_FROM_GRAVE);
                unported!(duel, from_grave.is_some());
                if from_grave.is_none() {
                    return false;
                }
            }
            if c.is_location(duel, LOCATION_MZONE) && c.is_facedown(duel) {
                return false;
            }
            c.is_controler(duel, tp) || Xyz::effect_xyz_material_chk(c, duel)
        };
        duel.get_matching_group(usable, tp, LOCATION_MZONE | LOCATION_GRAVE, LOCATION_MZONE, None)
    }

    fn sub_mat_filter<P: XyzProcedure>(c: CardId, duel: &Duel) -> bool {
        if P::LV.is_none() {
            return false;
        }
        unported!(duel, c.is_has_effect(duel, EFFECT_SPELL_XYZ_MAT).is_some());
        false
    }

    // The materials the condition and the target start from: (g, mg). Equips are not ported, so no material brings
    // EFFECT_EQUIP_SPELL_XYZ_MAT equips along.
    fn materials<P: XyzProcedure>(duel: &Duel, c: CardId, tp: u8, og: Option<CardSet>) -> (CardSet, CardSet) {
        if let Some(og) = og {
            let mat_filter = |mc: CardId, duel: &Duel| {
                (mc.is_controler(duel, tp) || Xyz::effect_xyz_material_chk(mc, duel)) && Xyz::mat_filter2::<P>(mc, duel, c, tp)
            };
            return (og, og.filter(duel, mat_filter, None));
        }
        let g = Xyz::get_materials(duel, tp);
        let mut mg = g.filter(duel, |mc, duel| Xyz::mat_filter2::<P>(mc, duel, c, tp), None);
        if !P::MUSTBEMAT && P::F.is_none() {
            mg.merge(duel.get_matching_group(Xyz::sub_mat_filter::<P>, tp, LOCATION_ONFIELD + LOCATION_GRAVE, 0, None));
        }
        (g, mg)
    }

    fn orichalcum_group(duel: &Duel, tp: u8) -> CardSet {
        let g = duel.get_matching_group(
            |c, duel| c.is_has_effect(duel, EFFECT_ORICHALCUM_CHAIN).is_some(),
            tp,
            LOCATION_HAND + LOCATION_ONFIELD + LOCATION_GRAVE + LOCATION_REMOVED,
            0,
            None,
        );
        unported!(duel, !g.is_empty());
        g
    }

    // Xyz.MatNumChkF and Xyz.MatNumChkF2 find no effect to check.
    fn check_material_set<P: XyzProcedure>(duel: &Duel, matg: CardSet, s: &Search) -> bool {
        if !matg.includes(s.mustg) {
            return false;
        }
        let restricted = |code| matg.is_exists(duel, |c, duel| c.is_has_effect(duel, code).is_some(), 1, None);
        unported!(duel, restricted(EFFECT_STAR_SERAPH_SOVEREIGNTY));
        unported!(duel, P::LV.is_some() && restricted(EFFECT_SATELLARKNIGHT_CAPELLA));
        if !matg.is_empty() && !P::exchk(duel, matg, s.tp, s.xyz) {
            return false;
        }
        if s.xyz.is_location(duel, LOCATION_EXTRA) {
            return duel.get_location_count_from_ex(s.tp, s.tp, Some(matg), s.xyz) > 0;
        }
        duel.get_mzone_count(s.tp, Some(matg)) > 0
    }

    // Xyz.RecursionChk: whether `c` added to the selection sg (whose materials are matg) leads to a valid set.
    fn recursion_chk<P: XyzProcedure>(duel: &Duel, c: CardId, mg: CardSet, s: &Search, sg: CardSet, matg: CardSet, ct: usize) -> bool {
        unported!(duel, c.is_has_effect(duel, EFFECT_ORICHALCUM_CHAIN).is_some());
        let xct = ct + 1;
        if (s.max != 0 && xct > s.max) || (P::MAXCT != Xyz::INFINITE_MATS && xct > P::MAXCT) {
            return false;
        }
        let restricted = |c: CardId, duel: &Duel| c.is_has_effect(duel, EFFECT_XYZ_MAT_RESTRICTION).is_some();
        unported!(duel, restricted(c, duel) || sg.is_exists(duel, restricted, 1, None));
        let (matg, sg) = (matg.union(c.into()), sg.union(c.into()));
        (xct >= s.min && xct >= P::CT && Xyz::check_material_set::<P>(duel, matg, s))
            || mg.is_exists(
                duel,
                |tc, duel| Xyz::recursion_chk::<P>(duel, tc, mg, s, sg, matg, xct),
                1,
                Some(sg),
            )
            || (!P::MUSTBEMAT && Xyz::is_double_material(duel, c))
    }

    fn is_double_material(duel: &Duel, c: CardId) -> bool {
        unported!(duel, c.is_has_effect(duel, EFFECT_DOUBLE_XYZ_MATERIAL).is_some());
        false
    }

    fn condition<P: XyzProcedure>(duel: &Duel, a: &Args) -> bool {
        let Some(c) = a.c else { return true };
        if usize::from(a.min) > P::MAXCT || (c.is_type(duel, TYPE_PENDULUM) && c.is_faceup(duel)) {
            return false;
        }
        let tp = c.get_controler(duel);
        let (g, mut mg) = Xyz::materials::<P>(duel, c, tp, a.materials);
        let mut mustg = aux::get_must_be_material_group(duel, tp, Some(g), tp, Some(c), Some(mg), REASON_XYZ);
        if let Some(must) = a.must {
            mustg.merge(must);
        }
        if !mg.includes(mustg) {
            return false;
        }
        if !P::MUSTBEMAT {
            mg.merge(Xyz::orichalcum_group(duel, tp));
        }
        let s = Search {
            xyz: c,
            tp,
            mustg,
            min: usize::from(a.min),
            max: usize::from(a.max),
        };
        let none = CardSet::default();
        mg.is_exists(duel, |mc, duel| Xyz::recursion_chk::<P>(duel, mc, mg, &s, none, none, 0), 1, None)
    }

    fn target<P: XyzProcedure>(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
        let (e, tp, must, og) = (a.e, a.tp, a.must, a.materials);
        let Some(c) = a.c else { return Ok(false) };
        let (g, mut mg) = Xyz::materials::<P>(duel, c, tp, og);
        let mut mustg = aux::get_must_be_material_group(duel, tp, Some(g), tp, Some(c), Some(mg), REASON_XYZ);
        if let Some(must) = must {
            mustg.merge(must);
        }
        let orichalcum = (!P::MUSTBEMAT).then(|| Xyz::orichalcum_group(duel, tp));
        mg.merge(orichalcum.unwrap_or_default());
        let (min, max) = (usize::from(a.min), usize::from(a.max));
        if orichalcum.is_none_or(|g| g.is_empty()) && must.is_some_and(|must| min != 0 && must.len() == min && must.len() == max) {
            e.set_label_object_group(duel, Some(mustg));
            return Ok(true);
        }
        let s = Search {
            xyz: c,
            tp,
            mustg,
            min,
            max,
        };
        let (mut matg, mut sg) = (CardSet::default(), CardSet::default());
        let mut finishable = false;
        while (max == 0 || matg.len() < max) && (P::MAXCT == Xyz::INFINITE_MATS || matg.len() < P::MAXCT) {
            let ct = matg.len();
            let selg = mg.filter(duel, |mc, duel| Xyz::recursion_chk::<P>(duel, mc, mg, &s, sg, matg, ct), Some(sg));
            if selg.is_empty() {
                break;
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
            let cancelable = og.is_none() && duel.is_summon_cancelable() && sg.is_empty();
            let Some(sc) = selg.select_unselect(duel, Some(sg), tp, finishable, cancelable, 1, 1)? else {
                if !finishable {
                    return Ok(false);
                }
                break;
            };
            if !sg.is_contains(sc) {
                sg.insert(sc);
                unported!(duel, sc.is_has_effect(duel, EFFECT_DOUBLE_XYZ_MATERIAL).is_some());
                matg.insert(sc);
            } else {
                sg.remove(sc);
                matg.remove(sc);
            }
            finishable = matg.len() >= P::CT && Xyz::check_material_set::<P>(duel, matg, &s);
        }
        e.set_label_object_group(duel, Some(sg));
        Ok(true)
    }

    fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
        let Some(c) = a.c else { return Ok(()) };
        let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
        unported!(
            duel,
            g.is_exists(duel, |mc, duel| mc.is_has_effect(duel, EFFECT_ORICHALCUM_CHAIN).is_some(), 1, None)
        );
        c.set_material(duel, Some(g));
        duel.overlay_full(c, g, true)
    }

    fn condition2<P: XyzProcedure>(duel: &Duel, a: &Args) -> bool {
        let (e, og) = (a.e, a.materials);
        let (Some(c), Some(alterf)) = (a.c, P::ALTERF) else { return true };
        if c.is_type(duel, TYPE_PENDULUM) && c.is_faceup(duel) {
            return false;
        }
        let tp = c.get_controler(duel);
        let mg = og.unwrap_or_else(|| duel.get_field_group(tp, LOCATION_MZONE, LOCATION_MZONE));
        let mut mustg = aux::get_must_be_material_group(duel, tp, og, tp, Some(c), Some(mg), REASON_XYZ);
        if let Some(must) = a.must {
            mustg.merge(must);
        }
        if mustg.len() > 1 || a.min > 1 || !mg.includes(mustg) {
            return false;
        }
        let alter_filter = |mc, duel: &Duel| Xyz::alter_filter::<P>(mc, duel, alterf, c, e, tp);
        match mustg.get_first() {
            Some(mustc) => alter_filter(mustc, duel),
            None => mg.is_exists(duel, alter_filter, 1, None),
        }
    }

    // Xyz.ProcCancellable, which the target sets, is read only by the xyzop functions of cards that are not ported.
    fn target2<P: XyzProcedure>(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
        let (e, tp, must, og) = (a.e, a.tp, a.must, a.materials);
        let (Some(c), Some(alterf)) = (a.c, P::ALTERF) else {
            return Ok(false);
        };
        let cancelable = og.is_none() && duel.is_summon_cancelable();
        let mg = og.unwrap_or_else(|| duel.get_field_group(tp, LOCATION_MZONE, LOCATION_MZONE));
        let mut mustg = aux::get_must_be_material_group(duel, tp, og, tp, Some(c), Some(mg), REASON_XYZ);
        if let Some(must) = must {
            mustg.merge(must);
        }
        let (min, max) = (usize::from(a.min), usize::from(a.max));
        let oc = if must.is_some_and(|must| min != 0 && must.len() == min && must.len() == max) {
            mustg.get_first()
        } else if !mustg.is_empty() {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
            mustg.select_full(duel, tp, 1, 1, cancelable, None)?.get_first()
        } else {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
            let alter_filter = |mc, duel: &Duel| Xyz::alter_filter::<P>(mc, duel, alterf, c, e, tp);
            mg.filter_select_full(duel, tp, alter_filter, 1, 1, cancelable, None)?.get_first()
        };
        let Some(oc) = oc else { return Ok(false) };
        if !P::op(duel, e, tp, oc)? {
            return Ok(false);
        }
        e.set_label_object_group(duel, Some(oc.into()));
        Ok(true)
    }

    fn operation2(duel: &mut Duel, a: &Args) -> Script<()> {
        let Some(c) = a.c else { return Ok(()) };
        let oc = a.e.get_label_object_group(duel).unwrap_or_default();
        c.set_material(duel, Some(oc));
        duel.overlay(c, oc)
    }
}
