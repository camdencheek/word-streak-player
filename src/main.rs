extern crate image;
extern crate img_hash;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;
use std::process::Command;
use img_hash::{ImageHash, HashType};
use image::{ImageBuffer,DynamicImage,imageops};


struct Tile {
    letter: String,
    multiplier: Multiplier,
}

impl Tile {
    pub fn new(letter: String, multiplier: Multiplier ) -> Tile {
        Tile {
            letter: letter,
            multiplier: multiplier,
        }
    }

    fn letter_value(&self) -> u16 {
        match self.letter.as_str() {
            "a" => 1, "b" => 4, "c" => 4, "d" => 2, "e" => 1, "f" => 4,
            "g" => 3, "h" => 3, "i" => 1, "j" => 10, "k" => 5, "l" => 2,
            "m" => 4, "n" => 2, "o" => 1, "p" => 4, "qu" => 10, "r" => 1,
            "s" => 1, "t" => 1, "u" => 2, "v" => 5, "w" => 4, "x" => 10,
            "y" => 3, "z" => 10, _ => panic!("self.letter is not a valid tile letter"),
        } 
    }
}

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
            match tile.multiplier {
                Multiplier::Letter(m) => { 
                    score += tile.letter_value() * m; }
                Multiplier::Word(m) => {
                    score += tile.letter_value();
                    multiplier *= m; }
                Multiplier::Unmultiplied => {
                    score += tile.letter_value(); }
            }
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
    let max_length = 5;
    let mut words: Vec<Box<Word>>  = Vec::new();
    let mut children = vec![];
    for row in 0..4 {
        for col in 0..4 {
            let mut new_word = Word::new(board.clone());
            new_word.add_tile(BoardLocation(row,col));
            children.push(thread::spawn(move || {
                println!["{},{},{}",row,col,new_word.get_string()];
                grown_words(Box::new(new_word), max_length)
            }));
        }
    } 

    for child in children {
        match child.join() {
            Ok(ref mut result) => words.append(result),
            Err(_) => panic!["Test"],
        }
    }
    
    words
}

fn adbshell(command: String) {
    let output = Command::new("/opt/android-sdk/platform-tools/adb")
        .arg("shell")
        .arg(format!("{}", command))
        .output()
        .expect("Failed to execute ADB Shell command");
}

fn adbscreenshot() {
    let take_screenshot = Command::new("/opt/android-sdk/platform-tools/adb")
        .arg("shell")
        .arg("/system/bin/screencap")
        .arg("-p")
        .arg("/sdcard/screenshot.png")
        .output();

    let copy_screenshot = Command::new("/opt/android-sdk/platform-tools/adb")
        .arg("pull")
        .arg("/sdcard/screenshot.png")
        .arg("/tmp/screenshot.png")
        .output();
}

fn compare_images() {
    let image1 = image::open(&Path::new("/tmp/w1.png")).unwrap();
    let image2 = image::open(&Path::new("/tmp/s2.png")).unwrap();

    // These two lines produce hashes with 64 bits (8 ** 2),
    // using the Gradient hash, a good middle ground between 
    // the performance of Mean and the accuracy of DCT.
    let hash1 = ImageHash::hash(&image1, 8, HashType::Gradient);
    let hash2 = ImageHash::hash(&image2, 8, HashType::Gradient);

    println!("Image1 hash: {}", hash1.to_base64());
    println!("Image2 hash: {}", hash2.to_base64());

    println!("% Difference: {}", hash1.dist_ratio(&hash2));


}

enum Multiplier {
    Word(u16),
    Letter(u16),
    Unmultiplied,
}

fn get_board_from_image(img: DynamicImage) {
    let rownums = [668, 1008, 1340, 1678];
    let colnums = [40, 378, 716, 1050];

    for row in rownums.iter() {
        for col in colnums.iter() {
            //recognize_tile(row, col);
        }
    }
}

fn recognize_image_tile(start_row: u32, start_col: u32, img: u32) {
    //let multiplier_img = imageops::crop(img,
                                        //start_row,
                                        //start_col,
                                        //start_row + 100, 
                                        //start_col + 100);
 
    //let letter_img = imageops::crop(img,
                                    //start_row + 100,
                                    //start_col + 100,
                                    //start_row + 260,
                                    //start_col + 260);

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

    let row1 = [Tile::new("o".to_string(), Multiplier::Unmultiplied),
                Tile::new("e".to_string(), Multiplier::Letter(3)),
                Tile::new("i".to_string(), Multiplier::Letter(3)),
                Tile::new("j".to_string(), Multiplier::Unmultiplied)];
    let row2 = [Tile::new("r".to_string(), Multiplier::Unmultiplied),
                Tile::new("e".to_string(), Multiplier::Word(3)),
                Tile::new("c".to_string(), Multiplier::Letter(3)),
                Tile::new("r".to_string(), Multiplier::Unmultiplied)];
    let row3 = [Tile::new("d".to_string(), Multiplier::Unmultiplied),
                Tile::new("a".to_string(), Multiplier::Unmultiplied),
                Tile::new("s".to_string(), Multiplier::Unmultiplied),
                Tile::new("a".to_string(), Multiplier::Unmultiplied)];
    let row4 = [Tile::new("r".to_string(), Multiplier::Unmultiplied),
                Tile::new("i".to_string(), Multiplier::Unmultiplied),
                Tile::new("t".to_string(), Multiplier::Unmultiplied),
                Tile::new("e".to_string(), Multiplier::Unmultiplied)];

    let board = Board{ grid: [row1, row2, row3, row4] };
    let mut potential_words = find_words(Arc::new(board));
    println!["{}",words.len()];
    potential_words.retain(|x| words.contains(&x.get_string()));
    potential_words.sort_by(|a,b| a.get_score().cmp(&b.get_score()));
    for word in potential_words {
        println!["{}, {}",word.get_string(), word.get_score()];
    }

    let image = image::open(&Path::new("/tmp/screenshot.png")).unwrap();
    get_board_from_image(image);


}
