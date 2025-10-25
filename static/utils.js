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

function formatMoney(value) {
  return parseFloat(value || 0).toFixed(2);
}

function formatHours(value) {
  return value ? parseFloat(value).toFixed(2) : "";
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
