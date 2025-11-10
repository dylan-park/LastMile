let allMaintenanceItems = [];
let requiredMaintenanceCount = 0;
let requiredMaintenanceIds = new Set();

async function loadMaintenanceItems() {
  try {
    allMaintenanceItems = await API.getMaintenanceItems();

    // Sort by mileage interval (ascending) by default
    allMaintenanceItems.sort((a, b) => a.mileage_interval - b.mileage_interval);

    const searchTerm = document.getElementById("maintenanceSearchInput").value;
    UI.renderMaintenanceItems(
      allMaintenanceItems,
      searchTerm,
      requiredMaintenanceIds,
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

    requiredMaintenanceCount = requiredItems.length;
    requiredMaintenanceIds = new Set(requiredItems.map((item) => item.id));

    // Update badge
    updateMaintenanceBadge();

    // Re-render if we're on the maintenance view
    if (document.getElementById("maintenanceView").style.display !== "none") {
      const searchTerm = document.getElementById(
        "maintenanceSearchInput",
      ).value;
      UI.renderMaintenanceItems(
        allMaintenanceItems,
        searchTerm,
        requiredMaintenanceIds,
      );
    }
  } catch (error) {
    console.error("Error loading required maintenance:", error);
  }
}

function updateMaintenanceBadge() {
  const badge = document.getElementById("maintenanceBadge");
  if (requiredMaintenanceCount > 0) {
    badge.textContent = requiredMaintenanceCount;
    badge.classList.remove("hidden");
  } else {
    badge.classList.add("hidden");
  }
}

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
    UI.closeMaintenanceModal();
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
    const item = allMaintenanceItems.find((m) => m.id === id);
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

function sortMaintenanceTable(field, direction) {
  const sorted = [...allMaintenanceItems].sort((a, b) => {
    let aVal = a[field];
    let bVal = b[field];

    if (field === "enabled") {
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

  const searchTerm = document.getElementById("maintenanceSearchInput").value;
  UI.renderMaintenanceItems(sorted, searchTerm, requiredMaintenanceIds);
}

const debouncedMaintenanceSearch = debounce((searchTerm) => {
  UI.renderMaintenanceItems(
    allMaintenanceItems,
    searchTerm,
    requiredMaintenanceIds,
  );
}, 300);
