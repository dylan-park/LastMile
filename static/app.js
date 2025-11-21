// ===== STATE MANAGEMENT =====
const state = {
  activeShift: null,
  allShifts: [],
  allMaintenanceItems: [],
  requiredMaintenanceIds: new Set(),
  requiredMaintenanceCount: 0,
  statsPeriod: "month", // "month", "all", or "custom"
  customDateRange: { start: null, end: null },
  currentView: "shifts", // "shifts" or "maintenance"
};

// ===== DATA LOADING =====
async function loadShifts() {
  try {
    // Determine which API endpoint to call based on the period
    if (state.statsPeriod === "all") {
      state.allShifts = await API.getShifts();
    } else {
      const dateRange = getLocalDateRange(
        state.statsPeriod,
        state.customDateRange,
      );

      if (!dateRange) {
        state.allShifts = await API.getShifts();
      } else {
        state.allShifts = await API.getShiftsByRange(
          dateRange.start,
          dateRange.end,
        );
      }
    }

    UI.updateStats(state.allShifts);

    const searchTerm = document.getElementById("searchInput").value;
    UI.renderShifts(state.allShifts, searchTerm);
  } catch (error) {
    console.error("Error loading shifts:", error);
    UI.showToast("Failed to load shifts", "error");
  }
}

async function checkActiveShift() {
  try {
    state.activeShift = await API.getActiveShift();
    UI.updateActiveShiftBanner(state.activeShift);
  } catch (error) {
    console.error("Error checking active shift:", error);
  }
}

async function loadMaintenanceItems() {
  try {
    state.allMaintenanceItems = await API.getMaintenanceItems();

    // Sort by mileage interval (ascending) by default
    state.allMaintenanceItems.sort(
      (a, b) => a.mileage_interval - b.mileage_interval,
    );

    const searchTerm = document.getElementById("maintenanceSearchInput").value;
    UI.renderMaintenanceItems(
      state.allMaintenanceItems,
      searchTerm,
      state.requiredMaintenanceIds,
    );
  } catch (error) {
    console.error("Error loading maintenance items:", error);
    UI.showToast("Failed to load maintenance items", "error");
  }
}

async function loadRequiredMaintenance() {
  try {
    const response = await API.getRequiredMaintenance();
    const requiredItems = response.required_maintenance_items || [];

    state.requiredMaintenanceCount = requiredItems.length;
    state.requiredMaintenanceIds = new Set(
      requiredItems.map((item) => item.id),
    );

    updateMaintenanceBadge();

    // Always re-render maintenance items with updated required status
    const searchTerm = document.getElementById("maintenanceSearchInput").value;
    UI.renderMaintenanceItems(
      state.allMaintenanceItems,
      searchTerm,
      state.requiredMaintenanceIds,
    );
  } catch (error) {
    console.error("Error loading required maintenance:", error);
  }
}

function updateMaintenanceBadge() {
  const badge = document.getElementById("maintenanceBadge");
  if (state.requiredMaintenanceCount > 0) {
    badge.textContent = state.requiredMaintenanceCount;
    badge.classList.remove("hidden");
  } else {
    badge.classList.add("hidden");
  }
}

// ===== SHIFT HANDLERS =====
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
  const notesValue = document.getElementById("notes").value.trim();
  const notes = notesValue.length === 0 ? null : notesValue;

  try {
    UI.showLoading();
    await API.endShift(state.activeShift.id, {
      odometer_end: parseInt(endOdo),
      earnings,
      tips,
      gas_cost: gasCost,
      notes,
    });
    UI.closeModal("endShiftModal");
    await checkActiveShift();
    await loadShifts();
    await loadMaintenanceItems(); // Refresh to get updated remaining_mileage
    await loadRequiredMaintenance();
    UI.showToast("Shift ended successfully", "success");
  } catch (error) {
    UI.showToast("Failed to end shift", "error");
  } finally {
    UI.hideLoading();
  }
}

async function handleShiftCellEdit(e) {
  const cell = e.target;
  const field = cell.dataset.field;
  const id = cell.dataset.id;
  let value = cell.textContent.trim();

  if (["odometer_start", "odometer_end"].includes(field)) {
    value = value ? parseInt(value) : null;
  } else if (["earnings", "tips", "gas_cost"].includes(field)) {
    value = value ? parseFloat(value) : 0;
  } else if (field === "notes") {
    value = value.length === 0 ? null : value;
  }

  try {
    const payload = {};
    payload[field] = value;

    await API.updateShift(id, payload);
    await loadShifts();

    // Update maintenance if odometer changed
    if (["odometer_start", "odometer_end"].includes(field)) {
      await loadMaintenanceItems(); // Refresh to get updated remaining_mileage
      await loadRequiredMaintenance();
    }
  } catch (error) {
    console.error("Error updating shift:", error);
    UI.showToast("Failed to update shift", "error");
    await loadShifts();
  }
}

// ===== DATETIME EDITING =====
let currentDatetimeEdit = { shiftId: null, field: null, currentShift: null };

function handleDatetimeClick(e) {
  const cell = e.target.closest(".datetime-cell");
  if (!cell) return;

  const shiftId = cell.dataset.id;
  const field = cell.dataset.field;

  // Find the shift data
  const shift = state.allShifts.find((s) => s.id === shiftId);
  if (!shift) return;

  // Store current edit context
  currentDatetimeEdit = { shiftId, field, currentShift: shift };

  // Get the current value
  const currentValue = shift[field];
  if (!currentValue) {
    UI.showToast("Cannot edit empty time", "error");
    return;
  }

  // Set modal title
  const title =
    field === "start_time" ? "Edit Start Time" : "Edit End Time";
  document.getElementById("datetimeModalTitle").textContent = title;

  // Convert UTC to local datetime-local format and populate input
  const localDatetime = utcToDatetimeLocal(currentValue);
  document.getElementById("datetimeInput").value = localDatetime;

  // Open modal
  UI.openModal("editDatetimeModal");
}

async function handleDatetimeSubmit() {
  const input = document.getElementById("datetimeInput");
  const localDatetimeValue = input.value;

  if (!localDatetimeValue) {
    UI.showToast("Please select a date and time", "error");
    return;
  }

  const { shiftId, field, currentShift } = currentDatetimeEdit;

  // Convert datetime-local value to Date object
  const localDate = new Date(localDatetimeValue);

  // Convert local time to UTC ISO string
  const utcIsoString = localToUTC(localDate);

  // Validate: if editing end_time, it must be after start_time
  if (field === "end_time") {
    const startTime = new Date(currentShift.start_time);
    const endTime = new Date(utcIsoString);

    if (endTime <= startTime) {
      UI.showToast("End time must be after start time", "error");
      return;
    }
  }

  // Validate: if editing start_time, it must be before end_time (if exists)
  if (field === "start_time" && currentShift.end_time) {
    const startTime = new Date(utcIsoString);
    const endTime = new Date(currentShift.end_time);

    if (startTime >= endTime) {
      UI.showToast("Start time must be before end time", "error");
      return;
    }
  }

  try {
    UI.showLoading();

    const payload = {};
    payload[field] = utcIsoString;

    await API.updateShift(shiftId, payload);
    await loadShifts();

    UI.closeModal("editDatetimeModal");
    UI.showToast("Time updated successfully", "success");
  } catch (error) {
    console.error("Error updating datetime:", error);
    UI.showToast("Failed to update time", "error");
  } finally {
    UI.hideLoading();
  }
}


async function handleDeleteShift(id) {
  if (!confirm("Are you sure you want to delete this shift?")) {
    return;
  }

  try {
    UI.showLoading();
    await API.deleteShift(id);
    await loadShifts();
    await loadRequiredMaintenance();
    UI.showToast("Shift deleted successfully", "success");
  } catch (error) {
    console.error("Error deleting shift:", error);
    UI.showToast("Failed to delete shift", "error");
  } finally {
    UI.hideLoading();
  }
}

async function handleExportCSV() {
  try {
    UI.showLoading();
    const blob = await API.exportCSV();
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "lastmile_shifts.csv";
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

// ===== MAINTENANCE HANDLERS =====
async function handleCreateMaintenance() {
  const name = document.getElementById("maintenanceName").value.trim();
  const mileageInterval = parseInt(
    document.getElementById("maintenanceMileageInterval").value,
  );
  const lastServiceMileage =
    parseInt(document.getElementById("maintenanceLastService").value) || 0;
  const enabled = document.getElementById("maintenanceEnabled").checked;
  const notesValue = document.getElementById("maintenanceNotes").value.trim();
  const notes = notesValue.length === 0 ? null : notesValue;

  if (!name || !mileageInterval) {
    UI.showToast("Please enter name and mileage interval", "error");
    return;
  }

  if (mileageInterval <= 0) {
    UI.showToast("Mileage interval must be positive", "error");
    return;
  }

  try {
    UI.showLoading();
    await API.createMaintenanceItem({
      name,
      mileage_interval: mileageInterval,
      last_service_mileage: lastServiceMileage,
      enabled,
      notes,
    });
    UI.closeModal("maintenanceModal");
    await loadMaintenanceItems();
    await loadRequiredMaintenance();
    UI.showToast("Maintenance item created successfully", "success");
  } catch (error) {
    console.error("Error creating maintenance item:", error);
    UI.showToast("Failed to create maintenance item", "error");
  } finally {
    UI.hideLoading();
  }
}

async function handleMaintenanceCellEdit(e) {
  const cell = e.target;
  const field = cell.dataset.field;
  const id = cell.dataset.id;
  let value = cell.textContent.trim();

  if (field === "mileage_interval" || field === "last_service_mileage") {
    value = value ? parseInt(value) : 0;
    if (value < 0) {
      UI.showToast("Value cannot be negative", "error");
      await loadMaintenanceItems();
      return;
    }
  } else if (field === "enabled") {
    // Toggle enabled state
    const item = state.allMaintenanceItems.find((m) => m.id === id);
    value = !item.enabled;
  } else if (field === "notes") {
    value = value.length === 0 ? null : value;
  }

  try {
    const payload = {};
    payload[field] = value;

    await API.updateMaintenanceItem(id, payload);
    await loadMaintenanceItems();
    await loadRequiredMaintenance();
  } catch (error) {
    console.error("Error updating maintenance item:", error);
    UI.showToast("Failed to update maintenance item", "error");
    await loadMaintenanceItems();
  }
}

async function handleDeleteMaintenance(id) {
  if (!confirm("Are you sure you want to delete this maintenance item?")) {
    return;
  }

  try {
    UI.showLoading();
    await API.deleteMaintenanceItem(id);
    await loadMaintenanceItems();
    await loadRequiredMaintenance();
    UI.showToast("Maintenance item deleted successfully", "success");
  } catch (error) {
    console.error("Error deleting maintenance item:", error);
    UI.showToast("Failed to delete maintenance item", "error");
  } finally {
    UI.hideLoading();
  }
}

// ===== SORTING =====
function sortTable(type, field, direction) {
  const items = type === "shifts" ? state.allShifts : state.allMaintenanceItems;

  const sorted = [...items].sort((a, b) => {
    let aVal = a[field];
    let bVal = b[field];

    if (field === "start_time" || field === "end_time") {
      aVal = aVal ? new Date(aVal) : new Date(0);
      bVal = bVal ? new Date(bVal) : new Date(0);
    } else if (field === "enabled") {
      aVal = aVal ? 1 : 0;
      bVal = bVal ? 1 : 0;
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

  if (type === "shifts") {
    const searchTerm = document.getElementById("searchInput").value;
    UI.renderShifts(sorted, searchTerm);
  } else {
    const searchTerm = document.getElementById("maintenanceSearchInput").value;
    UI.renderMaintenanceItems(sorted, searchTerm, state.requiredMaintenanceIds);
  }
}

// ===== VIEW & PERIOD CONTROLS =====
function switchView(view) {
  state.currentView = view;

  document.querySelectorAll(".view-toggle-option").forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.view === view);
  });

  const shiftsView = document.getElementById("shiftsView");
  const maintenanceView = document.getElementById("maintenanceView");

  if (view === "shifts") {
    shiftsView.style.display = "block";
    maintenanceView.style.display = "none";
  } else {
    shiftsView.style.display = "none";
    maintenanceView.style.display = "block";
  }
}

async function handleStatsPeriodToggle(period) {
  state.statsPeriod = period;

  document.querySelectorAll(".toggle-option").forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.period === period);
  });

  const customDateRangeEl = document.getElementById("customDateRange");
  if (period === "custom") {
    customDateRangeEl.classList.remove("hidden");

    // Set default dates to current month if not already set
    if (!state.customDateRange.start && !state.customDateRange.end) {
      const now = new Date();
      const firstDay = new Date(now.getFullYear(), now.getMonth(), 1);
      const lastDay = new Date(now.getFullYear(), now.getMonth() + 1, 0);

      document.getElementById("startDate").value = formatDateForInput(firstDay);
      document.getElementById("endDate").value = formatDateForInput(lastDay);

      state.customDateRange.start = firstDay;
      state.customDateRange.end = lastDay;
    }
  } else {
    customDateRangeEl.classList.add("hidden");
  }

  await loadShifts();
}

async function handleCustomDateChange() {
  const startDateInput = document.getElementById("startDate").value;
  const endDateInput = document.getElementById("endDate").value;

  state.customDateRange.start = startDateInput
    ? new Date(startDateInput + "T00:00:00")
    : null;
  state.customDateRange.end = endDateInput
    ? new Date(endDateInput + "T23:59:59")
    : null;

  await loadShifts();
}

// ===== THEME =====
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

// ===== INITIALIZATION =====
document.addEventListener("DOMContentLoaded", () => {
  updateThemeButton();

  // Theme toggle
  document.getElementById("themeToggle").addEventListener("click", toggleTheme);

  // Shift controls
  document
    .getElementById("startShiftBtn")
    .addEventListener("click", handleStartShift);
  document
    .getElementById("endShiftBtn")
    .addEventListener("click", () => UI.openModal("endShiftModal"));
  document
    .getElementById("exportBtn")
    .addEventListener("click", handleExportCSV);

  // Shift modal controls
  document
    .getElementById("modalClose")
    .addEventListener("click", () => UI.closeModal("endShiftModal"));
  document
    .getElementById("modalCancel")
    .addEventListener("click", () => UI.closeModal("endShiftModal"));
  document
    .getElementById("modalSubmit")
    .addEventListener("click", handleEndShift);
  document
    .querySelector(".modal-backdrop")
    .addEventListener("click", () => UI.closeModal("endShiftModal"));

  // Datetime modal controls
  document
    .getElementById("datetimeModalClose")
    .addEventListener("click", () => UI.closeModal("editDatetimeModal"));
  document
    .getElementById("datetimeModalCancel")
    .addEventListener("click", () => UI.closeModal("editDatetimeModal"));
  document
    .getElementById("datetimeModalSubmit")
    .addEventListener("click", handleDatetimeSubmit);

  // Datetime cell click handler (event delegation)
  document
    .getElementById("shiftsBody")
    .addEventListener("click", handleDatetimeClick);

  // Cell editing
  document.getElementById("searchInput").addEventListener("input", (e) => {
    debounce(() => UI.renderShifts(state.allShifts, e.target.value), 300)();
  });

  // Enter key on start odometer
  document.getElementById("startOdo").addEventListener("keypress", (e) => {
    if (e.key === "Enter") {
      handleStartShift();
    }
  });

  // Stats period toggle
  document.querySelectorAll(".toggle-option").forEach((btn) => {
    btn.addEventListener("click", () => {
      handleStatsPeriodToggle(btn.dataset.period);
    });
  });

  // Custom date range
  document
    .getElementById("startDate")
    .addEventListener("change", handleCustomDateChange);
  document
    .getElementById("endDate")
    .addEventListener("change", handleCustomDateChange);

  // View toggle
  document.querySelectorAll(".view-toggle-option").forEach((btn) => {
    btn.addEventListener("click", () => {
      switchView(btn.dataset.view);
    });
  });

  // Maintenance controls
  document
    .getElementById("addMaintenanceBtn")
    .addEventListener("click", () => UI.openModal("maintenanceModal"));
  document
    .getElementById("maintenanceModalClose")
    .addEventListener("click", () => UI.closeModal("maintenanceModal"));
  document
    .getElementById("maintenanceModalCancel")
    .addEventListener("click", () => UI.closeModal("maintenanceModal"));
  document
    .getElementById("maintenanceModalSubmit")
    .addEventListener("click", handleCreateMaintenance);
  document
    .querySelector(".maintenance-modal-backdrop")
    .addEventListener("click", () => UI.closeModal("maintenanceModal"));

  // Maintenance search
  document
    .getElementById("maintenanceSearchInput")
    .addEventListener("input", (e) => {
      debounce(
        () =>
          UI.renderMaintenanceItems(
            state.allMaintenanceItems,
            e.target.value,
            state.requiredMaintenanceIds,
          ),
        300,
      )();
    });

  // Table sorting
  UI.setupTableSorting("shifts", (field, direction) =>
    sortTable("shifts", field, direction),
  );
  UI.setupTableSorting("maintenance", (field, direction) =>
    sortTable("maintenance", field, direction),
  );

  // Cell editing
  UI.onCellEdit("shifts", handleShiftCellEdit);
  UI.onCellEdit("maintenance", handleMaintenanceCellEdit);

  // Load initial data
  Promise.all([
    checkActiveShift(),
    loadShifts(),
    loadRequiredMaintenance(),
    loadMaintenanceItems(),
  ]).catch((error) => {
    console.error("Error during initial load:", error);
    UI.showToast("Failed to load initial data", "error");
  });
});
