//! Configuration and reporting for the capacity policy and the collector.

/// The default of [`GcConfig::initial_capacity`].
const DEFAULT_INITIAL_CAPACITY: usize = 1024;
/// The default of [`GcConfig::minimum_free_fraction`].
const DEFAULT_MINIMUM_FREE_FRACTION: f64 = 0.1;
/// The default of [`GcConfig::growth_factor`].
const DEFAULT_GROWTH_FACTOR: usize = 2;

/// How aggressively a document grows its arena and collects garbage.
///
/// A document does not pay for its arena up front: the backing storage grows on demand, and the
/// *capacity* configured here is the number of slots the arena may hold before it considers itself
/// full. When an allocation finds the arena full, the document collects garbage and, if that did not
/// free enough room, grows the capacity. The numbers below are a deliberate trade-off rather than a
/// measured optimum, and they only need to be changed in unusual workloads:
///
/// * the **growth factor** of `2` is the usual amortized doubling: it keeps the number of
///   grow-and-reallocate steps logarithmic in the number of allocations, at the price of up to
///   twice the memory a perfectly sized arena would need;
/// * the **minimum free fraction** of `10%` exists to avoid a pathological loop in which a document
///   that is permanently almost full collects garbage before *every* allocation only to reuse a
///   handful of slots. Growing instead makes the following allocations cheap.
///
/// Use the `with_*` methods to change one value at a time; each of them validates its argument, so
/// an inconsistent configuration cannot be built.
///
/// # Examples
///
/// ```rust
/// use biodivine_lib_xml_dom::{Document, GcConfig};
///
/// // A document whose arena starts at eight slots and doubles from there.
/// let document = Document::with_gc_config(GcConfig::new().with_initial_capacity(8));
/// assert_eq!(document.capacity(), 8);
/// document.create_text("small").unwrap();
/// assert_eq!(document.node_count(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GcConfig {
    /// Number of slots the arena of a fresh document may hold before it considers itself full.
    initial_capacity: usize,
    /// Grow the capacity once a collection leaves less than this fraction of it free.
    minimum_free_fraction: f64,
    /// Multiplier applied to the capacity when it is grown.
    growth_factor: usize,
}

impl GcConfig {
    /// The configuration a document uses unless another one is given.
    pub const DEFAULT: Self = Self {
        initial_capacity: DEFAULT_INITIAL_CAPACITY,
        minimum_free_fraction: DEFAULT_MINIMUM_FREE_FRACTION,
        growth_factor: DEFAULT_GROWTH_FACTOR,
    };

    /// Creates the default configuration.
    pub const fn new() -> Self {
        Self::DEFAULT
    }

    /// Sets the initial capacity, in slots.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is zero, because an arena that may hold no slot at all could not
    /// allocate anything.
    #[must_use]
    pub fn with_initial_capacity(mut self, capacity: usize) -> Self {
        assert!(
            capacity > 0,
            "the initial capacity must be at least one slot"
        );
        self.initial_capacity = capacity;
        self
    }

    /// Sets the fraction of the capacity that must remain free after a collection, in `(0.0, 0.9]`.
    ///
    /// # Panics
    ///
    /// Panics if `fraction` is not in `(0.0, 0.9]`. Fractions above `0.9` would grow the arena for
    /// every collection even when it is mostly empty, and non-positive fractions would allow the
    /// collect-before-every-allocation loop this setting exists to prevent.
    #[must_use]
    pub fn with_minimum_free_fraction(mut self, fraction: f64) -> Self {
        assert!(
            fraction.is_finite() && fraction > 0.0 && fraction <= 0.9,
            "the minimum free fraction must be in (0.0, 0.9]"
        );
        self.minimum_free_fraction = fraction;
        self
    }

    /// Sets the factor by which the capacity grows, which has to be at least `2`.
    ///
    /// # Panics
    ///
    /// Panics if `factor` is smaller than `2`: a smaller factor would make almost no progress and
    /// the document would collect garbage before nearly every allocation.
    #[must_use]
    pub fn with_growth_factor(mut self, factor: usize) -> Self {
        assert!(
            factor >= 2,
            "the growth factor must be at least 2 so that the capacity grows geometrically"
        );
        self.growth_factor = factor;
        self
    }

    /// The number of slots a fresh document may hold before it considers itself full.
    pub const fn initial_capacity(&self) -> usize {
        self.initial_capacity
    }

    /// The fraction of the capacity that must remain free after a collection.
    pub const fn minimum_free_fraction(&self) -> f64 {
        self.minimum_free_fraction
    }

    /// The factor by which the capacity grows.
    pub const fn growth_factor(&self) -> usize {
        self.growth_factor
    }
}

impl Default for GcConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The free fraction, computed without an unchecked integer conversion.
///
/// Both arguments are slot counts and therefore bounded by `MAX_NODES` (`u32::MAX`), which `f64`
/// represents exactly.
pub(crate) fn free_fraction(free: usize, capacity: usize) -> f64 {
    let free = u32::try_from(free).expect("the free slot count is bounded by the arena capacity");
    let capacity = u32::try_from(capacity).expect("the arena capacity is bounded by MAX_NODES");
    if capacity == 0 {
        return 1.0;
    }
    f64::from(free) / f64::from(capacity)
}

/// What one run of the collector did.
///
/// Manual and automatic collections both report this, which makes the capacity policy observable: a
/// test or a profiling session can watch [`GcStats::capacity`] grow and the free fraction fall below
/// the configured threshold.
///
/// # Examples
///
/// ```rust
/// use biodivine_lib_xml_dom::Document;
///
/// let document = Document::empty();
/// let stats = document.collect_garbage();
/// assert_eq!(stats.collected, 0);
/// assert_eq!(stats.live, 0);
/// assert_eq!(stats.free, stats.capacity);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct GcStats {
    /// The capacity of the arena after the collection, in slots.
    pub capacity: usize,
    /// The number of slots that hold a node after the collection.
    pub live: usize,
    /// The number of slots that can be allocated without growing the arena.
    pub free: usize,
    /// The number of nodes that this collection reclaimed.
    pub collected: usize,
}

impl GcStats {
    /// The fraction of the capacity that is free, in `[0.0, 1.0]`.
    #[must_use]
    pub fn free_fraction(&self) -> f64 {
        free_fraction(self.free, self.capacity)
    }
}
