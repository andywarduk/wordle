use dictionary::Dictionary;
use solveapp::SolveApp;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmBoard {
    solve_app: SolveApp,
}

#[wasm_bindgen]
impl WasmBoard {
    pub fn add(&mut self, c: char) -> bool {
        self.solve_app.add(c)
    }

    pub fn remove(&mut self) -> bool {
        self.solve_app.remove()
    }

    pub fn toggle(&mut self, y: usize, x: usize) -> bool {
        self.solve_app.toggle(y, x)
    }

    pub fn toggle_column(&mut self, c: usize) -> bool {
        self.solve_app.toggle_col(c)
    }

    pub fn get_board(&self) -> Vec<u8> {
        let board = self.solve_app.board();

        board
            .iter()
            .flatten()
            .flat_map(|elem| match elem {
                solveapp::BoardElem::Empty => [0, 0],
                solveapp::BoardElem::Gray(c) => [1, *c as u8],
                solveapp::BoardElem::Yellow(c) => [2, *c as u8],
                solveapp::BoardElem::Green(c) => [3, *c as u8],
            })
            .collect()
    }

    pub fn calculate(&mut self) -> Option<usize> {
        self.solve_app.calculate();

        self.solve_app.words().count()
    }

    pub fn get_word(&self, index: usize) -> String {
        self.solve_app.get_word(index).unwrap_or_default()
    }
}

#[wasm_bindgen]
pub fn create_board() -> WasmBoard {
    // Load the dictionary
    let dictionary =
        Dictionary::new_from_bytes(include_bytes!("../../words.txt.gz"), false).unwrap();

    // Create the solve app
    let solve_app = SolveApp::new(dictionary);

    // Create board
    WasmBoard { solve_app }
}
