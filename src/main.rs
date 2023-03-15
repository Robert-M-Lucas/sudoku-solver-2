use std::fs;
use std::time::{Instant};

#[derive(Clone, Copy, Debug)]
struct SudokuPossibility {
    b: u16
}

impl SudokuPossibility {
    pub fn new() -> Self {
        Self { b: 0}
    }

    pub fn new_all() -> Self {
        Self { b: 0b11111111_10000000 }
    }

    pub fn and(&self, other: &SudokuPossibility) -> SudokuPossibility {
        SudokuPossibility { b: self.b & other.b }
    }

    pub fn added(&self, possibility: u8) -> SudokuPossibility {
        let possibility = possibility - 1;

        let mask: u16 = 0b10000000_00000000 >> possibility;

        #[cfg(debug_assertions)]
        assert_eq!(0, self.b & mask);

        let b = self.b | mask;

        SudokuPossibility { b }
    }

    pub fn contains(&self, possibility:u8) -> bool {
        let possibility = possibility - 1;

        let mask: u16 = 0b10000000_00000000 >> possibility;

        (self.b & mask) != 0b00000000_00000000
    }

    pub fn removed(&self, possibility: u8) -> SudokuPossibility {
        let possibility = possibility - 1;

        let mask: u16 = !(0b10000000_00000000 >> possibility);

        let b = self.b & mask;

        SudokuPossibility { b }
    }

    pub fn get(&self) -> u8 {
        let mut b = self.b;
        for i in 0..9 {
            if b & 0b10000000_00000000 == 0b10000000_00000000 {
                return i + 1;
            }
            b = b << 1;
        }
        panic!("Get called when possibility not solved");
    }

    pub fn count(&self) -> u8 {
        let mut b = self.b;
        let mut j = 0;
        for i in 0..9 {
            if b & 0b10000000_00000000 == 0b10000000_00000000 {
                j += 1;
            }
            b = b << 1;
        }
        j
    }

    pub fn try_get(&self) -> Option<u8> {
        let mut b = self.b;
        for i in 0..9 {
            if b & 0b10000000_00000000 == 0b10000000_00000000 {
                return Some(i + 1);
            }
            b = b << 1;
        }
        None
    }

    pub fn marked_solved(&self) -> SudokuPossibility {
        SudokuPossibility { b: self.b | 0b00000000_00000001 }
    }

    pub fn is_solved(&self) -> bool {
        (self.b << 15) == 0b10000000_00000000
    }
}

#[derive(Clone, Copy, Debug)]
struct Sudoku {
    data: [u8; 9*9]
}

impl Sudoku {
    pub fn load(path: String) -> Self {
        let contents = fs::read_to_string(path)
            .expect("File read error");


        let mut c = 0;

        let mut data = [0; 9*9];

        for i in contents.chars() {
            if i == '_' {
                data[c] = 0;
            }
            else {
                let n = i.to_digit(10);
                if n.is_none() { continue; }
                data[c] = n.unwrap() as u8;
            }
            c += 1;
        }

        Self { data }
    }

    pub fn as_string(&self) -> String {
        let mut s: String = "".to_string();

        for y in 0..9 {
            if y != 0 {
                s += "\n";
                if y % 3 == 0 { s += "---------------------\n"; }
            }

            for x in 0..3 {
                s += &format!("{} {} {}", self.data[(y * 9) + (x * 3)], self.data[(y * 9) + (x * 3) + 1], self.data[(y * 9) + (x * 3) + 2]).to_string();
                if x != 2 { s += " | "; }
            }
        }

        s
    }

    pub fn guaranteed_completed(&self) -> Sudoku {
        let mut data = self.data;
        let mut possibilities: [SudokuPossibility; 9 * 9]  = [SudokuPossibility::new(); 9 * 9];

        for i in 0..(9 * 9) {
            if self.data[i] == 0 {
                possibilities[i] = SudokuPossibility::new_all();
            }
            else {
                possibilities[i] = possibilities[i].added(self.data[i]).marked_solved();
            }
        }

        let mut change = true;
        while change {
            change = false;

            for z in 0..9 {
                let mut remaining_in_row = SudokuPossibility::new_all();
                for i in 0..9 {
                    if possibilities[(z * 9) + i].is_solved() {
                        remaining_in_row = remaining_in_row.removed(possibilities[(z * 9) + i].get())
                    }
                }

                for j in 0..9 {
                    if !possibilities[(z * 9) + j].is_solved() {
                        possibilities[(z * 9) + j] = possibilities[(z * 9) + j].and(&remaining_in_row);
                    }
                }

                let mut remaining_in_column = SudokuPossibility::new_all();
                for i in 0..9 {
                    if possibilities[(i * 9) + z].is_solved() {
                        remaining_in_column = remaining_in_column.removed(possibilities[(i * 9) + z].get())
                    }
                }

                for j in 0..9 {
                    if !possibilities[(j * 9) + z].is_solved() {
                        possibilities[(j * 9) + z] = possibilities[(j * 9) + z].and(&remaining_in_column);
                    }
                }
            }

            for x in 0..3 {
                for y in 0..3 {
                    let mut remaining_in_box = SudokuPossibility::new_all();
                    for x2 in 0..3 {
                        for y2 in 0..3 {
                            if possibilities[(x * 3) + (y * 3 * 9) + x2 + (y2 * 9)].is_solved() {
                                remaining_in_box = remaining_in_box.removed(possibilities[(x * 3) + (y * 3 * 9) + x2 + (y2 * 9)].get())
                            }
                        }
                    }

                    for x2 in 0..3 {
                        for y2 in 0..3 {
                            if !possibilities[(x * 3) + (y * 3 * 9) + x2 + (y2 * 9)].is_solved() {
                                possibilities[(x * 3) + (y * 3 * 9) + x2 + (y2 * 9)] = possibilities[(x * 3) + (y * 3 * 9) + x2 + (y2 * 9)].and(&remaining_in_box);
                            }
                        }
                    }
                }
            }

            for y in 0..9 {
                for x in 0..9 {
                    if !possibilities[(y * 9) + x].is_solved() && possibilities[(y * 9) + x].count() == 1 {
                        data[(y * 9) + x] = possibilities[(y * 9) + x].get();
                        possibilities[(y * 9) + x] = possibilities[(y * 9) + x].marked_solved();
                        change = true;
                    }
                }
            }

            for z in 0..9 {
                'number_loop: for n in 1..10 {
                    let mut last: i32 = -1;
                    for i in 0..9 {
                        if possibilities[(z * 9) + i].contains(n) {
                            if last != -1 || possibilities[(z * 9) + i].is_solved() { continue 'number_loop; }
                            last = i as i32;
                        }
                    }
                    if last != -1 {
                        data[(z * 9) + (last as usize)] = n;
                        possibilities[(z * 9) + (last as usize)] = SudokuPossibility::new().added(n).marked_solved();
                        change = true;
                    }

                    let mut last: i32 = -1;
                    for j in 0..9 {
                        if possibilities[(j * 9) + z].contains(n) {
                            if last != -1 || possibilities[(j * 9) + z].is_solved() { continue 'number_loop; }
                            last = j as i32;
                        }
                    }
                    if last != -1 {
                        data[((last as usize) * 9) + z] = n;
                        possibilities[((last as usize) * 9) + z] = SudokuPossibility::new().added(n).marked_solved();
                        change = true;
                    }
                }
            }

            for x in 0..3 {
                for y in 0..3 {
                    'number_loop: for n in 1..10 {
                        let mut last: (i32, i32) = (-1, -1);
                        for x2 in 0..3 {
                            for y2 in 0..3 {
                                if possibilities[(x * 3) + (y * 3 * 9) + x2 + (y2 * 9)].contains(n) {
                                    if last.0 != -1 || possibilities[(x * 3) + (y * 3 * 9) + x2 + (y2 * 9)].is_solved() { continue 'number_loop; }
                                    last = (x2 as i32, y2 as i32);
                                }
                            }
                        }

                        if last.0 != -1 {
                            data[(x * 3) + (y * 3 * 9) + last.0 as usize + (last.1 as usize * 9)] = n;
                            possibilities[(x * 3) + (y * 3 * 9) + last.0 as usize + (last.1 as usize * 9)] = SudokuPossibility::new().added(n).marked_solved();
                            change = true;
                        }
                    }
                }
            }
        }

        Sudoku { data }
    }

    pub fn check_validity(&self) -> bool {
        for z in 0..9 {
            let mut remaining_in_row = SudokuPossibility::new_all();
            let mut remaining_in_column = SudokuPossibility::new_all();
            for i in 0..9 {
                if self.data[(z * 9) + i] == 0 { continue; }
                if !remaining_in_row.contains(self.data[(z * 9) + i]) { return false; }
                remaining_in_row = remaining_in_row.removed(self.data[(z * 9) + i]);

                if self.data[(i * 9) + z] == 0 { continue; }
                if !remaining_in_column.contains(self.data[(i * 9) + z]) { return false; }
                remaining_in_column = remaining_in_column.removed(self.data[(i * 9) + z]);
            }

        }

        for x in 0..3 {
            for y in 0..3 {
                let mut remaining_in_box = SudokuPossibility::new_all();
                for x2 in 0..3 {
                    for y2 in 0..3 {
                        if self.data[(y * 3 * 9) + (x * 3) + x2 + (y2 * 9)] == 0 { continue; }
                        if !remaining_in_box.contains(self.data[(y * 3 * 9) + (x * 3) + x2 + (y2 * 9)]) { return false; }
                        remaining_in_box = remaining_in_box.removed(self.data[(y * 3 * 9) + (x * 3) + x2 + (y2 * 9)]);
                    }
                }
            }
        }

        true
    }

    pub fn bruteforce(&self) -> Option<Sudoku> {
        return Self::recursively_attempt(self.clone(), 0);
    }

    fn recursively_attempt(state: Sudoku, cell: usize) -> Option<Sudoku> {
        if cell == 81 { return Some(state); }

        if state.data[cell] != 0 {
            if cell == 80 { return Some(state); }
            return Self::recursively_attempt(state.clone(), cell + 1);
        }

        for n in 1..10 {
            let mut data = state.data;
            data[cell] = n;
            let mut new_state = Sudoku { data };
            if !new_state.check_validity() { continue; }
            let result = Self::recursively_attempt(new_state, cell + 1);
            if result.is_some() { return result; }
        }

        return None;
    }
}

fn main() {
    let s = Sudoku::load("data\\sudoku.txt".to_string());
    // println!("{}\n\n{}", s.as_string(), s.guaranteed_completed().as_string());
    // println!("\n{}", s.guaranteed_completed().bruteforce().unwrap().as_string());
    println!("{}\n", s.as_string());
    let start = Instant::now();
    let result = s.guaranteed_completed().bruteforce();
    let time = start.elapsed();
    println!("\n{}\nTook {:?}", result.unwrap().as_string(), time);
    let start = Instant::now();
    let _result = s.bruteforce();
    let time = start.elapsed();
    println!("Took {:?}", time);
}
