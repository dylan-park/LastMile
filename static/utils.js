const USER_TIMEZONE = "America/Chicago";

function formatDateTime(utcDateString) {
  if (!utcDateString) return "";

  try {
    const date = new Date(utcDateString + "Z");
    return new Intl.DateTimeFormat("en-US", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      timeZone: USER_TIMEZONE,
    }).format(date);
  } catch (error) {
    console.error("Error formatting date:", error);
    return utcDateString;
  }
}

function parseUTCToLocal(utcDateString) {
  if (!utcDateString) return null;

  try {
    // Add 'Z' to indicate UTC, then convert to local Date object
    const utcDate = new Date(utcDateString + "Z");

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
