const API_URL = "/api";

const API = {
  // ===== SHIFTS =====
  async getShifts() {
    const response = await fetch(`${API_URL}/shifts`);
    if (!response.ok) throw new Error("Failed to fetch shifts");
    return await response.json();
  },

  async getShiftsByRange(startUTC, endUTC) {
    const params = new URLSearchParams({
      start: startUTC,
      end: endUTC,
    });
    const response = await fetch(`${API_URL}/shifts/range?${params}`);
    if (!response.ok) throw new Error("Failed to fetch shifts by range");
    return await response.json();
  },

  async getActiveShift() {
    const response = await fetch(`${API_URL}/shifts/active`);
    if (!response.ok) throw new Error("Failed to fetch active shift");
    return await response.json();
  },

  async startShift(odometerStart) {
    const response = await fetch(`${API_URL}/shifts/start`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ odometer_start: odometerStart }),
    });
    if (!response.ok) {
      if (response.status === 409) {
        throw new Error("CONFLICT");
      }
      throw new Error("Failed to start shift");
    }
    return await response.json();
  },

  async endShift(shiftId, data) {
    const response = await fetch(`${API_URL}/shifts/${shiftId}/end`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    });
    if (!response.ok) throw new Error("Failed to end shift");
    return await response.json();
  },

  async updateShift(shiftId, data) {
    const sanitizedData = sanitizeData(data);
    const response = await fetch(`${API_URL}/shifts/${shiftId}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(sanitizedData),
    });
    if (!response.ok) throw new Error("Failed to update shift");
    return await response.json();
  },

  async exportCSV() {
    const response = await fetch(`${API_URL}/shifts/export`);
    if (!response.ok) throw new Error("Failed to export CSV");
    return await response.blob();
  },

  // ===== MAINTENANCE =====
  async getMaintenanceItems() {
    const response = await fetch(`${API_URL}/maintenance`);
    if (!response.ok) throw new Error("Failed to fetch maintenance items");
    return await response.json();
  },

  async getRequiredMaintenance() {
    const response = await fetch(`${API_URL}/maintenance/calculate`);
    if (!response.ok)
      throw new Error("Failed to calculate required maintenance");
    return await response.json();
  },

  async createMaintenanceItem(data) {
    const response = await fetch(`${API_URL}/maintenance/create`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    });
    if (!response.ok) throw new Error("Failed to create maintenance item");
    return await response.json();
  },

  async updateMaintenanceItem(itemId, data) {
    const sanitizedData = sanitizeData(data);
    const response = await fetch(`${API_URL}/maintenance/${itemId}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(sanitizedData),
    });
    if (!response.ok) throw new Error("Failed to update maintenance item");
    return await response.json();
  },

  async deleteMaintenanceItem(itemId) {
    const response = await fetch(`${API_URL}/maintenance/${itemId}`, {
      method: "DELETE",
    });
    if (!response.ok) throw new Error("Failed to delete maintenance item");
    return await response.json();
  },
};

// ===== HELPER FUNCTIONS =====
function sanitizeData(data) {
  const sanitized = {};

  for (const [key, value] of Object.entries(data)) {
    if (key === "notes") {
      // For notes, explicitly send null if empty, or the trimmed value
      if (typeof value === "string") {
        const trimmed = value.trim();
        sanitized[key] = trimmed.length === 0 ? null : trimmed;
      } else {
        sanitized[key] = value;
      }
    } else {
      sanitized[key] = value;
    }
  }

  return sanitized;
}
