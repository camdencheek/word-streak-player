use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashSet;
use std::sync::Arc;

struct Tile {
    letter: String,
    word_multiplier: u16,
    letter_multiplier: u16,
}

impl Tile {
    pub fn new(letter: String, letter_multiplier: u16, word_multiplier: u16 ) -> Tile {
        Tile {
            letter: letter,
            word_multiplier: word_multiplier,
            letter_multiplier: letter_multiplier,
        }

    }

    fn letter_value(&self) -> u16 {
        self.letter_multiplier * match self.letter.as_str() {
            "a" => 1, "b" => 4, "c" => 4, "d" => 2, "e" => 1, "f" => 4,
            "g" => 3, "h" => 3, "i" => 1, "j" => 10, "k" => 5, "l" => 2,
            "m" => 4, "n" => 2, "o" => 1, "p" => 4, "qu" => 10, "r" => 1,
            "s" => 1, "t" => 1, "u" => 2, "v" => 5, "w" => 4, "x" => 10,
            "y" => 3, "z" => 10, _ => panic!("self.letter is not a valid tile letter"),
        } 
    }
}

#[derive(Copy)]
struct BoardLocation(usize,usize);

impl Clone for BoardLocation {
    fn clone(&self) -> BoardLocation { *self }
}

struct Board {
    grid: [[Tile; 4]; 4],
}

impl Board {
    fn get_adjacent_tiles(&self, loc: &BoardLocation) -> Vec<BoardLocation> {
        let mut adjacent: Vec<BoardLocation> = Vec::new();
        for row in (loc.0 as isize)-1..(loc.0 as isize)+2 {
            for col in (loc.1 as isize)-1..(loc.1 as isize)+2 {
                if row < 0 || row > 3 || col < 0 || col > 3 {
                    continue;
                } else if row == (loc.0 as isize) && col == (loc.1 as isize) {
                    continue;
                } else {
                    adjacent.push(BoardLocation(row as usize,col as usize))
                }
            }
        }
        adjacent
    }

    fn get_tile(&self, loc: &BoardLocation) -> &Tile {
        &self.grid[loc.0][loc.1]
    }
}


struct Word {
    loc_vector: Vec<BoardLocation>,
    board: Arc<Board>,
}

impl Word {
    pub fn new(board: Arc<Board>) -> Word {
        Word {
            loc_vector: Vec::new(),
            board: board,
        }
    }
    
    fn add_tile(&mut self, loc: BoardLocation) {
        self.loc_vector.push(loc)
    }

    fn get_string(&self) -> String {
        let mut word = "".to_string();
        for loc in &self.loc_vector {
            word += &self.board.get_tile(loc).letter
        }
        word
    }

    fn get_score(&self) -> u16 {
        let mut score = 0;
        let mut multiplier = 1;
        for loc in &self.loc_vector {
            let tile = self.board.get_tile(loc);
            score += tile.letter_value();
            multiplier *= tile.word_multiplier;
        }
        (score * multiplier)
    }

    fn uses_loc(&self, loc: &BoardLocation) -> bool {
        for used_loc in &self.loc_vector {
            if used_loc.0 == loc.0 && used_loc.1 == loc.1 {
                return true
            }
        } 
        false
    }
}

fn grown_words(word: Box<Word>, length: u8) -> Vec<Box<Word>> {
    let mut new_words: Vec<Box<Word>> = Vec::new();
    for adjacent in word.board.get_adjacent_tiles(&word.loc_vector[word.loc_vector.len() -1]) {
        if !word.uses_loc(&adjacent) {
            let mut new_word = Word::new(word.board.clone());
            for loc in &word.loc_vector {
                new_word.add_tile(*loc);
            }
            new_word.add_tile(adjacent);
            new_words.push(Box::new(new_word));
            let mut new_new_word = Word::new(word.board.clone());
            for loc in &word.loc_vector {
                new_new_word.add_tile(*loc);
            }
            new_new_word.add_tile(adjacent);
            if length-1 != 1 {
                new_words.append(&mut grown_words(Box::new(new_new_word), length-1));
            }
        }
    }

    new_words
}

fn find_words(board: Arc<Board>) -> Vec<Box<Word>> {
    let max_length = 8;
    let mut words: Vec<Box<Word>>  = Vec::new();
    for row in 0..4 {
        for col in 0..4 {
            let mut new_word = Word::new(board.clone());
            new_word.add_tile(BoardLocation(row,col));
            println!["{},{},{}",row,col,new_word.get_string()];
            words.append(&mut grown_words(Box::new(new_word), max_length));
        }
    } 
    
    words
}



fn main() {
    let path = Path::new("enable1.txt");
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };

    let mut s = String::new();
    let mut words = HashSet::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("Couldn't read {}: {}", display, why.description()),
        Ok(_) => {
            for word in s.lines() {
                words.insert(word.trim().to_string());
            }
        }
    }

    let row1 = [Tile::new("o".to_string(), 1, 1),
                Tile::new("e".to_string(), 3, 1),
                Tile::new("i".to_string(), 3, 1),
                Tile::new("j".to_string(), 1, 1)];
    let row2 = [Tile::new("r".to_string(), 1, 1),
                Tile::new("e".to_string(), 1, 3),
                Tile::new("c".to_string(), 3, 1),
                Tile::new("r".to_string(), 1, 1)];
    let row3 = [Tile::new("d".to_string(), 1, 1),
                Tile::new("a".to_string(), 1, 1),
                Tile::new("s".to_string(), 1, 1),
                Tile::new("a".to_string(), 1, 1)];
    let row4 = [Tile::new("r".to_string(), 1, 1),
                Tile::new("i".to_string(), 1, 1),
                Tile::new("t".to_string(), 1, 1),
                Tile::new("e".to_string(), 1, 1)];

    let board = Board{ grid: [row1, row2, row3, row4] };
    let mut potential_words = find_words(Arc::new(board));
    println!["{}",words.len()];
    potential_words.retain(|x| words.contains(&x.get_string()));
    potential_words.sort_by(|a,b| a.get_score().cmp(&b.get_score()));
    let mut total_points: u16 = 0;
    for word in &potential_words {
        println!["{}, {}",word.get_string(), word.get_score()];
        total_points += word.get_score();
    }

    println!["Total score: {}", total_points];

}
