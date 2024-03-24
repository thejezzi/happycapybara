use std::collections::HashMap;
use std::error::Error;
use std::ops::{
    Deref,
    DerefMut,
};

/// The `Animal` trait is designed to identify entities that qualify as animals
/// and could be potentially added to a slaughterhouse's processing list.
/// This trait extends `AnimalClone` to enable the cloning of trait objects,
/// allowing them to be stored in a `Box` as dynamic trait objects, thus
/// facilitating polymorphism in Rust's type system.
///
/// An implementation of the `Animal` trait should look like this:
/// ```rust
/// use slaughterhouse::Animal;
///
/// #[derive(Debug, Clone)]
/// struct Dog {
///     name: String,
/// }
///
/// impl Animal for Dog {
///     fn race(&self) -> &str {
///         "Canine"
///     }
///
///     fn get_name(&self) -> String {
///         self.name.clone()
///     }
/// }
/// ```
///
/// The `AnimalClone` trait provides a mechanism for cloning trait objects,
/// specifically for types that implement the `Animal` trait. It's necessary due
/// to the default `Clone` trait not being directly applicable to trait objects.
///
/// The trait `AnimalClone` should be implemented for any animal type to enable
/// cloning of the trait objects. This is handled automatically via the provided
/// implementation for any type `T` where `T` implements `Animal` and `Clone`.
///
/// The implementation for `Box<dyn Animal>` ensures that boxed trait objects
/// can be cloned, allowing the dynamic types to be duplicated as needed,
/// ensuring that the original and the clone can exist independently.
pub trait Animal<'a>: AnimalClone<'a> + std::fmt::Debug {
    /// Returns the race of the animal as a string slice.
    fn race(&self) -> &str;

    /// Returns the name of the animal as a `String`.
    fn get_name(&self) -> String;
}

/// The `AnimalClone` trait is a workaround for Rust's limitation with cloning
/// boxed trait objects. It provides a method to clone a boxed trait object
/// without knowing its concrete type.
pub trait AnimalClone<'a> {
    /// Clones this object and returns a boxed object of the same type.
    fn clone_box(&self) -> Box<dyn Animal<'a> + 'a>;
}

/// The implementation of `AnimalClone` for any type `T` that satisfies the
/// constraints (`T` is part of the lifetime `'a`, implements `Animal` and
/// `Clone`). This implementation allows any animal type to be cloned into a
/// boxed trait object.
impl<'a, T> AnimalClone<'a> for T
where T: 'a + Animal<'a> + Clone
{
    /// Clones the concrete object into a boxed dynamic trait object.
    fn clone_box(&self) -> Box<dyn Animal<'a> + 'a> {
        Box::new(self.clone())
    }
}

/// Provides a way to clone a boxed trait object of the `Animal` trait.
/// This is essential for working with collections of boxed animals,
/// allowing for their duplication while preserving type information.
impl<'a> Clone for Box<dyn Animal<'a> + 'a> {
    /// Clones the boxed trait object, allowing for the duplication of animals
    /// within the system while maintaining the integrity of their types.
    fn clone(&self) -> Box<dyn Animal<'a> + 'a> {
        self.clone_box()
    }
}

type Hook<'a> = Option<Box<dyn Animal<'a> + 'a>>;

pub struct Hall<'a> {
    hooks: Vec<Hook<'a>>,
}

// TODO: Add documentation
impl<'a> Hall<'a> {
    pub fn new(capacity: usize) -> Self {
        Self { hooks: vec![None; capacity] }
    }
}

type Unit<'a> = HashMap<&'a str, Hall<'a>>;
type Locations<'a> = HashMap<&'a str, Unit<'a>>;

pub struct Slaughterhouse<'a>(Locations<'a>);

pub fn new<'a>() -> Slaughterhouse<'a> {
    Slaughterhouse::new()
}

impl<'a> Default for Slaughterhouse<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Deref for Slaughterhouse<'a> {
    type Target = Locations<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a> DerefMut for Slaughterhouse<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> Slaughterhouse<'a> {
    pub fn new() -> Self {
        Self(Locations::new())
    }

    pub fn add_location(&mut self, name: &'a str) {
        self.insert(name, Unit::new());
    }

    pub fn add_unit(
        &mut self,
        location: &'a str,
        name: &'a str,
        capacity: usize,
    ) -> Result<(), Box<dyn Error>> {
        self.get_mut(location)
            .ok_or("Location not found")?
            .insert(name, Hall::new(capacity));
        Ok(())
    }

    pub fn has_free_hook(&self) -> bool {
        self.iter().any(|(_, unit)| {
            unit.iter()
                .any(|(_, hall)| hall.hooks.iter().any(|hook| hook.is_none()))
        })
    }

    fn next_free_hook_index(&self) -> Result<usize, Box<dyn Error>> {
        let index = self
            .iter()
            .find_map(|(_, unit)| {
                unit.iter().find_map(|(_, hall)| {
                    hall.hooks.iter().position(|hook| hook.is_none())
                })
            })
            .ok_or("No free hooks")?;
        Ok(index)
    }

    pub fn add_animal(
        &mut self,
        location: &'a str,
        unit: &'a str,
        animal: Box<dyn Animal<'a>>,
    ) -> Result<usize, Box<dyn Error>> {
        if self.has_free_hook() {
            let index = self.next_free_hook_index()?;
            self.get_mut(location)
                .ok_or("Could not find to location")?
                .get_mut(unit)
                .ok_or("Could not find unit")?
                .hooks[index] = Some(animal);
            return Ok(index);
        }

        Err("No free hooks".into())
    }

    pub fn get_animal(
        &self,
        location: &'a str,
        unit_name: &'a str,
        index: usize,
    ) -> Result<Box<dyn Animal<'a> + 'a>, Box<dyn Error>> {
        let animal = self
            .get(location)
            .and_then(|unit| unit.get(unit_name))
            .and_then(|hall| hall.hooks.get(index).cloned())
            .flatten();
        animal.ok_or("Animal not found".into())
    }

    pub fn iter_hooks(&self) -> impl Iterator<Item = &Hook<'a>> {
        self.iter().flat_map(move |(_, unit)| {
            unit.iter().flat_map(|(_, hall)| hall.hooks.iter())
        })
    }
}

impl std::fmt::Debug for Slaughterhouse<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (location, unit) in self.iter() {
            writeln!(f, "{}", location)?;
            for (unit_name, hall) in unit {
                writeln!(f, "  {}", unit_name)?;
                for (index, hook) in hall.hooks.iter().enumerate() {
                    writeln!(f, "    {}: {:?}", index, hook)?;
                }
            }
        }
        Ok(())
    }
}
