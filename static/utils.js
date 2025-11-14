const USER_TIMEZONE = Intl.DateTimeFormat().resolvedOptions().timeZone;

// ===== DATE/TIME FORMATTING =====
function formatDateTime(utcDateString) {
  if (!utcDateString) return "";

  try {
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

function formatDateForInput(date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

// ===== TIMEZONE CONVERSIONS =====
function getLocalDateRange(period, customRange = { start: null, end: null }) {
  // Returns { start: UTC ISO string, end: UTC ISO string }

  if (period === "custom") {
    if (!customRange.start || !customRange.end) {
      return null;
    }

    // Custom range dates are local Date objects from input fields
    const startDate = new Date(customRange.start);
    startDate.setHours(0, 0, 0, 0);

    const endDate = new Date(customRange.end);
    endDate.setHours(23, 59, 59, 999);

    return {
      start: localToUTC(startDate),
      end: localToUTC(endDate),
    };
  } else if (period === "month") {
    // Get current month in user's local timezone
    const now = new Date();

    // Get user's local timezone current date components
    const localNow = new Date(
      now.toLocaleString("en-US", { timeZone: USER_TIMEZONE }),
    );

    const year = localNow.getFullYear();
    const month = localNow.getMonth();

    // Create start and end dates (these will be in local browser time initially)
    const startDate = new Date(year, month, 1, 0, 0, 0, 0);
    const endDate = new Date(year, month + 1, 0, 23, 59, 59, 999);

    return {
      start: localToUTC(startDate),
      end: localToUTC(endDate),
    };
  }

  // For "all" period, return null (no range needed)
  return null;
}

function localToUTC(localDate) {
  // Take a Date object whose components represent local user timezone time
  // and convert to UTC ISO string

  const year = localDate.getFullYear();
  const month = localDate.getMonth();
  const day = localDate.getDate();
  const hours = localDate.getHours();
  const minutes = localDate.getMinutes();
  const seconds = localDate.getSeconds();
  const ms = localDate.getMilliseconds();

  // Format as a string that can be interpreted in user's local timezone
  const dateString = `${year}-${String(month + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}T${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}.${String(ms).padStart(3, "0")}`;

  // Parse this string as if it's in user's local timezone by using toLocaleString
  // First, create a UTC date with these components
  const utcDate = new Date(
    Date.UTC(year, month, day, hours, minutes, seconds, ms),
  );

  // Get the formatted string in user's local timezone
  const localString = utcDate.toLocaleString("en-US", {
    timeZone: USER_TIMEZONE,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  });

  // Calculate the offset by comparing the UTC interpretation vs local interpretation
  const [datePart, timePart] = localString.split(", ");
  const [mo, da, yr] = datePart.split("/");
  const [hr, min, sec] = timePart.split(":");

  const localInterpretation = new Date(Date.UTC(yr, mo - 1, da, hr, min, sec));
  const offset = utcDate.getTime() - localInterpretation.getTime();

  // Apply the offset to get the correct UTC time
  const correctUTC = new Date(utcDate.getTime() - offset);

  return correctUTC.toISOString();
}

// ===== NUMBER FORMATTING =====
function formatMoney(value) {
  return parseFloat(value || 0).toFixed(2);
}

function formatHours(value) {
  return parseFloat(value || 0).toFixed(2);
}

// ===== UTILITY FUNCTIONS =====
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
