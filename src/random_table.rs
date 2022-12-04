use rltk::RandomNumberGenerator;

pub struct RandomEntry {
    name : String,
    weight : i32
}

impl RandomEntry {
    pub fn new<S:ToString>(name : S, weight : i32) -> RandomEntry {
        return RandomEntry{ name : name.to_string(), weight };
    }
}

#[derive(Default)]
pub struct RandomTable {
    entries : Vec<RandomEntry>,
    total_weight : i32
}

impl RandomTable {
    pub fn new() -> RandomTable {
        return RandomTable{ entries : Vec::new(), total_weight : 0 };
    }

    pub fn add<S:ToString>(mut self, name : S, weight : i32) -> RandomTable {
        self.total_weight += weight;
        self.entries.push(RandomEntry::new(name.to_string(), weight));
        return self;
    }

    pub fn roll(&self, rng : &mut RandomNumberGenerator) -> String {
        // Partitions the total weight into chunks for each item;
        // Depending on which chunk the roll falls into, returns the corresponding item
        // Total=100(1:50,2:25,3:25) -> roll between 1 and 99 -> if 1-49: return item 1;
        // if 50-74: return item 2; if 74-99: return item 3

        if self.total_weight == 0 { return "None".to_string(); }
        let mut roll = rng.roll_dice(1, self.total_weight) - 1;
        let mut index : usize = 0;

        while roll > 0 {
            if roll < self.entries[index].weight {
                return self.entries[index].name.clone();
            }

            roll -= self.entries[index].weight;
            index += 1;
        }

        return "None".to_string();
    }
}
