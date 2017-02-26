extern crate image;
extern crate img_hash;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
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

#[derive(Copy)]
struct BoardLocation(usize,usize);

impl Clone for BoardLocation {
    fn clone(&self) -> BoardLocation { *self }
}

struct Board {
    grid: Vec<Vec<Tile>>,
}

impl Board {
    fn new(board_vec: Vec<Vec<Tile>>) -> Board {
        Board {
            grid: board_vec,
        }
    }

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

fn grown_words(word: Box<Word>, length: u8, word_list: Arc<HashSet<String>>) -> Vec<Box<Word>> {
    let mut new_words: Vec<Box<Word>> = Vec::new();
    for adjacent in word.board.get_adjacent_tiles(&word.loc_vector.last().unwrap()) {
        if !word.uses_loc(&adjacent) {
            let mut new_word = Word::new(word.board.clone());
            for loc in &word.loc_vector {
                new_word.add_tile(*loc);
            }
            new_word.add_tile(adjacent);
            if word_list.contains(&new_word.get_string()) {
                new_words.push(Box::new(new_word));
            }
            let mut new_new_word = Word::new(word.board.clone());
            for loc in &word.loc_vector {
                new_new_word.add_tile(*loc);
            }
            new_new_word.add_tile(adjacent);
            if length-1 != 1 {
                new_words.append(&mut grown_words(Box::new(new_new_word), length-1, word_list.clone()));
            }
        }
    }

    new_words
}

fn find_words(board: Arc<Board>, word_list: Arc<HashSet<String>>) -> Vec<Box<Word>> {
    let max_length = 5;
    let mut words: Vec<Box<Word>>  = Vec::new();
    let mut children = vec![];
    for row in 0..4 {
        for col in 0..4 {
            let mut new_word = Word::new(board.clone());
            new_word.add_tile(BoardLocation(row,col));
            let word_list_clone = word_list.clone();
            children.push(thread::spawn(move || {
                println!["{},{},{}",row,col,new_word.get_string()];
                grown_words(Box::new(new_word), max_length, word_list_clone)
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

#[derive(Copy)]
enum Multiplier {
    Word(u16),
    Letter(u16),
    Unmultiplied,
}

impl Clone for Multiplier {
    fn clone(&self) -> Multiplier {
        match self {
            &Multiplier::Letter(l) => Multiplier::Letter(l),
            &Multiplier::Word(w) => Multiplier::Word(w),
            &Multiplier::Unmultiplied => Multiplier::Unmultiplied,
        }
    }
}

fn get_multiplier_hashes() -> Vec<(ImageHash,Multiplier)> {
    let multipliers = vec!["dl","tl","dw","tw","un"];
    let mut multiplier_hashes: Vec<(ImageHash,Multiplier)> = vec![];
    for multiplier in multipliers {
        let multiplier_image = image::open(&Path::new(&format!("images/{}.png",multiplier))).unwrap();

        let multiplier_hash = ImageHash::hash(&multiplier_image, 8, HashType::Gradient);
        multiplier_hashes.push((multiplier_hash, match multiplier {
            "dl" => Multiplier::Letter(2),
            "tl" => Multiplier::Letter(3),
            "dw" => Multiplier::Word(2),
            "tw" => Multiplier::Word(3),
            "un" => Multiplier::Unmultiplied ,
            _ => panic!("Not a proper multiplier string")
        }));
    }

    multiplier_hashes
}

fn get_letter_hashes() -> Vec<(ImageHash,String)> {
    let letters = vec!["a","b","c","d","e","f","g","h","i","j","k","l","m","n","o",
                        "p","qu","r","s","t","u","v","w","x","y","z"];

    let mut letter_hashes: Vec<(ImageHash,String)> = vec![];
    for letter in letters {
        let letter_image = image::open(&Path::new(&format!("images/{}.png",letter))).unwrap();

        let letter_hash = ImageHash::hash(&letter_image, 8, HashType::Gradient);
        letter_hashes.push((letter_hash, letter.to_string()));
    }

    letter_hashes
}

fn get_board_from_image(img: DynamicImage) -> Board {
    let rownums: Vec<u32> = vec![672, 1008, 1346, 1682];
    let colnums: Vec<u32> = vec![45, 382, 718, 1055];

    let letter_hashes = Arc::new(get_letter_hashes());
    let multiplier_hashes = Arc::new(get_multiplier_hashes());
    let image = Arc::new(img);
    
    let mut board_vec_children: Vec<Vec<thread::JoinHandle<Tile>>> = vec![];
    for row in rownums.iter() {
        let mut board_row_children= vec![];
        for col in colnums.iter() {
            let letter_hashes_clone = letter_hashes.clone();
            let multiplier_hashes_clone = multiplier_hashes.clone();
            let row_clone = row.clone();
            let col_clone = col.clone();
            let image_clone = image.clone();
            board_row_children.push(thread::spawn(move || {
                recognize_image_tile(
                    row_clone, 
                    col_clone, 
                    &image_clone, 
                    multiplier_hashes_clone, 
                    letter_hashes_clone)
            }))

        }
        board_vec_children.push(board_row_children);
    }

    let mut board_vec: Vec<Vec<Tile>> = vec![];
    for board_vec_child in board_vec_children {
        let mut board_row: Vec<Tile> = vec![];
        for row_child in board_vec_child {
            board_row.push(row_child.join().unwrap());
        }
        board_vec.push(board_row);
    }

    Board::new(board_vec)
}

fn recognize_image_tile(start_row: u32, start_col: u32, 
                        img: &DynamicImage, 
                        multiplier_hashes: Arc<Vec<(ImageHash,Multiplier)>>,
                        letter_hashes: Arc<Vec<(ImageHash,String)>>)
                        -> Tile {
    let mut image1 = img.clone();
    let mut image2 = img.clone();
    let multiplier_img = imageops::crop(&mut image1,
                                        start_col,
                                        start_row,
                                        95, 
                                        95).to_image();

    let multiplier_hash = ImageHash::hash(&multiplier_img, 8, HashType::Gradient);
    let mut min_difference = 1.0;
    let mut min_multiplier: Multiplier = Multiplier::Unmultiplied;
    for hash in multiplier_hashes.iter() {
        let difference = multiplier_hash.dist_ratio(&hash.0);
        if difference < min_difference {
            min_difference = difference;
            min_multiplier = hash.1;
        }
    }
 
    let letter_img = imageops::crop(&mut image2,
                                    start_col + 100,
                                    start_row + 100,
                                    160,
                                    160).to_image();

    let letter_hash = ImageHash::hash(&letter_img, 8, HashType::Gradient);
    let mut min_difference_letter = 1.0;
    let mut min_letter: String = "".to_string();
    for hash in letter_hashes.iter() {
        let difference = letter_hash.dist_ratio(&hash.0);
        if difference < min_difference_letter {
            min_difference_letter = difference;
            min_letter = hash.1.clone();
        }
    }
 
    println!["{},{}", match min_multiplier {
        Multiplier::Word(2) => "dw",
        Multiplier::Word(3) => "tw",
        Multiplier::Letter(2) => "dl",
        Multiplier::Letter(3) => "tl",
        Multiplier::Unmultiplied => "no",
        _ => "Error"
    }, min_letter];

    Tile::new(min_letter, min_multiplier)

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


    let image = image::open(&Path::new("images/screenshot.png")).unwrap();
    let board = get_board_from_image(image);
    let mut potential_words = find_words(Arc::new(board),Arc::new(words));
    potential_words.sort_by(|a,b| a.get_score().cmp(&b.get_score()));
    for word in potential_words {
        println!["{}, {}",word.get_string(), word.get_score()];
    }

}
