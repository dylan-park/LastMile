let activeShift = null;
let allShifts = [];
let statsPeriod = "month"; // Default to monthly view

async function loadShifts() {
  try {
    allShifts = await API.getShifts();
    UI.updateStats(allShifts, statsPeriod);
    const searchTerm = document.getElementById("searchInput").value;
    UI.renderShifts(allShifts, searchTerm);
  } catch (error) {
    console.error("Error loading shifts:", error);
    UI.showToast("Failed to load shifts", "error");
  }
}

async function checkActiveShift() {
  try {
    activeShift = await API.getActiveShift();
    UI.updateActiveShiftBanner(activeShift);
  } catch (error) {
    console.error("Error checking active shift:", error);
  }
}

async function handleStartShift() {
  const odoStart = document.getElementById("startOdo").value;

  if (!odoStart) {
    UI.showToast("Please enter starting odometer reading", "error");
    return;
  }

  try {
    UI.showLoading();
    await API.startShift(parseInt(odoStart));
    document.getElementById("startOdo").value = "";
    await checkActiveShift();
    await loadShifts();
    UI.showToast("Shift started successfully", "success");
  } catch (error) {
    if (error.message === "CONFLICT") {
      UI.showToast("There is already an active shift", "error");
    } else {
      UI.showToast("Failed to start shift", "error");
    }
  } finally {
    UI.hideLoading();
  }
}

async function handleEndShift() {
  const endOdo = document.getElementById("endOdo").value;

  if (!endOdo) {
    UI.showToast("Please enter ending odometer reading", "error");
    return;
  }

  const earnings = parseFloat(document.getElementById("earnings").value) || 0;
  const tips = parseFloat(document.getElementById("tips").value) || 0;
  const gasCost = parseFloat(document.getElementById("gasCost").value) || 0;
  const notes = document.getElementById("notes").value.trim() || null;

  try {
    UI.showLoading();
    await API.endShift(activeShift.id, {
      odometer_end: parseInt(endOdo),
      earnings,
      tips,
      gas_cost: gasCost,
      notes,
    });
    UI.closeModal();
    await checkActiveShift();
    await loadShifts();
    UI.showToast("Shift ended successfully", "success");
  } catch (error) {
    UI.showToast("Failed to end shift", "error");
  } finally {
    UI.hideLoading();
  }
}

async function handleCellEdit(e) {
  const cell = e.target;
  const field = cell.dataset.field;
  const id = cell.dataset.id;
  let value = cell.textContent.trim();

  if (["odometer_start", "odometer_end"].includes(field)) {
    value = value ? parseInt(value) : null;
  } else if (["earnings", "tips", "gas_cost"].includes(field)) {
    value = value ? parseFloat(value) : 0;
  } else if (field === "notes") {
    value = value.length === 0 ? "" : value;
  }

  try {
    const payload = {};
    payload[field] = value;

    await API.updateShift(id, payload);
    await loadShifts();
  } catch (error) {
    console.error("Error updating shift:", error);
    UI.showToast("Failed to update shift", "error");
    await loadShifts();
  }
}

async function handleExportCSV() {
  try {
    UI.showLoading();
    const blob = await API.exportCSV();
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "uber_eats_shifts.csv";
    document.body.appendChild(a);
    a.click();
    window.URL.revokeObjectURL(url);
    document.body.removeChild(a);
    UI.showToast("CSV exported successfully", "success");
  } catch (error) {
    console.error("Error exporting CSV:", error);
    UI.showToast("Failed to export CSV", "error");
  } finally {
    UI.hideLoading();
  }
}

function sortTable(field, direction) {
  const sorted = [...allShifts].sort((a, b) => {
    let aVal = a[field];
    let bVal = b[field];

    if (field === "start_time" || field === "end_time") {
      aVal = aVal ? new Date(aVal) : new Date(0);
      bVal = bVal ? new Date(bVal) : new Date(0);
    } else if (typeof aVal === "string") {
      aVal = aVal.toLowerCase();
      bVal = bVal ? bVal.toLowerCase() : "";
    } else {
      aVal = parseFloat(aVal) || 0;
      bVal = parseFloat(bVal) || 0;
    }

    if (direction === "asc") {
      return aVal > bVal ? 1 : -1;
    } else {
      return aVal < bVal ? 1 : -1;
    }
  });

  const searchTerm = document.getElementById("searchInput").value;
  UI.renderShifts(sorted, searchTerm);
}

function toggleTheme() {
  document.documentElement.classList.toggle("dark-mode");
  const isDark = document.documentElement.classList.contains("dark-mode");
  localStorage.setItem("theme", isDark ? "dark" : "light");
  updateThemeButton();
}

function updateThemeButton() {
  const isDark = document.documentElement.classList.contains("dark-mode");
  const sunIcon = document.querySelector(".sun-icon");
  const moonIcon = document.querySelector(".moon-icon");

  if (isDark) {
    sunIcon.classList.add("hidden");
    moonIcon.classList.remove("hidden");
  } else {
    sunIcon.classList.remove("hidden");
    moonIcon.classList.add("hidden");
  }
}

function handleStatsPeriodToggle(period) {
  statsPeriod = period;

  // Update toggle buttons
  document.querySelectorAll(".toggle-option").forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.period === period);
  });

  // Update stats with animation
  UI.updateStats(allShifts, statsPeriod);
}

const debouncedSearch = debounce((searchTerm) => {
  UI.renderShifts(allShifts, searchTerm);
}, 300);

// Initialize
document.addEventListener("DOMContentLoaded", () => {
  // Update theme button to match current state
  updateThemeButton();

  document.getElementById("themeToggle").addEventListener("click", toggleTheme);
  document
    .getElementById("startShiftBtn")
    .addEventListener("click", handleStartShift);
  document
    .getElementById("endShiftBtn")
    .addEventListener("click", () => UI.openModal());
  document
    .getElementById("exportBtn")
    .addEventListener("click", handleExportCSV);
  document
    .getElementById("modalClose")
    .addEventListener("click", () => UI.closeModal());
  document
    .getElementById("modalCancel")
    .addEventListener("click", () => UI.closeModal());
  document
    .getElementById("modalSubmit")
    .addEventListener("click", handleEndShift);

  document.getElementById("searchInput").addEventListener("input", (e) => {
    debouncedSearch(e.target.value);
  });

  document
    .querySelector(".modal-backdrop")
    .addEventListener("click", () => UI.closeModal());

  document.getElementById("startOdo").addEventListener("keypress", (e) => {
    if (e.key === "Enter") {
      handleStartShift();
    }
  });

  // Stats period toggle listeners
  document.querySelectorAll(".toggle-option").forEach((btn) => {
    btn.addEventListener("click", () => {
      handleStatsPeriodToggle(btn.dataset.period);
    });
  });

  UI.setupTableSorting();

  checkActiveShift();
  loadShifts();
});
