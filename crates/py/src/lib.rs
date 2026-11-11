#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use carddata::{CardDatabase, Deck as Decklist};
use engine::{DUEL_MODE_MR5, DuelRecord, PlayerOptions, Position, PositionError, Prompt, Xoshiro256StarStar};
use fiber::Fiber;
use numpy::{PyArray1, PyArray2, PyArrayDyn, PyArrayMethods, PyReadonlyArray1};
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use scripts::LIBRARY;
use sim::{
    ACTION_FEATURES, ACTIONS, AutoAdvance, CARD_FEATURES, CHAIN_FEATURES, GLOBAL_FEATURES, MAX_CHAIN, MAX_OBS_CARDS, Options, PROMPTS,
    SHAPES, Section, find, indices,
};

create_exception!(ygo_sim, Error, PyException);

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn fail(error: impl std::fmt::Display) -> PyErr {
    Error::new_err(error.to_string())
}

// The value behind a Mutex that only makes a pyclass Sync; a poisoned one is still whole, as nothing panics while holding it.
fn exclusive<T>(mutex: &mut Mutex<T>) -> &mut T {
    mutex.get_mut().unwrap_or_else(PoisonError::into_inner)
}

fn same_length(values: usize, slots: usize) -> PyResult<()> {
    match values == slots {
        true => Ok(()),
        false => Err(fail(format!("{values} values for {slots} slots"))),
    }
}

/// The card database, leaked as duels hold it for good: load one per process.
#[pyclass(frozen)]
struct Database(&'static CardDatabase);

#[pymethods]
impl Database {
    #[new]
    #[pyo3(signature = (path = None))]
    fn new(path: Option<PathBuf>) -> PyResult<Self> {
        let path = path.unwrap_or_else(|| carddata::vendor().join("BabelCDB"));
        Ok(Self(Box::leak(Box::new(CardDatabase::load(&path).map_err(fail)?))))
    }

    /// A .ydk, or else the repository's format of `main` and `extra` lines.
    fn load_deck(&self, path: PathBuf) -> PyResult<Deck> {
        let text = std::fs::read_to_string(&path).map_err(fail)?;
        Ok(Deck(match path.extension().is_some_and(|extension| extension == "ydk") {
            true => Decklist::parse_ydk(&text, self.0),
            false => Decklist::parse(&text, self.0),
        }))
    }
}

#[pyclass(frozen, from_py_object)]
#[derive(Clone)]
struct Deck(Decklist);

#[pymethods]
impl Deck {
    #[new]
    fn new(main: Vec<u32>, extra: Vec<u32>) -> Self {
        Self(Decklist { main, extra })
    }

    #[getter]
    fn main(&self) -> Vec<u32> {
        self.0.main.clone()
    }

    #[getter]
    fn extra(&self) -> Vec<u32> {
        self.0.extra.clone()
    }
}

// Duel `number` of a deck pair, both decks shuffled, set up as the diff harness does.
fn setup(decks: &[Decklist; 2], number: u64) -> DuelRecord {
    let mut rng = StdRng::seed_from_u64(number);
    let seed = rng.random();
    let mut position = Position::new(DUEL_MODE_MR5, [PlayerOptions::default(); 2]);
    for (team, deck) in (0..).zip(decks) {
        position.add_deck(team, &deck.shuffled(&mut rng));
    }
    DuelRecord {
        seed,
        position,
        actions: Vec::new(),
    }
}

/// Int32 tensors of `size` rows: global_ [GLOBAL], cards [MAX_OBS_CARDS, CARD], chain [MAX_CHAIN, CHAIN], actions [ACTIONS, ACTION].
#[pyclass(frozen)]
struct Tensors {
    #[pyo3(get, name = "global_")]
    global: Py<PyArrayDyn<i32>>,
    #[pyo3(get)]
    cards: Py<PyArrayDyn<i32>>,
    #[pyo3(get)]
    chain: Py<PyArrayDyn<i32>>,
    #[pyo3(get)]
    actions: Py<PyArrayDyn<i32>>,
}

#[pymethods]
impl Tensors {
    #[new]
    fn new(py: Python<'_>, size: usize) -> Self {
        let [global, cards, chain, actions] = SHAPES.map(|shape| PyArrayDyn::zeros(py, [&[size], shape].concat(), false).unbind());
        Self {
            global,
            cards,
            chain,
            actions,
        }
    }
}

impl Tensors {
    fn write<R>(&self, py: Python<'_>, write: impl FnOnce(&mut sim::Tensors) -> PyResult<R>) -> PyResult<R> {
        let mut global = self.global.bind(py).try_readwrite()?;
        let mut cards = self.cards.bind(py).try_readwrite()?;
        let mut chain = self.chain.bind(py).try_readwrite()?;
        let mut actions = self.actions.bind(py).try_readwrite()?;
        let tensors = sim::Tensors::new(
            global.as_slice_mut().map_err(fail)?,
            cards.as_slice_mut().map_err(fail)?,
            chain.as_slice_mut().map_err(fail)?,
            actions.as_slice_mut().map_err(fail)?,
        );
        write(&mut tensors.map_err(fail)?)
    }
}

/// One duel, for search: `record` holds how it came about unless it was taken from a batch or determinized.
#[pyclass]
struct Duel {
    duel: Mutex<Fiber>,
    record: Option<DuelRecord>,
}

impl Duel {
    fn replay(database: &Database, record: DuelRecord) -> PyResult<Self> {
        Ok(Self {
            duel: Mutex::new(Fiber::new(record.replay(database.0, &LIBRARY).map_err(fail)?)),
            record: Some(record),
        })
    }

    fn duel(&mut self) -> &mut Fiber {
        exclusive(&mut self.duel)
    }
}

#[pymethods]
impl Duel {
    #[new]
    fn new(database: &Database, decks: (Deck, Deck), seed: u64) -> PyResult<Self> {
        Self::replay(database, setup(&[decks.0.0, decks.1.0], seed))
    }

    #[staticmethod]
    fn from_record(database: &Database, record: &[u8]) -> PyResult<Self> {
        Self::replay(database, DuelRecord::from_bytes(record).map_err(fail)?)
    }

    #[staticmethod]
    fn from_position(database: &Database, position: &[u8], seed: u64) -> PyResult<Self> {
        let position = Position::from_bytes(position).map_err(fail)?;
        let seed = StdRng::seed_from_u64(seed).random();
        let actions = Vec::new();
        Self::replay(database, DuelRecord { seed, position, actions })
    }

    fn clone(&mut self) -> Self {
        Self {
            duel: Mutex::new(self.duel().clone()),
            record: self.record.clone(),
        }
    }

    /// The player the prompt waits for, None once the duel ended.
    #[getter]
    fn acting(&mut self) -> Option<u8> {
        self.duel().prompt().map(Prompt::player)
    }

    /// The winner (2 for a draw) and the reason.
    #[getter]
    fn result(&mut self) -> Option<(u8, u8)> {
        self.duel().result()
    }

    fn legal_actions<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<i64>>> {
        let legal = self.duel().policy_actions();
        let indices: Result<Vec<i64>, _> = indices(&legal).map(|index| index.map(|index| index as i64)).collect();
        Ok(PyArray1::from_vec(py, indices.map_err(fail)?))
    }

    fn apply(&mut self, index: usize) -> PyResult<()> {
        let duel = self.duel();
        let action = find(&duel.policy_actions(), index).ok_or_else(|| fail(format!("index {index} is not a legal action")))?;
        duel.apply(action).map_err(fail)?;
        self.record.iter_mut().for_each(|record| record.actions.push(action));
        Ok(())
    }

    /// Writes what `player` sees into row `row` of `out`, a new Tensors of one row if None.
    #[pyo3(signature = (player, out = None, row = 0))]
    fn observe(&mut self, py: Python<'_>, player: u8, out: Option<Py<Tensors>>, row: usize) -> PyResult<Py<Tensors>> {
        let out = out.map_or_else(|| Py::new(py, Tensors::new(py, 1)), Ok)?;
        let duel = self.duel();
        out.get().write(py, |tensors| tensors.write_duel(row, duel, player).map_err(fail))?;
        Ok(out)
    }

    /// A duel agreeing with what `player` sees, the hidden cards drawn from `pool` (the opponent's deck, as known or assumed).
    fn determinize(&mut self, player: u8, seed: u64, pool: Deck) -> PyResult<Self> {
        let mut rng = Xoshiro256StarStar::new(StdRng::seed_from_u64(seed).random());
        Ok(Self {
            duel: Mutex::new(Fiber::new(self.duel().determinize(player, &mut rng, &pool.0).map_err(fail)?)),
            record: None,
        })
    }

    fn position<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        Ok(PyBytes::new(py, &self.duel().position().to_bytes().map_err(fail)?))
    }

    fn record<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyBytes>>> {
        let bytes = self.record.as_ref().map(DuelRecord::to_bytes).transpose().map_err(fail)?;
        Ok(bytes.map(|bytes| PyBytes::new(py, &bytes)))
    }
}

#[pyclass(frozen, get_all, eq)]
#[derive(PartialEq)]
struct Outcome {
    slot: usize,
    episode: u64,
    /// The winner (2 for a draw) and the reason, None for a duel cut off at max_steps.
    result: Option<(u8, u8)>,
    steps: u64,
    skipped: u64,
}

type Generator = Box<dyn Fn(usize, u64) -> Result<Option<engine::Duel>, PositionError> + Send + Sync>;

#[derive(Clone)]
struct Settings {
    database: &'static CardDatabase,
    decks: Arc<[[Decklist; 2]]>,
    size: usize,
    seed: u64,
    duels: Option<u64>,
    options: Options,
}

impl Settings {
    // Duel n = episode * size + slot of a run plays the slot's deck pair, seeded by seed + n.
    fn start(&self) -> PyResult<sim::Batch<Generator>> {
        let settings = self.clone();
        let generator = move |slot: usize, episode: u64| {
            let number = episode * settings.size as u64 + slot as u64;
            let decks = settings.decks.get(slot % settings.decks.len());
            let Some(decks) = decks.filter(|_| settings.duels.is_none_or(|duels| number < duels)) else {
                return Ok(None);
            };
            let record = setup(decks, settings.seed.wrapping_add(number));
            engine::Duel::from_position(&record.position, settings.database, &LIBRARY, record.seed).map(Some)
        };
        sim::Batch::new(self.size, self.options, Box::new(generator) as Generator).map_err(fail)
    }
}

/// Duels stepped in lockstep on rayon's pool (RAYON_NUM_THREADS); with `duels`, slots retire once that many have started.
#[pyclass]
struct Batch {
    batch: Mutex<sim::Batch<Generator>>,
    settings: Settings,
    actions: Vec<usize>,
}

#[pymethods]
impl Batch {
    /// `decks` holds one deck pair for every slot or one per slot; `auto_advance` is "off", "own" or "both".
    #[new]
    #[pyo3(signature = (database, decks, size, seed = 0, auto_advance = "off", max_steps = None, duels = None))]
    fn new(
        database: &Database,
        decks: Vec<(Deck, Deck)>,
        size: usize,
        seed: u64,
        auto_advance: &str,
        max_steps: Option<u64>,
        duels: Option<u64>,
    ) -> PyResult<Self> {
        if decks.len() != 1 && decks.len() != size {
            return Err(fail(format!("{} deck pairs for {size} slots", decks.len())));
        }
        let auto_advance = match auto_advance {
            "off" => AutoAdvance::Off,
            "own" => AutoAdvance::Own,
            "both" => AutoAdvance::Both,
            _ => return Err(fail(format!("auto_advance {auto_advance:?} is not off, own or both"))),
        };
        let settings = Settings {
            database: database.0,
            decks: decks.into_iter().map(|(first, second)| [first.0, second.0]).collect(),
            size,
            seed,
            duels,
            options: Options {
                auto_advance,
                max_steps: max_steps.unwrap_or(u64::MAX),
            },
        };
        Ok(Self {
            batch: Mutex::new(settings.start()?),
            settings,
            actions: vec![0; size],
        })
    }

    /// Starts over from the first duels.
    fn reset(&mut self) -> PyResult<()> {
        *exclusive(&mut self.batch) = self.settings.start()?;
        Ok(())
    }

    fn __len__(&self) -> usize {
        self.settings.size
    }

    /// An action index per slot (retired slots' are ignored); returns the outcomes of the duels that ended.
    fn step(&mut self, py: Python<'_>, actions: PyReadonlyArray1<i64>) -> PyResult<Vec<Outcome>> {
        let given = actions.as_slice().map_err(fail)?;
        same_length(given.len(), self.actions.len())?;
        for (action, &index) in self.actions.iter_mut().zip(given) {
            *action = usize::try_from(index).map_err(fail)?;
        }
        let Self { batch, actions, .. } = self;
        let batch = exclusive(batch);
        let ended = py.detach(|| Ok::<_, sim::BatchError>(batch.step(actions)?.iter().flatten().copied().collect::<Vec<_>>()));
        Ok(ended.map_err(fail)?.into_iter().map(Outcome::from).collect())
    }

    /// The legal actions of each slot as a bool [size, ACTIONS], written into `out` if given.
    #[pyo3(signature = (out = None))]
    fn masks<'py>(&mut self, py: Python<'py>, out: Option<Bound<'py, PyArray2<bool>>>) -> PyResult<Bound<'py, PyArray2<bool>>> {
        let out = out.unwrap_or_else(|| PyArray2::zeros(py, [self.settings.size, ACTIONS], false));
        let masks = exclusive(&mut self.batch).masks();
        let mut array = out.try_readwrite()?;
        let slice = array.as_slice_mut().map_err(fail)?;
        same_length(masks.len(), slice.len())?;
        slice.copy_from_slice(masks);
        drop(array);
        Ok(out)
    }

    /// The player each slot's prompt waits for as an int32 [size], -1 for a retired slot.
    #[pyo3(signature = (out = None))]
    fn acting<'py>(&mut self, py: Python<'py>, out: Option<Bound<'py, PyArray1<i32>>>) -> PyResult<Bound<'py, PyArray1<i32>>> {
        let out = out.unwrap_or_else(|| PyArray1::zeros(py, self.settings.size, false));
        let batch = exclusive(&mut self.batch);
        let mut array = out.try_readwrite()?;
        let slice = array.as_slice_mut().map_err(fail)?;
        same_length(slice.len(), batch.len())?;
        for (slot, acting) in slice.iter_mut().enumerate() {
            *acting = batch
                .duel(slot)
                .and_then(engine::Duel::prompt)
                .map_or(-1, |prompt| prompt.player().into());
        }
        drop(array);
        Ok(out)
    }

    /// What each slot's acting player sees, written into `out` if given.
    #[pyo3(signature = (out = None))]
    fn observe(&mut self, py: Python<'_>, out: Option<Py<Tensors>>) -> PyResult<Py<Tensors>> {
        let out = out.map_or_else(|| Py::new(py, Tensors::new(py, self.settings.size)), Ok)?;
        let batch = exclusive(&mut self.batch);
        out.get().write(py, |tensors| py.detach(|| batch.observe(tensors)).map_err(fail))?;
        Ok(out)
    }

    /// A copy of the slot's duel, None for a retired slot.
    fn duel(&mut self, slot: usize) -> Option<Duel> {
        let duel = exclusive(&mut self.batch).duel(slot)?.clone();
        Some(Duel {
            duel: Mutex::new(Fiber::new(duel)),
            record: None,
        })
    }
}

impl From<sim::Outcome> for Outcome {
    fn from(outcome: sim::Outcome) -> Self {
        let sim::Outcome {
            slot,
            episode,
            result,
            steps,
            skipped,
        } = outcome;
        Self {
            slot,
            episode,
            result,
            steps,
            skipped,
        }
    }
}

#[pymodule]
fn ygo_sim(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Database>()?;
    module.add_class::<Deck>()?;
    module.add_class::<Duel>()?;
    module.add_class::<Batch>()?;
    module.add_class::<Tensors>()?;
    module.add_class::<Outcome>()?;
    module.add("Error", module.py().get_type::<Error>())?;
    module.add("ACTIONS", ACTIONS)?;
    module.add("MAX_OBS_CARDS", MAX_OBS_CARDS)?;
    module.add("MAX_CHAIN", MAX_CHAIN)?;
    module.add("GLOBAL_FEATURES", GLOBAL_FEATURES)?;
    module.add("CARD_FEATURES", CARD_FEATURES)?;
    module.add("CHAIN_FEATURES", CHAIN_FEATURES)?;
    module.add("ACTION_FEATURES", ACTION_FEATURES)?;
    module.add("PROMPTS", PROMPTS)?;
    // Per section of the action indices its name, first index and number of slots.
    let sections = Section::ALL.map(|section| (format!("{section:?}"), section.start(), section.slots()));
    module.add("SECTIONS", sections)
}
