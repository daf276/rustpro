// Script entry points and coroutines: a prompting function bails, then reruns from a snapshot with the logged answers, muted.
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::buffer::ProgressiveBuffer;
use crate::card::{CardId, CardSet};
use crate::duel::Duel;
use crate::effect::{EffectId, Event};
use crate::list::List;
use crate::processor::{OcgStatus, Unit};

// A Duel action needed a choice the log does not hold, or the duel ended while the function ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interrupt;

pub type Script<T> = Result<T, Interrupt>;

// The parameters (e, tp, eg, ep, ev, re, r, rp) the core passes script functions, then a summon procedure's; 0 is nil.
#[derive(Debug, Clone, Copy, Default)]
pub struct Args {
    pub e: EffectId,
    pub tp: u8,
    pub eg: Option<CardSet>,
    pub ep: u8,
    pub ev: u32,
    pub re: Option<EffectId>,
    pub r: u32,
    pub rp: u8,
    pub c: Option<CardId>,
    // gc of EFFECT_FUSION_MATERIAL, else the materials a procedure must use.
    pub must: Option<CardSet>,
    // g of EFFECT_FUSION_MATERIAL, else the only materials a procedure may use.
    pub materials: Option<CardSet>,
    // minc of a Normal Summon procedure (its exeff is not ported).
    pub min: u16,
    pub max: u16,
    pub chkf: u64,
    pub zone: u32,
    pub releasable: u32,
}

impl Args {
    // A field effect's condition only receives the effect.
    pub(crate) fn effect(e: EffectId) -> Self {
        Self { e, ..Self::default() }
    }

    pub(crate) fn event(e: EffectId, tp: u8, event: &Event) -> Self {
        Self {
            e,
            tp,
            eg: event.event_cards,
            ep: event.event_player,
            ev: event.event_value,
            re: event.reason_effect,
            r: event.reason,
            rp: event.reason_player,
            ..Self::default()
        }
    }

    pub(crate) fn with_procedure(self, procedure: Procedure) -> Self {
        let Procedure {
            card,
            must,
            materials,
            min,
            max,
            chkf,
            zone,
            releasable,
        } = procedure;
        Self {
            c: Some(card),
            must,
            materials,
            min,
            max,
            chkf,
            zone,
            releasable,
            ..self
        }
    }
}

// The parameters after the card that the core passes a summon procedure's target and operation.
#[derive(Debug, Clone, Copy, Default)]
pub struct Procedure {
    pub card: CardId,
    pub must: Option<CardSet>,
    pub materials: Option<CardSet>,
    pub min: u16,
    pub max: u16,
    pub chkf: u64,
    pub zone: u32,
    pub releasable: u32,
}

pub type InitialEffect = fn(&mut Duel, CardId);

// A Fusion material check (tp, sg, fc) built by a closure over a card, kept with that card.
pub type FusionCheck = (fn(&Duel, CardId, u8, CardSet, CardId) -> bool, CardId);

// A Synchro material check (tp, sg, sc).
pub type SynchroCheck = fn(&Duel, u8, CardSet, CardId) -> bool;

// The scripts the oracle loads: utility.lua and the libraries it pulls in (their global effects), and c<code>.lua.
#[derive(Debug, Clone, Copy)]
pub struct Library {
    pub load: fn(&mut Duel),
    pub card_script: fn(u32) -> Option<InitialEffect>,
}

pub type Response = List<u8, 32>;

// The choice log of the outermost running function and what its attempts carry over.
#[derive(Debug, Clone, Default)]
pub struct Replay {
    pub log: Vec<Response>,
    pub cursor: usize,
    // The trace count when the last attempt started; the next attempt mutes what was written since, as it writes that again.
    pub messages: u64,
    // The duel before the pending function's first attempt; set while one of its prompts waits. Clones share it.
    pub snapshot: Option<Snapshot>,
    pub spare: Spare,
    // The prompt a nested loop met with no answer logged, carried out to the function's unit.
    pub prompt: Option<Box<Unit>>,
    // The unit stack of a yield's nested run, kept for its capacity.
    pub units: Vec<Unit>,
    pub interrupted: bool,
    // The running functions' effects and players, innermost last, restored as the reason after an action's units ran.
    pub stack: List<(EffectId, u8), 4>,
}

pub type Snapshot = Arc<Mutex<Duel>>;

// Panics only follow bugs, so a poisoned snapshot is used as it is.
pub(crate) fn lock(snapshot: &Snapshot) -> MutexGuard<'_, Duel> {
    snapshot.lock().unwrap_or_else(PoisonError::into_inner)
}

// The last finished function's snapshot, which the next one reuses unless a clone shares it; a clone starts without one.
#[derive(Debug, Default)]
pub struct Spare(Option<Snapshot>);

impl Clone for Spare {
    fn clone(&self) -> Self {
        Self(None)
    }
}

impl Duel {
    // A cost or target with chk=0: a check, which cannot reach a prompt.
    pub(crate) fn check_script(&mut self, call: impl FnOnce(&mut Self) -> Script<bool>) -> bool {
        call(self).unwrap_or(false)
    }

    // The body of the Execute units; an outermost call snapshots first unless `promptless`. None if a prompt waits or the duel ended.
    pub(crate) fn run_script<T>(
        &mut self,
        effect: EffectId,
        player: u8,
        promptless: bool,
        call: impl Fn(&mut Self) -> Script<T>,
    ) -> Option<T> {
        let nested = !self.core.replay.stack.is_empty();
        if nested || promptless {
            self.core.replay.stack.push((effect, player));
            let result = call(self);
            self.core.replay.stack.pop();
            if !nested {
                self.assumes.get_mut().clear();
            }
            if result.is_err() && nested {
                self.core.replay.interrupted = true;
            } else if result.is_err() && self.result.is_none() {
                self.reach_unported(concat!(file!(), ":", line!(), " (a promptless function prompted)"));
            }
            return result.ok();
        }
        let (snapshot, mute) = match self.core.replay.snapshot.take() {
            Some(snapshot) => {
                // No answer is empty; an empty one reruns to the prompt that waits (Duel::rerun_to_prompt).
                if !self.returns.data.is_empty() {
                    let answer = self.returns.data.clone();
                    self.core.replay.log.push(answer);
                }
                self.copy_from(&mut lock(&snapshot));
                (snapshot, self.trace.count.get() - self.core.replay.messages)
            }
            None => (self.snapshot(), 0),
        };
        self.core.replay.cursor = 0;
        self.trace.mute.set(mute);
        let start = self.trace.count.get();
        self.core.replay.stack.push((effect, player));
        let result = call(self);
        self.core.replay.stack.pop();
        self.core.replay.interrupted = false;
        match result {
            Ok(value) => {
                // duel::restore_assumes, which the core runs once the outermost Lua call returns.
                self.assumes.get_mut().clear();
                debug_assert_eq!(self.trace.mute.get(), 0, "a rerun wrote fewer messages than its previous attempt");
                self.core.replay.log.clear();
                self.core.replay.messages = 0;
                self.core.replay.spare = Spare(Some(snapshot));
                Some(value)
            }
            Err(Interrupt) => {
                if self.result.is_none() {
                    self.core.replay.messages = start;
                    self.core.replay.snapshot = Some(snapshot);
                }
                None
            }
        }
    }

    // A check whose Lua registers and resets effects, run on a copy; brings back what later lookups tell apart (ids, resets, messages).
    pub fn check_mut<T>(&self, check: impl FnOnce(&mut Duel) -> T) -> T {
        let mut copy = self.clone();
        copy.reset_pending();
        let result = check(&mut copy);
        self.trace.buffer.swap(&copy.trace.buffer);
        self.trace.count.swap(&copy.trace.count);
        self.trace.mute.swap(&copy.trace.mute);
        self.script_state.swap(&copy.script_state);
        debug_assert!(
            copy.effects.added_since(&self.effects).next().is_none(),
            "a check kept an effect it registered"
        );
        self.infos.field_id.set(copy.infos.field_id.get());
        self.effects.copy_cells_from(&copy.effects);
        let mut pending = self.pending_resets.borrow_mut();
        for effect in self.effects.added_since(&copy.effects) {
            if !pending.contains(&effect) {
                pending.push(effect);
            }
        }
        if self.unported.get().is_none() {
            self.unported.set(copy.unported.get());
        }
        result
    }

    pub(crate) fn reset_pending(&mut self) {
        let trace = std::mem::take(&mut self.trace);
        for effect in std::mem::take(self.pending_resets.get_mut()) {
            if let Some(handler) = self.effects[effect].handler {
                self.card_remove_effect(handler, effect);
            }
        }
        self.trace = trace;
    }

    // A copy without the trace and the replay state, which live on across attempts.
    fn snapshot(&mut self) -> Snapshot {
        if let Some(mut spare) = self.core.replay.spare.0.take()
            && let Some(spare_duel) = Arc::get_mut(&mut spare)
        {
            spare_duel.get_mut().unwrap_or_else(PoisonError::into_inner).copy_from(self);
            return spare;
        }
        let trace = std::mem::take(&mut self.trace);
        let replay = std::mem::take(&mut self.core.replay);
        let snapshot = Arc::new(Mutex::new(self.clone()));
        self.trace = trace;
        self.core.replay = replay;
        snapshot
    }

    // Equals `source` but for the trace and replay state; one must be a snapshot of the other, as sync_from copies only changes.
    fn copy_from(&mut self, source: &mut Duel) {
        // Destructures every field, so that a new one fails to compile until it is listed.
        macro_rules! copy_fields {
            ($target:ident <- $source:ident; sync: $($sync:ident),*; clone: $($clone:ident),*; rest: $($rest:tt)*) => {
                let Duel { $($sync,)* $($clone,)* $($rest)* } = $source;
                $($target.$sync.sync_from($sync);)*
                $($target.$clone.clone_from($clone);)*
            };
        }
        copy_fields!(self <- source;
            sync: cards, effects, field_effects;
            clone: player, infos, relations, returns, return_cards, nil_event, random, result, library, database, script_state,
                script_groups, values_reset, fusion_check_additional, synchro_check_additional, ritual_summoning_level, assumes,
                pending_resets, material_cards, overlay_cards, counters, card_targets, card_relations, announced_cards,
                indestructable_effects, battled_cards, immune_effects, equip_cards, unported;
            rest: core, trace: _, destination_probe: _);
        let (replay, own) = (std::mem::take(&mut core.replay), std::mem::take(&mut self.core.replay));
        self.core.clone_from(core);
        (core.replay, self.core.replay) = (replay, own);
    }

    // Between a coroutine's yield and resume: the emplaced units run, a prompt among them answered from the log or interrupting.
    pub(crate) fn yield_script(&mut self) -> Script<()> {
        self.returns.set_i32(0, 0);
        let mut units = std::mem::take(&mut self.core.replay.units);
        let result = self.run_yield(&mut units);
        units.clear();
        self.core.replay.units = units;
        result?;
        if let Some(&(effect, player)) = self.core.replay.stack.last() {
            self.core.reason_effect.set(Some(effect));
            self.core.reason_player.set(player);
        }
        Ok(())
    }

    fn run_yield(&mut self, units: &mut Vec<Unit>) -> Script<()> {
        loop {
            if self.result.is_some() {
                return Err(Interrupt);
            }
            match self.process_unit(units) {
                OcgStatus::End => return Ok(()),
                OcgStatus::Continue => {}
                OcgStatus::Awaiting => {
                    let replay = &mut self.core.replay;
                    match replay.log.get(replay.cursor) {
                        Some(answer) => {
                            self.returns = ProgressiveBuffer { data: answer.clone() };
                            self.core.replay.cursor += 1;
                        }
                        None => {
                            self.core.replay.prompt = units.pop().map(Box::new);
                            self.core.replay.interrupted = true;
                        }
                    }
                }
            }
            if self.core.replay.interrupted {
                return Err(Interrupt);
            }
        }
    }
}
