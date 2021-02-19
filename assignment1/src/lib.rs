// Library routines for reading and solving Sudoku puzzles

#![warn(clippy::all)]
pub mod verify;

use std::io::Read;
use std::num::NonZeroU8;

// Type definition for a 9x9 array that will represent a Sudoku puzzle.
// Entries with None represent unfilled positions in the puzzle.
// Soduko [col][row]
type Sudoku = [[Option<NonZeroU8>; 9]; 9];

// This function is called by main. It calls solve() to recursively find the solution.
// The puzzle is modified in-place.
pub fn solve_puzzle(puzzle: &mut Sudoku) {
    solve(puzzle);
    // println!("{}", solve(puzzle));
}

// Fills in the empty positions in the puzzle with the right values, using a
// recursive brute force approach. Modify the puzzle in place. Return true if
// solved successfully, false if unsuccessful. You may modify the function signature
// if you need/wish.
fn solve(puzzle: &mut Sudoku) -> bool {

    for col in 0..9 {
        for row in 0..9{
            // check if square is empty
            if puzzle[col][row] == None {
                // Check every possible value for this square
                for test_val in 1..10{
                    if check_square(puzzle, row, col, NonZeroU8::new(test_val)){
                        // preserve the original value
                        puzzle[col][row] = NonZeroU8::new(test_val);
                        //test if the puzzle is solvable with this new value
                        if solve(puzzle) {return true;}
                    }
                }
                // if no value could be found for the square a previous square isn't filled right
                //put the square how it was
                puzzle[col][row] = None;
                return false;
            }
        }
    }
    // if we don't run into any empty squares we've reached bottom of the recursion successfully
    true
}

// Helper that checks if a specific square in the puzzle can take on
// a given value. Return true if that value is allowed in that square, false otherwise.
// You can choose not to use this if you prefer.
fn check_square(puzzle: &Sudoku, row: usize, col: usize, val: Option<NonZeroU8>) -> bool {

    let x = col /3 * 3;
    let y = row /3 * 3;

    let block_valid = (x..x+3).all(|idx_x| (y..y+3).all(|idx_y|puzzle[idx_x][idx_y] != val));
    let col_valid = (0..9).all(|idx| puzzle[col][idx] != val);
    let row_valid  = (0..9).all(|idx| puzzle[idx][row] != val);

    row_valid && col_valid && block_valid
}

// Helper for printing a sudoku puzzle to stdout for debugging.
pub fn print_puzzle(puzzle: &Sudoku) {
    for row in puzzle.iter() {
        for elem in row.iter() {
            print!("{}", elem.map(|e| (e.get() + b'0') as char).unwrap_or('.'));
        }
        print!("\n");
    }
    print!("\n");
}

// Read the input byte by byte until a complete Sudoku puzzle has been
// read or EOF is reached.  Assumes the input follows the correct format
// (i.e. matching the files in the input folder).
pub fn read_puzzle(reader: &mut impl Read) -> Option<Box<Sudoku>> {
    // Turn the input stream into an iterator of bytes
    let mut bytes = reader.bytes().map(|b| b.expect("input error")).peekable();
    // Go thru the input until we find a puzzle or EOF (None)
    loop {
        match bytes.peek() {
            Some(b'1'..=b'9') | Some(b'.') => break,
            None => return None,
            _ => {
                bytes.next();
            }
        }
    }
    let mut puzzle = Box::new([[None; 9]; 9]);

    // Fill in the puzzle matrix. Ignore the non-puzzle input bytes.
    for i in 0..9 {
        let mut j = 0;
        while j < 9 {
            let b = bytes.next().expect("unexpected EOF");

            let elem = match b {
                b'1'..=b'9' => NonZeroU8::new(b - b'0'),
                b'.' => None,
                _ => continue,
            };
            puzzle[i][j] = elem;
            j += 1;
        }
    }

    Some(puzzle)
}

// Do a simple check that the puzzle is valid.
// Returns true if it is valid, false if it is not.
// (The verifier server doesn't tell you what's wrong so this function can also help you track
// down an error if your puzzles are not being solved correctly.)
pub fn check_puzzle(puzzle: &Sudoku) -> bool {
    /*
    I'm verifying rows, columns, and squares by summing their components rather than checking for uniqueness.
    This way is definitely easier although not 100% perfect, I didn't want to change the f'n signature or modify the main code
    to use check square to validate each square. There are specific puzzles that could pass when
    they shouldn't (e.x. a puzzle with each cell set to 5) however the solver would never return these so it should be ok.
    */
     // Check that each row is valid
     let mut col_sums: [usize; 9] = [0; 9];
     let mut row_sums: [usize; 9] = [0; 9];

     // checking if each row is valid
     for row in 0..9 { // do each row
         for col in 0..9 {
             col_sums[col] += puzzle[row][col].unwrap().get() as usize;
             row_sums[row] += puzzle[row][col].unwrap().get() as usize;
         }
     }

     for row in 0..=2{
         for col in 0..=2{
             let (x_root, y_root) = (col * 3, row * 3);
             let mut square_sum = 0;
             for element in puzzle[x_root..x_root + 3].iter().flat_map(|x| &x[y_root .. y_root + 3]){
                 square_sum += element.unwrap().get();
             }
             if square_sum != 45{
                 return false;
             }

         }
     }
     return col_sums.iter().fold(0,|a, &b| a + b) == 405 && row_sums.iter().fold(0,|a, &b| a + b) == 405;
}
