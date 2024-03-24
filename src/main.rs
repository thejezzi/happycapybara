pub mod slaughterhouse;

#[derive(Clone, Debug)]
struct Cow {
    name: String,
}
impl<'a> slaughterhouse::Animal<'a> for Cow {
    fn race(&self) -> &str {
        "Cow"
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}
impl Cow {
    fn new(name: &str) -> Box<Self> {
        Box::new(Cow { name: name.to_string() })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut slayhouse = slaughterhouse::new();
    slayhouse.add_location("Farm");
    let _ = slayhouse.add_unit("Farm", "Barn", 3);

    let cow = Cow::new("Bessie");
    let my_second_cow = Cow::new("Ulla");
    slayhouse.add_animal("Farm", "Barn", cow)?;
    slayhouse.add_animal("Farm", "Barn", my_second_cow)?;
    slayhouse.get_animal("Farm", "Barn", 0)?;

    println!("{:?}", slayhouse);
    Ok(())
}
