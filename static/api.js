const API_URL = "/api";

const API = {
  async getShifts() {
    const response = await fetch(`${API_URL}/shifts`);
    if (!response.ok) throw new Error("Failed to fetch shifts");
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
    // Sanitize the data before sending
    const sanitizedData = {};

    for (const [key, value] of Object.entries(data)) {
      if (key === "notes") {
        // For notes, explicitly send null if empty, or the trimmed value
        if (typeof value === "string") {
          const trimmed = value.trim();
          sanitizedData[key] = trimmed.length === 0 ? null : trimmed;
        } else {
          sanitizedData[key] = value;
        }
      } else {
        sanitizedData[key] = value;
      }
    }

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
};
