//!  Compound conditions are conditions that are defined on other conditions.
//!  These are called *dependent conditions*.  Dependent conditions can be
//!  Either primitive conditions, like cuts or contours, or they can be other
//!  Compund conditions.  This nesting allows one to build up arbitrarily
//!  Complex gate logic.  The compund conditions that we define are:
//!  
//!  *  Not - takes a single condition and returns its boolean negation.
//!  *  And - takes an arbitrary number of dependent conditions and
//! requires all of them to be true.
//!  *  Or - takes an arbitrary number of dependent conditions and
//! Requires at least one to be true.
//!
//!  Compound conditions make not promise that their dependent gates are
//!  Fully evaluated.  It's perfectly fair game (and is the case) that
//!  Short circuit logic can be used to reduce the number of conditions
//!  that need to be evaluated until the truth or falsity of the
//!  main condition is known. All of these gate cache as well which
//!  further reduces the number of gate evaluation needed if a
//!  compound condition is applied to more than one target.
//!
//!  And and Or conditions depend on a cache and a vector of dependent conditions,
//!  This is abstracted out as a ConditionList which has the cached value and
//!  the dependent vector of conditions.
//!
//! ### Note
//!   conditions are stored as weak references to the underlying
//!  condition.  If upgrading the condition gives a None, the underlying
//!  condition has been deleted out from underneath us and is treated
//!  as returning false.
//!
use super::*;
use crate::parameters::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;

///
/// Not conditions take a single dependent condition and
/// return the boolean inverse of that condition when checked.
/// Sinced the computational complexity of the dependent condition
/// cannot be bounded (due to nested compound conditions), this is
/// a caching condition.
///
pub struct Not {
    dependent: Weak<RefCell<dyn Condition>>,
    cache: Option<bool>,
}

impl Not {
    pub fn new(cond: &Container) -> Not {
        Not {
            dependent: Rc::downgrade(&cond.clone()),
            cache: None,
        }
    }
}
impl Condition for Not {
    fn evaluate(&mut self, event: &FlatEvent) -> bool {
        let result = if let Some(d) = self.dependent.upgrade() {
            !d.borrow_mut().check(&event)
        } else {
            false
        };
        self.cache = Some(result);
        result
    }
    fn get_cached_value(&self) -> Option<bool> {
        self.cache
    }
    fn invalidate_cache(&mut self) {
        self.cache = None;
    }
}
//  The ConditionList provides common structure and code for
//  maintainng an arbitrary list of dependent conditions.
//  A cache variable is also associated with the list so that
//  common caching logic can be used.
//  this struct need not be exposed to the world:
struct ConditionList {
    dependent_conditions: Vec<Weak<RefCell<dyn Condition>>>,
    cache: Option<bool>,
}
impl ConditionList {
    pub fn new() -> ConditionList {
        ConditionList {
            dependent_conditions: Vec::new(),
            cache: None,
        }
    }
    pub fn add_condition(&mut self, c: &Container) -> &mut Self {
        self.dependent_conditions.push(Rc::downgrade(&c.clone()));

        self
    }
    // Clears the dependent conditions:
    //
    pub fn clear(&mut self) -> &mut Self {
        self.dependent_conditions.clear();
        self
    }
}

/// And conditions evaluate their condition list and require
/// all dependent conditions to be true if the condition
/// is to be true.  
///
/// * This is a caching condition.
/// * The evaluation is short circuited - that is if any
/// evaluation returns false, no more dependent conditions are
/// evaluated and all are evaluated as false.
///
pub struct And {
    dependencies: ConditionList,
}

impl And {
    pub fn new() -> And {
        And {
            dependencies: ConditionList::new(),
        }
    }
    pub fn add_condition(&mut self, c: &Container) -> &mut Self {
        self.dependencies.add_condition(c);
        self
    }
    pub fn clear(&mut self) -> &mut Self {
        self.dependencies.clear();
        self
    }
}
impl Condition for And {
    fn evaluate(&mut self, event: &FlatEvent) -> bool {
        let mut result = true; // Failed gates will contradict this.

        if let Some(c) = self.dependencies.cache {
            return c;
        } else {
            for d in &self.dependencies.dependent_conditions {
                if let Some(g) = d.upgrade() {
                    if !g.borrow_mut().check(&event) {
                        result = false;
                        break;
                    }
                } else {
                    result = false;
                    break;
                }
            }
        }

        self.dependencies.cache = Some(result);
        result
    }
    fn get_cached_value(&self) -> Option<bool> {
        self.dependencies.cache
    }
    fn invalidate_cache(&mut self) {
        self.dependencies.cache = None;
    }
}
///  Or is a compound condition that only requires that
///  one of its dependent gates is true for an event.
///
pub struct Or {
    dependencies: ConditionList,
}
impl Or {
    pub fn new() -> Or {
        Or {
            dependencies: ConditionList::new(),
        }
    }
    pub fn add_condition(&mut self, c: &Container) -> &mut Self {
        self.dependencies.add_condition(c);
        self
    }
    pub fn clear(&mut self) -> &mut Self {
        self.dependencies.clear();
        self
    }
}

impl Condition for Or {
    fn evaluate(&mut self, event: &FlatEvent) -> bool {
        let mut result = false;
        if let Some(b) = self.dependencies.cache {
            return b;
        } else {
            for d in &self.dependencies.dependent_conditions {
                if let Some(c) = d.upgrade() {
                    if c.borrow_mut().check(&event) {
                        result = true;
                        break;
                    }
                }
            }
        }
        self.dependencies.cache = Some(result);
        result
    }
    fn get_cached_value(&self) -> Option<bool> {
        self.dependencies.cache
    }
    fn invalidate_cache(&mut self) {
        self.dependencies.cache = None;
    }
}
