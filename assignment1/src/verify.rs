// This module verifies Sudoku puzzles using curl

use crate::Sudoku;
use curl::easy;
use std::io::Write;

const URL: &str = "54.209.48.141:4590/verify";  // the verification server
const MATRIX_LENGTH: usize = 202;

// Callback handler for the curl easy handles
struct SudokuHandler {
    pub result: bool,
    puzzle: Box<Sudoku>,
}

impl SudokuHandler {
    pub fn new(puzzle: Box<Sudoku>) -> Self {
        Self {
            puzzle,
            result: false,
        }
    }
}

impl easy::Handler for SudokuHandler {  // this defines callbacks for curl to use
    // this function is called by curl when data has arrived from the server
    fn write(&mut self, data: &[u8]) -> Result<usize, easy::WriteError> {
        match std::str::from_utf8(data) {
            Ok(resp) => {
                self.result = resp == "1";  // "1" means verification succeeded
                println!("Server returned: {}", resp);
            }
            Err(_) => println!("Garbage server response"),
        }
        Ok(data.len())  // tell curl how many bytes we processed from data
    }

    // this function is called by curl when it wants more data to send to the server
    fn read(&mut self, mut data: &mut [u8]) -> Result<usize, easy::ReadError> {
        let start = data.as_ptr();  // keep track of current offset in data
        write_puzzle_to_json(&self.puzzle, &mut data).expect("JSON writing error");
        let end = data.as_ptr();  // find data offset
        let len = end as usize - start as usize;
        Ok(len)  // tell curl how many bytes we wrote into data
    }
}

// helper function for setting up a curl "easy" handle
fn create_easy(puzzle: Box<Sudoku>) -> Result<easy::Easy2<SudokuHandler>, curl::Error> {
    let handler = SudokuHandler::new(puzzle);
    let mut easy = easy::Easy2::new(handler);
    let mut headers = easy::List::new();  // HTTP headers

    headers.append("Content-Type: application/json")?;
    headers.append("Expect:")?;

    easy.http_headers(headers)?;
    easy.url(URL)?;
    easy.post(true)?;  // we use HTTP "POST" instead of "GET"
    easy.post_field_size(MATRIX_LENGTH as u64)?;
    Ok(easy)  // result is a curl easy handle
}

// convert a puzzle into JSON format to send to the server
fn write_puzzle_to_json(puzzle: &Sudoku, writer: &mut impl Write) -> std::io::Result<()> {
    write!(writer, "{{\"content\": [")?;

    for (i, row) in puzzle.iter().enumerate() {
        write!(writer, "[")?;

        for (j, elem) in row.iter().enumerate() {
            let val = elem.map(|e| e.get()).unwrap_or(0);
            write!(writer, "{}", val)?;
            if j < 8 {
                write!(writer, ",")?;
            }
        }

        write!(writer, "]")?;
        if i < 8 {
            write!(writer, ", ")?;
        }
    }

    write!(writer, "]}}")?;
    Ok(())
}

// This function is called from main to verifies all of the puzzles
pub fn verify_puzzles(puzzles: impl Iterator<Item = Box<Sudoku>>, num_connections: usize) {
    let mut total = 0;
    let mut verified = 0;
    let mut handles = Vec::new();
    let mut m = curl::multi::Multi::new();

    m.set_max_total_connections(num_connections).expect("Error setting max connections");

    for puzzle in puzzles {
        let easy = create_easy(puzzle).unwrap();
        handles.push(m.add2(easy).unwrap());

    }

    while m.perform().unwrap() > 0 {
        m.wait( &mut [], std::time::Duration::from_secs(1)).expect("Error waiting on multi");
    }

    for handle in handles{
        if handle.get_ref().result { verified += 1; }
        m.remove2(handle).expect("Unable to remove a handler from multi");
        total += 1;
    }

    // the following is the single-threaded version
    // for puzzle in puzzles {
    //     let easy = create_easy(puzzle).unwrap();
    //     easy.perform().unwrap();
    //     if easy.get_ref().result { verified += 1; }
    //     total += 1;
    // }

    println!("Verified {} out of {}", verified, total);
}
