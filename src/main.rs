mod slaughterhouse {
    use std::{
        collections::HashMap,
        error::Error,
        ops::{Deref, DerefMut},
    };

    type Hook = Option<Box<dyn AnimalLike>>;

    pub struct Hall {
        hooks: Vec<Hook>,
    }

    impl Hall {
        pub fn new(capacity: usize) -> Self {
            Self {
                hooks: vec![None; capacity],
            }
        }
    }

    type Unit<'a> = HashMap<&'a str, Hall>;
    type Locations<'a> = HashMap<&'a str, Unit<'a>>;

    pub struct Slaughterhouse<'a>(Locations<'a>);

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
                    unit.iter()
                        .find_map(|(_, hall)| hall.hooks.iter().position(|hook| hook.is_none()))
                })
                .ok_or("No free hooks")?;
            Ok(index)
        }

        pub fn add_animal(
            &mut self,
            location: &'a str,
            unit: &'a str,
            animal: Box<dyn AnimalLike>,
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
        ) -> Result<Box<dyn AnimalLike>, Box<dyn Error>> {
            let animal = self
                .get(location)
                .and_then(|unit| unit.get(unit_name))
                .and_then(|hall| hall.hooks.get(index).cloned())
                .flatten();
            animal.ok_or("Animal not found".into())
        }

        pub fn iter_hooks(&self) -> impl Iterator<Item = &Hook> {
            self.iter()
                .flat_map(|(_, unit)| unit.iter().flat_map(|(_, hall)| hall.hooks.iter()))
        }
    }

    pub trait AnimalLike: AnimalClone + std::fmt::Debug {
        fn race(&self) -> &str;
    }

    pub trait AnimalClone {
        fn clone_box(&self) -> Box<dyn AnimalLike>;
    }

    // Implement AnimalClone for all T where T is AnimalLike and Clone.
    impl<T> AnimalClone for T
    where
        T: 'static + AnimalLike + Clone,
    {
        fn clone_box(&self) -> Box<dyn AnimalLike> {
            Box::new(self.clone())
        }
    }

    impl Clone for Box<dyn AnimalLike> {
        fn clone(&self) -> Box<dyn AnimalLike> {
            self.clone_box()
        }
    }

    pub fn new<'a>() -> Slaughterhouse<'a> {
        Slaughterhouse::new()
    }
}

#[derive(Clone, Debug)]
struct Cow;
impl slaughterhouse::AnimalLike for Cow {
    fn race(&self) -> &str {
        "Cow"
    }
}
impl Cow {
    fn new() -> Box<Self> {
        Box::new(Cow)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut slayhouse = slaughterhouse::new();
    slayhouse.add_location("Farm");
    let _ = slayhouse.add_unit("Farm", "Barn", 5);

    let cow = Cow::new();
    let my_second_cow = Cow::new();
    let hook_index = slayhouse.add_animal("Farm", "Barn", cow)?;
    slayhouse.add_animal("Farm", "Barn", my_second_cow)?;
    let animal = slayhouse.get_animal("Farm", "Barn", 0)?;

    println!("{:?} was added to {}", animal, hook_index + 1);
    slayhouse.iter_hooks().for_each(|hook| {
        print!("{:?} ", hook);
    });
    println!();

    Ok(())
}
