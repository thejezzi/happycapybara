use crate::slaughterhouse::AnimalLike;

mod slaughterhouse {
    use std::{
        collections::HashMap,
        error::Error,
        ops::{Deref, DerefMut},
    };

    type Hook<'a> = Option<Box<dyn AnimalLike<'a> + 'a>>;

    pub struct Hall<'a> {
        hooks: Vec<Hook<'a>>,
    }

    impl<'a> Hall<'a> {
        pub fn new(capacity: usize) -> Self {
            Self {
                hooks: vec![None; capacity],
            }
        }
    }

    type Unit<'a> = HashMap<&'a str, Hall<'a>>;
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
            animal: Box<dyn AnimalLike<'a>>,
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
        ) -> Result<Box<dyn AnimalLike<'a> + 'a>, Box<dyn Error>> {
            let animal = self
                .get(location)
                .and_then(|unit| unit.get(unit_name))
                .and_then(|hall| hall.hooks.get(index).cloned())
                .flatten();
            animal.ok_or("Animal not found".into())
        }

        pub fn iter_hooks(&self) -> impl Iterator<Item = &Hook<'a>> {
            self.iter()
                .flat_map(move |(_, unit)| unit.iter().flat_map(|(_, hall)| hall.hooks.iter()))
        }
    }

    pub trait AnimalLike<'a>: AnimalClone<'a> + std::fmt::Debug {
        fn race(&self) -> &str;
        fn get_name(&self) -> String;
    }

    pub trait AnimalClone<'a> {
        fn clone_box(&self) -> Box<dyn AnimalLike<'a> + 'a>;
    }

    impl<'a, T> AnimalClone<'a> for T
    where
        T: 'a + AnimalLike<'a> + Clone,
    {
        fn clone_box(&self) -> Box<dyn AnimalLike<'a> + 'a> {
            Box::new(self.clone())
        }
    }

    impl<'a> Clone for Box<dyn AnimalLike<'a> + 'a> {
        fn clone(&self) -> Box<dyn AnimalLike<'a> + 'a> {
            self.clone_box()
        }
    }

    pub fn new<'a>() -> Slaughterhouse<'a> {
        Slaughterhouse::new()
    }
}

#[derive(Clone, Debug)]
struct Cow {
    name: String,
}
impl<'a> slaughterhouse::AnimalLike<'a> for Cow {
    fn race(&self) -> &str {
        "Cow"
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
}
impl Cow {
    fn new() -> Box<Self> {
        Box::new(Cow {
            name: "Bessie".to_string(),
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut slayhouse = slaughterhouse::new();
    slayhouse.add_location("Farm");
    let _ = slayhouse.add_unit("Farm", "Barn", 5);

    let cow = Cow::new();
    let my_second_cow = Cow::new();
    let hook_index = slayhouse.add_animal("Farm", "Barn", cow.clone())?;
    slayhouse.add_animal("Farm", "Barn", my_second_cow)?;
    let animal = slayhouse.get_animal("Farm", "Barn", 0)?;

    println!("{:?} was added to {}", animal, hook_index + 1);
    slayhouse.iter_hooks().for_each(|hook| {
        print!("{:?} ", hook);
    });
    println!();

    println!("{:?}", (*cow).get_name());

    Ok(())
}
