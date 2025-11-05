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

function convertChicagoToUTC(chicagoDate) {
  // Given a Date object where we want to treat the date/time components
  // as if they represent a time in Chicago timezone, return the UTC ISO string

  try {
    const year = chicagoDate.getFullYear();
    const month = chicagoDate.getMonth() + 1; // 1-12
    const day = chicagoDate.getDate();
    const hour = chicagoDate.getHours();
    const minute = chicagoDate.getMinutes();
    const second = chicagoDate.getSeconds();

    // Format as ISO string components
    const isoString = `${year}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}T${String(hour).padStart(2, "0")}:${String(minute).padStart(2, "0")}:${String(second).padStart(2, "0")}`;

    // Create a date string that can be parsed unambiguously
    // We'll use the toLocaleString approach to handle timezone conversion
    const monthNames = [
      "Jan",
      "Feb",
      "Mar",
      "Apr",
      "May",
      "Jun",
      "Jul",
      "Aug",
      "Sep",
      "Oct",
      "Nov",
      "Dec",
    ];
    const dateString = `${monthNames[month - 1]} ${day}, ${year} ${String(hour).padStart(2, "0")}:${String(minute).padStart(2, "0")}:${String(second).padStart(2, "0")} GMT-0600`;

    // Parse this date string - it will interpret as CST
    let parsedDate = new Date(dateString);

    // Check if we need to account for DST
    // We do this by creating a test date and checking Chicago timezone offset
    const testDate = new Date(Date.UTC(year, month - 1, day, 12, 0, 0));
    const chicagoTestString = testDate.toLocaleString("en-US", {
      timeZone: USER_TIMEZONE,
      hour12: false,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    });

    // Check if this date is in DST by comparing with a known standard time date
    const jan = new Date(Date.UTC(year, 0, 1, 12, 0, 0));
    const jul = new Date(Date.UTC(year, 6, 1, 12, 0, 0));

    const janOffset =
      new Date(jan.toLocaleString("en-US", { timeZone: "UTC" })).getTime() -
      new Date(
        jan.toLocaleString("en-US", { timeZone: USER_TIMEZONE }),
      ).getTime();
    const julOffset =
      new Date(jul.toLocaleString("en-US", { timeZone: "UTC" })).getTime() -
      new Date(
        jul.toLocaleString("en-US", { timeZone: USER_TIMEZONE }),
      ).getTime();
    const testOffset =
      new Date(
        testDate.toLocaleString("en-US", { timeZone: "UTC" }),
      ).getTime() -
      new Date(
        testDate.toLocaleString("en-US", { timeZone: USER_TIMEZONE }),
      ).getTime();

    // Determine if DST is in effect (offset is less in summer due to DST)
    const isDST = testOffset < Math.max(janOffset, julOffset);

    // CST is UTC-6, CDT is UTC-5
    const offsetHours = isDST ? 5 : 6;

    // Create the correct UTC date
    const utcDate = new Date(
      Date.UTC(year, month - 1, day, hour + offsetHours, minute, second),
    );

    return utcDate.toISOString();
  } catch (error) {
    console.error("Error converting Chicago to UTC:", error);
    return chicagoDate.toISOString();
  }
}

function getChicagoOffset(date) {
  // Get the offset in minutes for America/Chicago timezone at a specific date
  // This accounts for DST automatically

  const utcDate = new Date(date.toLocaleString("en-US", { timeZone: "UTC" }));
  const chicagoDate = new Date(
    date.toLocaleString("en-US", { timeZone: USER_TIMEZONE }),
  );

  return (chicagoDate.getTime() - utcDate.getTime()) / 60000;
}

function getChicagoDateRange(period, customRange = { start: null, end: null }) {
  // Returns { start: UTC ISO string, end: UTC ISO string }
  // based on the period and custom range provided

  if (period === "custom") {
    if (!customRange.start || !customRange.end) {
      return null;
    }

    // Custom range dates are already in Chicago timezone
    // Convert start to beginning of day (00:00:00) and end to end of day (23:59:59)
    const startDate = new Date(customRange.start);
    startDate.setHours(0, 0, 0, 0);

    const endDate = new Date(customRange.end);
    endDate.setHours(23, 59, 59, 999);

    return {
      start: convertChicagoToUTC(startDate),
      end: convertChicagoToUTC(endDate),
    };
  } else if (period === "month") {
    // Get current month in Chicago timezone
    const now = new Date();
    const chicagoNow = new Date(
      now.toLocaleString("en-US", { timeZone: USER_TIMEZONE }),
    );

    const year = chicagoNow.getFullYear();
    const month = chicagoNow.getMonth();

    // First day of month at 00:00:00
    const startDate = new Date(year, month, 1, 0, 0, 0, 0);

    // Last day of month at 23:59:59
    const endDate = new Date(year, month + 1, 0, 23, 59, 59, 999);

    return {
      start: convertChicagoToUTC(startDate),
      end: convertChicagoToUTC(endDate),
    };
  }

  // For "all" period, return null (no range needed)
  return null;
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
