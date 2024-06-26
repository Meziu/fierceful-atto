//! Generic equipment management system applicable to [members](crate::member::Member).

use crate::member::Properties;

/// Equipment trait to interoperate with a [`Member`](crate::member::Member)'s [`Properties`](crate::member::Properties).
pub trait Equipment {
    type Properties: Properties;

    /// Returns the total property values generated from the used equipment.
    ///
    /// # Notes
    ///
    /// This value should be either mutably applied to a property object via [`Properties::apply_properties()`](crate::member::Properties::apply_properties)
    /// or be automatically summed up with a [`Member`](crate::member::Member)'s associated properties using
    /// [`Member::final_properties()`](crate::member::Member::final_properties).
    fn associated_properties(&self) -> Self::Properties;
}
