const USER_TIMEZONE = "America/Chicago";

function formatDateTime(utcDateString) {
  if (!utcDateString) return "";

  try {
    // Handle SurrealDB datetime format which already includes 'Z'
    // Format: 2025-10-27T01:11:47.739709100Z
    const date = new Date(utcDateString);

    return new Intl.DateTimeFormat("en-US", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      timeZone: USER_TIMEZONE,
      hour12: true,
    }).format(date);
  } catch (error) {
    console.error("Error formatting date:", error);
    return utcDateString;
  }
}

function parseUTCToLocal(utcDateString) {
  if (!utcDateString) return null;

  try {
    // SurrealDB datetime already includes 'Z', so parse directly
    const utcDate = new Date(utcDateString);

    // Convert to local timezone string, then parse back to get local Date
    const localString = new Intl.DateTimeFormat("en-US", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      timeZone: USER_TIMEZONE,
      hour12: false,
    }).format(utcDate);

    // Parse the formatted string back to a Date object in local context
    // Format will be: MM/DD/YYYY, HH:mm:ss
    const [datePart, timePart] = localString.split(", ");
    const [month, day, year] = datePart.split("/");
    const [hour, minute, second] = timePart.split(":");

    return new Date(year, month - 1, day, hour, minute, second);
  } catch (error) {
    console.error("Error parsing UTC to local:", error);
    return new Date(utcDateString);
  }
}

function formatMoney(value) {
  return parseFloat(value || 0).toFixed(2);
}

function formatHours(value) {
  return parseFloat(value || 0).toFixed(2);
}

function debounce(func, wait) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}
