//!  Conditions are what SpecTcl called 'gates'  When a condition is
//!  *applied* to a spectrum it *gates* that spectrum in the sense
//!  That the condition can determine if a spectrum is incremented
//!  For any specific event.
//!
//!  Thus a condition can be thought of as a boolean function defined
//!  over the parameter values of an event.   What makes a struct
//!  a condition is for it to implement the Condition trait
//!  defined by this module.
//!
//!  For spectra to have a condition applied to them, they need
//!  some reference to a gate which is evaluated over each event
//!  polymorphically.  In the case of SpecTcl this is handled using
//!  C++ virtual functions and gate container pointer-like objects.
//!  For Rust, the mechanism of dynamic dispatch requires the use
//!  of *trait objects*   A trait object is a pointer like container
//!  (e.g. Rc or Box) which is defined to contain/reference an
//!  object that is known to implement a specific trait.  This trait,
//!  then defines the interface presented by objects in that container
//!  to clients via the container.
//!
//!  In our case, in order to support transparent replacement, we'll use
//!  Rc<dyn Condition>.  These get cloned to pass them to
//!  spectra where Rc::downgrade() turns them into Weak references.
//!  This trick will get around the SpecTcl problem of *etnernal gates*
//!  Since it's too much trouble, in general, to track down all
//!  references to a conition, in SpecTcl, gates are never deleted, but
//!  turned into False  gates.  This provide known behavior but some
//!  tricks need to be employed to make those gates invisible in the
//!  gate dictionary (effectively assuming that all False gates are'
//!  actually deleted).   In Rust, a Weak referenc does not prevent
//!  The deletion of the underlying object (via dropping the
//!  remaining Rc containers),  instead, the Weak::upgrade() method is
//!  required to to actually use a Weak reference to an object and,
//!  if the underlying object has been deleted this will return
//!  None.  We can treat None both as:
//!
//!  *    A false evaluation of the condition.
//!  *    A signal that we should remove the condition from whatever it's
//! being used for.
//!
//!  Organization of this module is similar to the ring_items module
//!  We define:
//!
//!   *  The traits needed to implement a condition (plural because
//!  there may need to be traits to get a description of the condition that
//!  can be used to create textual or graphical displays of the condition).
//!   *   A ConditionDictionary type which, when instantiated provides a
//! mechanism to lookup Conditions from names assigned to them.
//!

use crate::parameters;
use std::collections::HashMap;
use std::rc::Rc;

/// The Container trait defines the interface to a condition through
/// a gate container.   This interface includes:
/// *  Support for an evaluation of the condition for a flattened
/// event
/// *  Support for caching the evaluation of the condition
///
pub trait Condition {
    // Mandatory methods

    ///
    /// Evaluate the condition for a specific event.
    /// If the implementor supports caching, this method should
    /// save the results of the evaulation and indicate its cache
    /// is valid (e.g. get_cached_value calls prior to invalidate_cache
    /// should return Some(\the_cached_value))
    ///
    fn evaluate(&self, event: &parameters::FlatEvent) -> bool;

    /// Optional methods:
    /// Caching not implemented is the default.
    ///
    fn get_cached_value(&self) -> Option<bool> {
        None
    }
    fn invalidate_cache(&self) {}
    ///
    /// The method that really sould be called to check a gate:
    /// If the object has a cached value, the cached value
    /// is returned, otherwise the evaluate, required method is
    /// invoked to force condition evaluation.
    ///
    fn check(&self, event: &parameters::FlatEvent) -> bool {
        if let Some(b) = self.get_cached_value() {
            b
        } else {
            self.evaluate(event)
        }
    }
}

/// The ConditionContainer is the magic by which
/// Condition objects get dynamic dispatch to their checking
/// Condition methods
///
pub type Container = Rc<dyn Condition>;

/// ConditionDictionary contains a correspondence between textual
/// names and conditions held in Containers.
/// This provides storage and lookup for conditions that are created
/// in the rustogrammer program through e.g. commands or applying
/// a condition to an spectrum.
///
pub type ConditionDictionary = HashMap<String, Container>;


/// The True gate is implemented in this module and returns True
/// no matter what the event contains.  It serves as a trival example
/// of how conditions can be implemented.  No caching is required
/// for the True gate:

pub struct True {}
impl Condition for True {
    fn evaluate(&self, event: &parameters::FlatEvent) -> bool {
        true
    }
}

/// The false gate is implemented in this module and returns
/// False no matter what the event contains.  It servers as a trivial
/// example of a condition implementation
///
pub struct False {}
impl Condition for False {
    fn evaluate(&self, event : &parameters::FlatEvent) -> bool {
        false
    }
}