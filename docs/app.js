// Cache-busting: bump this alongside the ?v=N on the <script> tag in
// index.html whenever this file or the wasm build changes, otherwise
// browsers/GitHub Pages' CDN can keep serving a stale cached copy.
// NOTE: static `import` specifiers must be string literals (can't use a
// template/variable here) -- bump this literally, and bump ASSET_VERSION
// below (used for the .wasm fetch) and index.html's ?v=N to match.
import init, { SpreadsheetApp } from "./pkg/wasm_app.js?v=3";
const ASSET_VERSION = 3;

// Size of the visible window into the sheet. The sheet itself is much
// bigger than this (same sparse region/block structure as the CLI) --
// these are just how many cells we render on screen at once, and can be
// changed at runtime via the "New sheet" control.
let ROWS = 100;
let COLS = 100;

function colName(col) {
  let n = col;
  let s = "";
  // eslint-disable-next-line no-constant-condition
  while (true) {
    s = String.fromCharCode(65 + (n % 26)) + s;
    if (n < 26) break;
    n = Math.floor(n / 26) - 1;
  }
  return s;
}

function cellName(row, col) {
  return `${colName(col)}${row + 1}`;
}

let app;
let selected = { row: 0, col: 0 }; // local (visible-window) coordinates
let rowOffset = 0; // absolute sheet row shown at the top of the window
let colOffset = 0; // absolute sheet col shown at the left of the window

const gridEl = document.getElementById("grid");
const formulaBar = document.getElementById("formulaBar");
const commandBar = document.getElementById("commandBar");
const cellLabel = document.getElementById("cellLabel");
const statusEl = document.getElementById("status");
const newRowsInput = document.getElementById("newRows");
const newColsInput = document.getElementById("newCols");

// Grid is rebuilt from scratch whenever ROWS/COLS change (e.g. "New Sheet").
// Cell clicks are handled with ONE delegated listener on the table instead
// of one listener per cell -- matters once this is 100x100 = 10,000 cells.
function buildGrid() {
  let html = "<thead><tr><th></th>";
  for (let c = 0; c < COLS; c++) html += `<th></th>`;
  html += "</tr></thead><tbody>";
  for (let r = 0; r < ROWS; r++) {
    html += `<tr><td class="rowhead"></td>`;
    for (let c = 0; c < COLS; c++) {
      html += `<td class="cell" data-row="${r}" data-col="${c}" id="cell-${r}-${c}"></td>`;
    }
    html += "</tr>";
  }
  html += "</tbody>";
  gridEl.innerHTML = html;
}

gridEl.addEventListener("click", (e) => {
  const td = e.target.closest("td.cell");
  if (!td) return;
  selectCell(parseInt(td.dataset.row, 10), parseInt(td.dataset.col, 10));
});

// Column/row headers depend on the current viewport offset, so they're
// redrawn separately from the grid skeleton (which only needs rebuilding
// when ROWS/COLS themselves change).
function updateHeaders() {
  const headThs = gridEl.querySelectorAll("thead th");
  for (let c = 0; c < COLS; c++) {
    headThs[c + 1].textContent = colName(colOffset + c);
  }
  const rowheads = gridEl.querySelectorAll("td.rowhead");
  rowheads.forEach((td, r) => {
    td.textContent = rowOffset + r + 1;
  });
}

function renderAll() {
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const val = app.get_cell(rowOffset + r, colOffset + c);
      const td = document.getElementById(`cell-${r}-${c}`);
      td.textContent = val;
      td.classList.toggle("err", val === "ERR" || val === "DIV0");
    }
  }
}

function selectCell(localRow, localCol) {
  const prev = document.getElementById(`cell-${selected.row}-${selected.col}`);
  if (prev) prev.classList.remove("selected");
  selected = { row: localRow, col: localCol };
  const cur = document.getElementById(`cell-${localRow}-${localCol}`);
  if (cur) cur.classList.add("selected");

  const absRow = rowOffset + localRow;
  const absCol = colOffset + localCol;
  cellLabel.textContent = cellName(absRow, absCol);
  formulaBar.value = app.get_formula(absRow, absCol) || app.get_cell(absRow, absCol);
  formulaBar.focus();
}

function setStatus(status, ms) {
  const ok = status === "ok";
  statusEl.innerHTML = `<span class="${ok ? "ok" : "bad"}">${status}</span> -- ${ms.toFixed(3)} ms`;
}

// Called after every engine command (formula bar, raw command, undo, redo):
// the viewport may have moved (W/A/S/D/SCROLL_TO), so re-sync it from the
// engine's own cursor position rather than assuming it didn't change.
function afterCommand() {
  rowOffset = app.cursor_row();
  colOffset = app.cursor_col();
  updateHeaders();
  renderAll();
  setStatus(app.status(), app.last_time_ms());
}

function commit() {
  const raw = formulaBar.value.trim();
  const absRow = rowOffset + selected.row;
  const absCol = colOffset + selected.col;
  const name = cellName(absRow, absCol);
  // Accept both "23" and Excel-style "=A1+B2" input.
  const rhs = raw.startsWith("=") ? raw.slice(1) : raw;
  const cmd = rhs === "" ? `${name}=0` : `${name}=${rhs}`;
  app.execute(cmd);
  afterCommand();
  // Move selection down a row, like Excel/Sheets does on Enter.
  selectCell(Math.min(selected.row + 1, ROWS - 1), selected.col);
}

function runRawCommand() {
  const raw = commandBar.value.trim();
  if (raw === "") return;
  app.execute(raw);
  afterCommand();
  // Selection may now be pointing at different cell content if the
  // viewport moved -- refresh the formula bar/label for it.
  selectCell(selected.row, selected.col);
  commandBar.value = "";
  commandBar.focus();
}

function newSheet() {
  const requestedRows = parseInt(newRowsInput.value, 10);
  const requestedCols = parseInt(newColsInput.value, 10);
  ROWS = Number.isFinite(requestedRows) && requestedRows > 0 ? requestedRows : 100;
  COLS = Number.isFinite(requestedCols) && requestedCols > 0 ? requestedCols : 100;

  app = new SpreadsheetApp();
  rowOffset = 0;
  colOffset = 0;
  selected = { row: 0, col: 0 };
  buildGrid();
  updateHeaders();
  renderAll();
  selectCell(0, 0);
  setStatus("new sheet", 0);
}

formulaBar.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    e.preventDefault();
    commit();
  }
});

commandBar.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    e.preventDefault();
    runRawCommand();
  }
});

document.getElementById("runCommandBtn").addEventListener("click", runRawCommand);
document.getElementById("newSheetBtn").addEventListener("click", newSheet);

document.getElementById("undoBtn").addEventListener("click", () => {
  app.execute("U");
  afterCommand();
  selectCell(selected.row, selected.col);
});

document.getElementById("redoBtn").addEventListener("click", () => {
  app.execute("R");
  afterCommand();
  selectCell(selected.row, selected.col);
});

document.getElementById("clearBtn").addEventListener("click", () => {
  app = new SpreadsheetApp();
  afterCommand();
  selected = { row: 0, col: 0 };
  selectCell(0, 0);
  setStatus("cleared", 0);
});

// Sidebar example chips: fill the relevant input and focus it (doesn't
// auto-run, so formula examples work correctly once you've picked a cell).
document.querySelectorAll(".example").forEach((btn) => {
  btn.addEventListener("click", () => {
    const target = btn.dataset.target === "formula" ? formulaBar : commandBar;
    target.value = btn.dataset.value;
    target.focus();
  });
});

async function main() {
  await init({ module_or_path: `./pkg/wasm_app_bg.wasm?v=${ASSET_VERSION}` });
  app = new SpreadsheetApp();
  buildGrid();
  updateHeaders();
  renderAll();
  selectCell(0, 0);
  document.getElementById("loading").style.display = "none";
  document.getElementById("app").style.display = "block";
}

main();
