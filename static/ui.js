const UI = {
  showToast(message, type = "info") {
    const toast = document.getElementById("toast");
    toast.textContent = message;
    toast.className = `toast show ${type}`;

    setTimeout(() => {
      toast.className = "toast";
    }, 3000);
  },

  showLoading() {
    document.getElementById("loadingOverlay").classList.remove("hidden");
  },

  hideLoading() {
    document.getElementById("loadingOverlay").classList.add("hidden");
  },

  updateStats(shifts) {
    // Filter to only completed shifts (those with end_time)
    const completedShifts = shifts.filter((s) => s.end_time);

    const totalEarnings = completedShifts.reduce(
      (sum, s) => sum + parseFloat(s.day_total || 0),
      0,
    );
    const totalHours = completedShifts.reduce(
      (sum, s) => sum + parseFloat(s.hours_worked || 0),
      0,
    );
    const totalMiles = completedShifts.reduce(
      (sum, s) => sum + parseFloat(s.miles_driven || 0),
      0,
    );
    const avgRate = totalHours > 0 ? totalEarnings / totalHours : 0;

    // Remove loading class and add animation class
    const statValues = document.querySelectorAll(".stat-value");
    statValues.forEach((el) => {
      el.classList.remove("loading");
      el.classList.add("updating");
      setTimeout(() => el.classList.remove("updating"), 300);
    });

    document.getElementById("statTotalEarnings").textContent =
      `$${formatMoney(totalEarnings)}`;
    document.getElementById("statTotalHours").textContent =
      formatHours(totalHours);
    document.getElementById("statAvgRate").textContent =
      `$${formatMoney(avgRate)}`;
    document.getElementById("statTotalMiles").textContent =
      Math.round(totalMiles);
  },

  updateActiveShiftBanner(shift) {
    const banner = document.getElementById("activeShiftBanner");
    const startSection = document.getElementById("startShiftSection");

    if (shift) {
      banner.classList.remove("hidden");
      startSection.style.display = "none";

      const startTime = formatDateTime(shift.start_time);
      document.getElementById("shiftInfo").textContent =
        `Started at ${startTime} | Odometer: ${shift.odometer_start}`;
    } else {
      banner.classList.add("hidden");
      startSection.style.display = "flex";
    }
  },

  renderShifts(shifts, searchTerm = "") {
    const tbody = document.getElementById("shiftsBody");

    // Apply search filtering on the already backend-filtered shifts
    const filtered = shifts.filter((shift) => {
      if (!searchTerm) return true;
      const search = searchTerm.toLowerCase();
      return (
        shift.id.toString().includes(search) ||
        (shift.notes && shift.notes.toLowerCase().includes(search)) ||
        formatDateTime(shift.start_time).toLowerCase().includes(search) ||
        formatDateTime(shift.end_time).toLowerCase().includes(search)
      );
    });

    if (filtered.length === 0) {
      tbody.innerHTML = `
                <tr class="empty-state">
                    <td colspan="12">
                        <div class="empty-content">
                            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
                                <line x1="16" y1="2" x2="16" y2="6"></line>
                                <line x1="8" y1="2" x2="8" y2="6"></line>
                                <line x1="3" y1="10" x2="21" y2="10"></line>
                            </svg>
                            <p>${searchTerm ? "No shifts found" : "No shifts recorded yet"}</p>
                            <small>${searchTerm ? "Try a different search term" : "Start your first shift to begin tracking"}</small>
                        </div>
                    </td>
                </tr>
            `;
      return;
    }

    tbody.innerHTML = filtered
      .map(
        (shift) => `
            <tr data-shift-id="${shift.id}">
                <td>${formatDateTime(shift.start_time)}</td>
                <td>${formatDateTime(shift.end_time)}</td>
                <td class="calculated">${formatHours(shift.hours_worked)}</td>
                <td contenteditable="true" data-field="odometer_start" data-id="${shift.id}">${shift.odometer_start}</td>
                <td contenteditable="true" data-field="odometer_end" data-id="${shift.id}">${shift.odometer_end || ""}</td>
                <td class="calculated">${shift.miles_driven || ""}</td>
                <td class="money editable-money" contenteditable="true" data-field="earnings" data-id="${shift.id}">${formatMoney(shift.earnings)}</td>
                <td class="money editable-money" contenteditable="true" data-field="tips" data-id="${shift.id}">${formatMoney(shift.tips)}</td>
                <td class="money editable-money" contenteditable="true" data-field="gas_cost" data-id="${shift.id}">${formatMoney(shift.gas_cost)}</td>
                <td class="money calculated">$${formatMoney(shift.day_total)}</td>
                <td class="money calculated">$${shift.hourly_pay ? formatMoney(shift.hourly_pay) : ""}</td>
                <td class="notes-cell" contenteditable="true" data-field="notes" data-id="${shift.id}" title="${shift.notes || ""}">${shift.notes || ""}</td>
            </tr>
        `,
      )
      .join("");

    this.attachCellEditListeners();
  },

  attachCellEditListeners() {
    document.querySelectorAll('td[contenteditable="true"]').forEach((cell) => {
      cell.addEventListener("blur", handleCellEdit);
      cell.addEventListener("keydown", (e) => {
        if (e.key === "Enter" && !e.shiftKey) {
          e.preventDefault();
          cell.blur();
        }
      });
    });
  },

  setupTableSorting() {
    document.querySelectorAll("th[data-sort]").forEach((th) => {
      th.addEventListener("click", () => {
        const sortField = th.dataset.sort;
        const currentSort = th.classList.contains("sort-asc")
          ? "asc"
          : th.classList.contains("sort-desc")
            ? "desc"
            : null;

        document.querySelectorAll("th").forEach((header) => {
          header.classList.remove("sort-asc", "sort-desc");
        });

        if (currentSort === "asc") {
          th.classList.add("sort-desc");
          sortTable(sortField, "desc");
        } else {
          th.classList.add("sort-asc");
          sortTable(sortField, "asc");
        }
      });
    });
  },

  openModal() {
    const modal = document.getElementById("endShiftModal");
    modal.classList.add("show");
    document.body.style.overflow = "hidden";
  },

  closeModal() {
    const modal = document.getElementById("endShiftModal");
    modal.classList.remove("show");
    document.body.style.overflow = "";

    document.getElementById("endOdo").value = "";
    document.getElementById("earnings").value = "0";
    document.getElementById("tips").value = "0";
    document.getElementById("gasCost").value = "0";
    document.getElementById("notes").value = "";
  },
};
