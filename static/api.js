const API_URL = "http://localhost:3000/api";

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
    const response = await fetch(`${API_URL}/shifts/${shiftId}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
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
