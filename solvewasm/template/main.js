import { perf_now, perf_end, perf_mark } from "$link(prefix=./|perf.js)";

// Load the WASM linkage
import init, { create_board } from "$link(prefix=./|../pkg/solvewasm.js)";

const debug = false; // Set to true to debug this script

// Spinner state
let spinner = true;

// Game board
let board;

// Initialise the WASM module
init({
  module_or_path: new URL("$link(../pkg/solvewasm_bg.wasm)", location.href),
})
  .then(() => {
    // Create the board
    board = create_board();

    // Hide spinner
    spinner_show(false);

    // Create the board
    create_board_tiles();

    // Display the board
    const main = document.getElementById("main");
    main.style.display = "flex";

    // Set up key handler
    document.onkeydown = (e) => {
      key_press(e);
    };
  })
  .catch((e) => {
    console.error("Caught error initialising WASM module:", e);
  });

function create_board_tiles() {
  const board_elem = document.getElementById("board");

  for (let y = 0; y < 6; y++) {
    for (let x = 0; x < 5; x++) {
      const cell = document.createElement("div");
      cell.id = `cell_${y}_${x}`;
      cell.classList.add("space");
      cell.classList.add("space_none");
      cell.onclick = cell_click;
      board_elem.appendChild(cell);
    }
  }
}

function key_press(e) {
  const key = e.key;

  if (
    key.length == 1 &&
    ((key >= "a" && key <= "z") || (key >= "A" && key <= "Z"))
  ) {
    // Letter key
    if (!e.ctrlKey && !e.altKey && !e.metaKey) {
      if (board.add(key.toUpperCase())) {
        calculate();
      }
    }
  } else if (key.length == 1 && key >= "1" && key <= "5") {
    // Number key
    if (!e.ctrlKey && !e.altKey && !e.metaKey) {
      if (board.toggle_column(parseInt(key) - 1)) {
        calculate();
      }
    }
  } else if (key == "Backspace" || key == "Delete") {
    // Backspace or delete
    if (board.remove()) {
      calculate();
    }
  }
}

function cell_click(e) {
  const split = e.target.id.split("_");

  switch (split[0]) {
    case "cell":
      {
        const y = parseInt(split[1]);
        const x = parseInt(split[2]);

        if (board.toggle(y, x)) {
          calculate();
        }
      }
      break;
  }
}

function calculate() {
  // Get the board state
  const board_flat = board.get_board();
  const board_rows = chop_flat_array(board_flat, 2 * 5);

  // Iterate rows
  for (let y = 0; y < 6; y++) {
    const row = board_rows[y];
    const cells = chop_flat_array(row, 2);

    // Iterate cells
    for (let x = 0; x < 5; x++) {
      const cell = cells[x];

      const type = cell[0];
      const letter = cell[1];

      // Upate the cell
      const cell_elem = document.getElementById(`cell_${y}_${x}`);

      switch (type) {
        case 0:
          cell_elem.className = "space space_none";
          cell_elem.innerText = "";
          break;
        case 1:
          cell_elem.className = "space space_incorrect";
          cell_elem.innerText = String.fromCharCode(letter);
          break;
        case 2:
          cell_elem.className = "space space_half_correct";
          cell_elem.innerText = String.fromCharCode(letter);
          break;
        case 3:
          cell_elem.className = "space space_correct";
          cell_elem.innerText = String.fromCharCode(letter);
          break;
      }
    }
  }

  // Calculate the words
  let calc_start = perf_now();

  const words = board.calculate();

  perf_end("calculate", calc_start);

  // Update the count div
  const count_div = document.getElementById("count");

  count_div.innerText = words ? `${words} words found` : "";

  // Update the words div
  const words_div = document.getElementById("words");
  words_div.innerHTML = "";

  if (words) {
    let words_start = perf_now();

    for (let i = 0; i < words; i++) {
      let word = board.get_word(i);

      const word_div = document.createElement("div");
      word_div.innerText = word;
      words_div.appendChild(word_div);
    }

    perf_end("words_div", words_start);
  }
}

function chop_flat_array(array, stride) {
  const result = [];

  for (let i = 0; i < array.length; i += stride) {
    result.push(array.subarray(i, i + stride));
  }

  return result;
}

// Loading spinner

function spinner_show(visible) {
  if (visible != spinner) {
    const elem = document.getElementById("loading");

    if (visible) {
      elem.style.display = "";
      document.body.style.overflow = "hidden";
    } else {
      elem.style.display = "none";
      document.body.style.overflow = "";
    }

    spinner = visible;
  }
}
