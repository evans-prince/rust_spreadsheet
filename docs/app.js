import init, { SpreadsheetApp } from "./pkg/wasm_app.js";

const ROWS = 20;
const COLS = 10;

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
let selected = { row: 0, col: 0 };

const gridEl = document.getElementById("grid");
const formulaBar = document.getElementById("formulaBar");
const cellLabel = document.getElementById("cellLabel");
const statusEl = document.getElementById("status");

function buildGrid() {
  let html = "<thead><tr><th></th>";
  for (let c = 0; c < COLS; c++) html += `<th>${colName(c)}</th>`;
  html += "</tr></thead><tbody>";
  for (let r = 0; r < ROWS; r++) {
    html += `<tr><td class="rowhead">${r + 1}</td>`;
    for (let c = 0; c < COLS; c++) {
      html += `<td class="cell" data-row="${r}" data-col="${c}" id="cell-${r}-${c}"></td>`;
    }
    html += "</tr>";
  }
  html += "</tbody>";
  gridEl.innerHTML = html;

  gridEl.querySelectorAll("td.cell").forEach((td) => {
    td.addEventListener("click", () => {
      selectCell(parseInt(td.dataset.row, 10), parseInt(td.dataset.col, 10));
    });
  });
}

function renderAll() {
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const val = app.get_cell(r, c);
      const td = document.getElementById(`cell-${r}-${c}`);
      td.textContent = val;
      td.classList.toggle("err", val === "ERR" || val === "DIV0");
    }
  }
}

function selectCell(row, col) {
  const prev = document.getElementById(`cell-${selected.row}-${selected.col}`);
  if (prev) prev.classList.remove("selected");
  selected = { row, col };
  const cur = document.getElementById(`cell-${row}-${col}`);
  if (cur) cur.classList.add("selected");
  cellLabel.textContent = cellName(row, col);
  formulaBar.value = app.get_formula(row, col) || app.get_cell(row, col);
  formulaBar.focus();
}

function setStatus(status, ms) {
  const ok = status === "ok";
  statusEl.innerHTML = `<span class="${ok ? "ok" : "bad"}">${status}</span> -- ${ms.toFixed(3)} ms`;
}

function commit() {
  const raw = formulaBar.value.trim();
  const name = cellName(selected.row, selected.col);
  // Accept both "23" and Excel-style "=A1+B2" input.
  const rhs = raw.startsWith("=") ? raw.slice(1) : raw;
  const cmd = rhs === "" ? `${name}=0` : `${name}=${rhs}`;
  app.execute(cmd);
  renderAll();
  setStatus(app.status(), app.last_time_ms());
  // Move selection down a row, like Excel/Sheets does on Enter.
  selectCell(Math.min(selected.row + 1, ROWS - 1), selected.col);
}

formulaBar.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    e.preventDefault();
    commit();
  }
});

document.getElementById("undoBtn").addEventListener("click", () => {
  app.execute("U");
  renderAll();
  setStatus(app.status(), app.last_time_ms());
  selectCell(selected.row, selected.col);
});

document.getElementById("redoBtn").addEventListener("click", () => {
  app.execute("R");
  renderAll();
  setStatus(app.status(), app.last_time_ms());
  selectCell(selected.row, selected.col);
});

document.getElementById("clearBtn").addEventListener("click", () => {
  app = new SpreadsheetApp();
  renderAll();
  setStatus("cleared", 0);
  selectCell(0, 0);
});

async function main() {
  await init();
  app = new SpreadsheetApp();
  buildGrid();
  renderAll();
  selectCell(0, 0);
  document.getElementById("loading").style.display = "none";
  document.getElementById("app").style.display = "block";
}

main();
