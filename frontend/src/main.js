import "./style.css";
import { invoke } from "@tauri-apps/api/core";

const speedElem = document.getElementById("speed");
const statusElem = document.getElementById("status");
const countdownElem = document.getElementById("countdown");
const countdownRow = document.getElementById("countdown-row");
const downloadingElem = document.getElementById("downloading");

let monitoring = true;
const toggleBtn = document.getElementById("toggle-btn");

toggleBtn.addEventListener("click", () => {
  monitoring = !monitoring;
  if (monitoring) {
    toggleBtn.textContent = "Stop Monitoring";
    toggleBtn.classList.remove("stopped");
    invoke("start_monitoring");
  } else {
    toggleBtn.textContent = "Start Monitoring";
    toggleBtn.classList.add("stopped");
    invoke("stop_monitoring");
    // Reset UI elements after stopping
    speedElem.textContent = "0.00 KB/s";
    statusElem.textContent = "Idle";
    countdownElem.textContent = "";
    countdownRow.style.display = "none";
    downloadingElem.className = "downloading-value false";
    downloadingElem.textContent = "No";
    downloadingElem.title = "No";
  }
});

window.__TAURI__.event.listen("net-data", (event) => {
  if (!monitoring) return;
  const data = event.payload;
  if (data && typeof data === "object") {
    // Format speed
    const speed =
      typeof data.average_kbps === "number"
        ? data.average_kbps.toFixed(2)
        : "0.00";
    speedElem.textContent = `${speed} KB/s`;

    // Status logic
    let statusText = "Idle";
    if (["1", "2", "3", "4", "5", "6", "7"].includes(data.status)) {
      countdownElem.textContent = data.status;
      countdownRow.style.display = "flex";
      statusText = "Sleeping soon";
    } else {
      countdownElem.textContent = "";
      countdownRow.style.display = "none";
      statusText = data.status;
    }
    statusElem.textContent = statusText;

    // Downloading logic
    let downloadingText = "No";
    let downloadingClass = "false";
    if (data.is_downloading === "true") {
      downloadingText = "Yes";
      downloadingClass = "true";
    } else if (data.is_downloading === "false") {
      downloadingText = "No";
      downloadingClass = "false";
    }
    downloadingElem.className = `downloading-value ${downloadingClass}`;
    downloadingElem.textContent = downloadingText;
    downloadingElem.title = downloadingText;
  }
});
