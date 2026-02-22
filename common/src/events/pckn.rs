#![deny(missing_docs)]
#![allow(clippy::cast_possible_wrap)]

use Match::*;

/// A Port, Channel, Key, NoteId (PCKN) tuple.
///
/// CLAP addresses notes and voices use this 4-value tuple: `port`, `channel`, `key` and `note_id`.
/// Each of the components in this PCKN can either be a specific value, or a wildcard that matches
/// any value in that part of the tuple. This is representing using the [`Match`] enum.
///
/// For instance, a [`Pckn`] of `(0, 3, All, All)` will match all voices
/// on channel 3 of port 0. And a [`Pckn`] of `(All, 0, 60, All)` will match
/// all channel 0 key 60 voices, independent of port or note id.
///
/// See the [`matches`](Pckn::matches) for an implementation of the PCKN matching logic that you
/// can use to match incoming events against active voices.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Pckn {
    /// The Note Port the plugin received this event on. See the Note Ports extension.
    pub port_index: Match<u16>,
    /// The Channel the note is on, akin to MIDI1 channels. This is usually in the `0..=15` range.
    pub channel: Match<u16>,
    /// The note's Key. This is the same representation as MIDI1 Key numbers,
    /// `60` being a Middle C, and is in the `0..=127` range.
    pub key: Match<u16>,
    /// The unique ID of this note. This is used to distinguish between multiple overlapping
    /// notes that play the same key. This is in the `0..i32::MAX` range.
    pub note_id: Match<u32>,
}

impl Pckn {
    /// Constructs a new PCKN tuple from each of its components.
    pub fn new(
        port: impl Into<Match<u16>>,
        channel: impl Into<Match<u16>>,
        key: impl Into<Match<u16>>,
        note_id: impl Into<Match<u32>>,
    ) -> Self {
        Self {
            port_index: port.into(),
            channel: channel.into(),
            key: key.into(),
            note_id: note_id.into(),
        }
    }

    /// Returns a [`Pckn`] tuple that matches *all* events, i.e. all of its components are set to
    /// [`Match::All`].
    #[inline]
    pub const fn match_all() -> Self {
        Self {
            port_index: All,
            channel: All,
            key: All,
            note_id: All,
        }
    }

    /// Returns whether this [`Pckn`] tuple matches all possible notes.
    ///
    /// This is true if all four matchers are set to [`Match::All`].
    pub const fn matches_all(&self) -> bool {
        self.port_index.is_all()
            && self.channel.is_all()
            && self.key.is_all()
            && self.note_id.is_all()
    }

    /// Returns `true` if this PCKN tuple matches the given one, considering both specific values
    /// and wildcard [`Match::All`] values.
    ///
    /// # Examples
    ///
    /// ```
    /// use clack_common::events::{Match, Pckn};
    ///
    /// assert!(Pckn::new(0u16, 0u16, 60u16, 42u32).matches(&Pckn::new(0u16, 0u16, 60u16, Match::All)));
    /// ```
    pub fn matches(&self, other: &Pckn) -> bool {
        self.port_index.matches(other.port_index)
            && self.channel.matches(other.channel)
            && self.key.matches(other.key)
            && self.note_id.matches(other.note_id)
    }

    // Raw accessors

    /// Constructs a new PCKN tuple from its raw, C-FFI compatible components.
    ///
    /// Components set to any negative value (usually `-1`) are interpreted as [`Match::All`], while
    /// any other value is interpreted as [`Match::Specific`].
    #[inline]
    pub const fn from_raw(port: i16, channel: i16, key: i16, note_id: i32) -> Self {
        Self {
            port_index: Match::<u16>::from_raw(port),
            channel: Match::<u16>::from_raw(channel),
            key: Match::<u16>::from_raw(key),
            note_id: Match::<u32>::from_raw(note_id),
        }
    }

    /// Returns the raw, C-FFI compatible Port component of this PCKN.
    ///
    /// This returns `-1` if the port is set to [`Match::All`], otherwise the specific value is
    /// returned.
    #[inline]
    pub const fn raw_port_index(&self) -> i16 {
        match self.port_index {
            Specific(p) => p as i16,
            All => -1,
        }
    }

    /// Returns the raw, C-FFI compatible Channel component of this PCKN.
    ///
    /// This returns `-1` if the Channel is set to [`Match::All`], otherwise the specific value is
    /// returned.
    #[inline]
    pub const fn raw_channel(&self) -> i16 {
        match self.channel {
            Specific(p) => p as i16,
            All => -1,
        }
    }

    /// Returns the raw, C-FFI compatible Key component of this PCKN.
    ///
    /// This returns `-1` if the Key is set to [`Match::All`], otherwise the specific value is
    /// returned.
    #[inline]
    pub const fn raw_key(&self) -> i16 {
        match self.key {
            Specific(p) => p as i16,
            All => -1,
        }
    }

    /// Returns the raw, C-FFI compatible Note ID component of this PCKN.
    ///
    /// This returns `-1` if the Note ID is set to [`Match::All`], otherwise the specific value is
    /// returned.
    #[inline]
    pub const fn raw_note_id(&self) -> i32 {
        match self.note_id {
            Specific(p) => p as i32,
            All => -1,
        }
    }
}

/// Represents matching either a specific value or all values of a given type.
///
/// This is used in the [`Pckn`] type to support matching multiple kinds of notes at once.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Match<T> {
    /// Matches a specific value.
    Specific(T),
    /// Matches all values.
    All,
}

impl<T> Match<T> {
    /// Returns a reference to the specific value of this matcher, or [`None`] if this is [`Match::All`].
    ///
    /// # Example
    ///
    /// ```
    /// use clack_common::events::Match;
    ///
    /// assert_eq!(Match::Specific(42).as_specific(), Some(&42));
    /// assert_eq!(Match::<u16>::All.as_specific(), None);
    /// ```
    pub const fn as_specific(&self) -> Option<&T> {
        match self {
            Specific(v) => Some(v),
            All => None,
        }
    }

    /// Returns a reference to the specific value of this matcher, or [`None`] if this is [`Match::All`].
    ///
    /// # Example
    ///
    /// ```
    /// use clack_common::events::Match;
    ///
    /// assert_eq!(Match::Specific(42).into_specific(), Some(42));
    /// assert_eq!(Match::<u16>::All.into_specific(), None);
    /// ```
    pub fn into_specific(self) -> Option<T> {
        match self {
            Specific(v) => Some(v),
            All => None,
        }
    }

    /// Returns whether this matcher is [`Match::All`].
    /// # Example
    ///
    /// ```
    /// use clack_common::events::Match;
    ///
    /// assert!(Match::<u16>::All.is_all());
    /// assert!(!Match::Specific(42).is_all());
    /// ```
    #[inline]
    pub const fn is_all(&self) -> bool {
        matches!(self, All)
    }

    /// Returns whether this matcher is [`Match::Specific`].
    /// # Example
    ///
    /// ```
    /// use clack_common::events::Match;
    ///
    /// assert!(Match::Specific(42).is_specific());
    /// assert!(!Match::<u16>::All.is_specific());
    /// ```
    #[inline]
    pub const fn is_specific(&self) -> bool {
        matches!(self, Specific(_))
    }
}

impl<T> From<T> for Match<T> {
    #[inline]
    fn from(value: T) -> Self {
        Specific(value)
    }
}

impl From<u8> for Match<u16> {
    #[inline]
    fn from(value: u8) -> Self {
        Specific(value.into())
    }
}

impl From<u8> for Match<u32> {
    #[inline]
    fn from(value: u8) -> Self {
        Specific(value.into())
    }
}

impl From<u16> for Match<u32> {
    #[inline]
    fn from(value: u16) -> Self {
        Specific(value.into())
    }
}

impl From<ClapId> for Match<u32> {
    #[inline]
    fn from(value: ClapId) -> Self {
        Specific(value.get())
    }
}

impl<T: PartialEq> Match<T> {
    /// Returns `true` if the given [`Match`] matches this one, `false` otherwise.
    ///
    /// This will always return true if any of the two is [`Match::All`]. Otherwise, if both values
    /// are specific, they are compared directly (using their [`PartialEq`] implementation).
    ///
    /// # Example
    ///
    /// ```
    /// use clack_common::events::Match;
    ///
    /// assert!(Match::Specific(42).matches(42));
    /// assert!(!Match::Specific(42).matches(21));
    ///
    /// assert!(Match::Specific(42).matches(Match::All));
    /// assert!(Match::All.matches(42));
    /// assert!(Match::<u16>::All.matches(Match::All));
    /// ```
    #[inline]
    pub fn matches(&self, other: impl Into<Match<T>>) -> bool {
        match (self, other.into()) {
            (Specific(x), Specific(y)) => *x == y,
            _ => true,
        }
    }
}

impl Match<u16> {
    /// Creates the [`Match`] that corresponds to the given raw C-FFI compatible `i16` type.
    #[inline]
    pub const fn from_raw(raw: i16) -> Self {
        if raw < 0 { All } else { Specific(raw as u16) }
    }

    /// Returns the raw C-FFI compatible `i16` type that corresponds to this [`Match`].
    ///
    /// If this matches a specific value, it is returned. Otherwise, if this matches all values, it
    /// returns `-1`.
    #[inline]
    pub const fn to_raw(&self) -> i16 {
        match self {
            Specific(raw) => *raw as i16,
            All => -1,
        }
    }
}

impl Match<u32> {
    /// Creates the [`Match`] that corresponds to the given raw C-FFI compatible `i32` type.
    #[inline]
    pub const fn from_raw(raw: i32) -> Self {
        if raw < 0 { All } else { Specific(raw as u32) }
    }

    /// Returns the raw C-FFI compatible `i32` type that corresponds to this [`Match`].
    ///
    /// If this matches a specific value, it is returned. Otherwise, if this matches all values, it
    /// returns `-1`.
    #[inline]
    pub const fn to_raw(&self) -> i32 {
        match self {
            Specific(raw) => *raw as i32,
            All => -1,
        }
    }
}

macro_rules! impl_event_pckn {
    (self.$($raw_event:ident).*) => {
        /// The [`Pckn`](crate::events::Pckn) tuple indicating which note(s) this note event targets.
        #[inline]
        pub const fn pckn(&self) -> crate::events::Pckn {
            Pckn::from_raw(
                self.$($raw_event).*.port_index,
                self.$($raw_event).*.channel,
                self.$($raw_event).*.key,
                self.$($raw_event).*.note_id,
            )
        }

        /// Sets the [`Pckn`](crate::events::Pckn) tuple for this event.
        #[inline]
        pub const fn set_pckn(&mut self, pckn: crate::events::Pckn) {
            self.$($raw_event).*.port_index = pckn.raw_port_index();
            self.$($raw_event).*.channel = pckn.raw_channel();
            self.$($raw_event).*.key = pckn.raw_key();
            self.$($raw_event).*.note_id = pckn.raw_note_id();
        }

        /// Sets the [`Pckn`](crate::events::Pckn) tuple for this event.
        ///
        /// This method takes and returns ownership of the event, allowing it to be used in a
        /// builder-style pattern.
        #[inline]
        pub const fn with_pckn(mut self, pckn: crate::events::Pckn) -> Self {
            self.set_pckn(pckn);
            self
        }

        /// The index of the note port this event targets.
        ///
        /// This returns [`Match::All`] if this event targets all possible note ports.
        #[inline]
        pub const fn port_index(&self) -> Match<u16> {
            Match::<u16>::from_raw(self.$($raw_event).*.port_index)
        }

        /// Sets the index of the note port this event targets.
        ///
        /// Use [`Match::All`] to target all possible note ports.
        #[inline]
        pub const fn set_port_index(&mut self, port_index: Match<u16>) {
            self.$($raw_event).*.port_index = port_index.to_raw()
        }

        /// Sets the index of the note port this event targets.
        ///
        /// Use [`Match::All`] to target all possible note ports.
        ///
        /// This method takes and returns ownership of the event, allowing it to be used in a
        /// builder-style pattern.
        #[inline]
        pub const fn with_port_index(mut self, port_index: Match<u16>) -> Self {
            self.$($raw_event).*.port_index = port_index.to_raw();
            self
        }

        /// The note channel this event targets (0-15).
        ///
        /// This returns [`Match::All`] if this event targets all possible note channels.
        #[inline]
        pub const fn channel(&self) -> Match<u16> {
            Match::<u16>::from_raw(self.$($raw_event).*.channel)
        }

        /// Sets the note channel this event targets (0-15).
        ///
        /// Use [`Match::All`] to target all possible channels.
        #[inline]
        pub const fn set_channel(&mut self, channel: Match<u16>) {
            self.$($raw_event).*.channel = channel.to_raw();
        }

        /// Sets the note channel this event targets (0-15).
        ///
        /// Use [`Match::All`] to target all possible channels.
        ///
        /// This method takes and returns ownership of the event, allowing it to be used in a
        /// builder-style pattern.
        #[inline]
        pub const fn with_channel(mut self, channel: Match<u16>) -> Self {
            self.$($raw_event).*.channel = channel.to_raw();
            self
        }

        /// The key of the note(s) this event targets (0-127).
        ///
        /// This returns [`Match::All`] if this event targets all possible note keys.
        #[inline]
        pub const fn key(&self) -> Match<u16> {
            Match::<u16>::from_raw(self.$($raw_event).*.key)
        }

        /// Sets the key of the note(s) this event targets (0-127).
        ///
        /// Use [`Match::All`] to target all possible note keys.
        #[inline]
        pub const fn set_key(&mut self, key: Match<u16>) {
            self.$($raw_event).*.key = key.to_raw();
        }

        /// Sets the key of the note(s) this event targets (0-127).
        ///
        /// Use [`Match::All`] to target all possible note keys.
        ///
        /// This method takes and returns ownership of the event, allowing it to be used in a
        /// builder-style pattern.
        #[inline]
        pub const fn with_key(mut self, key: Match<u16>) -> Self {
            self.$($raw_event).*.key = key.to_raw();
            self
        }

        /// The specific ID of the Note this event targets.
        ///
        /// This returns [`Match::All`] if this event doesn't target a specific note, or doesn't
        /// provide a Note ID.
        #[inline]
        pub const fn note_id(&self) -> Match<u32> {
            Match::<u32>::from_raw(self.$($raw_event).*.note_id)
        }

        /// Sets the specific ID of the Note this event targets.
        ///
        /// Use [`Match::All`] to not target a single specific note in particular.
        #[inline]
        pub const fn set_note_id(&mut self, note_id: Match<u32>) {
            self.$($raw_event).*.note_id = note_id.to_raw();
        }

        /// Sets the specific ID of the Note this event targets.
        ///
        /// Use [`Match::All`] to not target a single specific note in particular.
        ///
        /// This method takes and returns ownership of the event, allowing it to be used in a
        /// builder-style pattern.
        #[inline]
        pub const fn with_note_id(mut self, note_id: Match<u32>) -> Self {
            self.$($raw_event).*.note_id = note_id.to_raw();
            self
        }
    };
}

use crate::utils::ClapId;
pub(crate) use impl_event_pckn;
