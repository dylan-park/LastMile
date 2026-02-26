const UI = {
  // ===== NOTIFICATIONS =====
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

  // ===== STATS =====
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

  // ===== SHIFTS RENDERING =====
  renderShifts(shifts, searchTerm = "") {
    const tbody = document.getElementById("shiftsBody");

    // Apply search filtering
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

    const shiftsContainer = tbody.closest(".table-container");

    if (filtered.length === 0) {
      tbody.innerHTML = this._getEmptyState(
        13,
        searchTerm ? "No shifts found" : "No shifts recorded yet",
        searchTerm
          ? "Try a different search term"
          : "Start your first shift to begin tracking",
        "calendar",
      );
      if (shiftsContainer) shiftsContainer.classList.remove("has-data");
      return;
    }

    if (shiftsContainer) shiftsContainer.classList.add("has-data");

    tbody.innerHTML = filtered
      .map(
        (shift) => `
            <tr data-shift-id="${shift.id}">
                <td class="datetime-cell" data-field="start_time" data-id="${shift.id}">${formatDateTime(shift.start_time)}</td>
                <td class="datetime-cell" data-field="end_time" data-id="${shift.id}">${formatDateTime(shift.end_time)}</td>
                <td class="calculated">${formatHours(shift.hours_worked)}</td>
                <td contenteditable="true" data-field="odometer_start" data-id="${shift.id}">${shift.odometer_start}</td>
                <td contenteditable="true" data-field="odometer_end" data-id="${shift.id}">${shift.odometer_end || ""}</td>
                <td class="calculated">${shift.miles_driven || ""}</td>
                <td class="money editable-money" contenteditable="true" data-field="fare" data-id="${shift.id}">${formatMoney(shift.fare)}</td>
                <td class="money editable-money" contenteditable="true" data-field="tips" data-id="${shift.id}">${formatMoney(shift.tips)}</td>
                <td class="money editable-money" contenteditable="true" data-field="gas_cost" data-id="${shift.id}">${formatMoney(shift.gas_cost)}</td>
                <td class="money calculated">$${formatMoney(shift.day_total)}</td>
                <td class="money calculated">$${shift.hourly_pay ? formatMoney(shift.hourly_pay) : ""}</td>
                <td class="notes-cell" contenteditable="true" data-field="notes" data-id="${shift.id}" title="${shift.notes || ""}">${shift.notes || ""}</td>
                <td class="action-cell">
                    <button class="btn-delete-small" onclick="handleDeleteShift('${shift.id}')" title="Delete">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <polyline points="3 6 5 6 21 6"></polyline>
                            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                        </svg>
                    </button>
                </td>
            </tr>
        `,
      )
      .join("");
  },

  // ===== MAINTENANCE RENDERING =====
  renderMaintenanceItems(items, searchTerm = "", requiredIds = new Set()) {
    const tbody = document.getElementById("maintenanceBody");

    // Apply search filtering
    const filtered = items.filter((item) => {
      if (!searchTerm) return true;
      const search = searchTerm.toLowerCase();
      return (
        item.name.toLowerCase().includes(search) ||
        (item.notes && item.notes.toLowerCase().includes(search))
      );
    });

    const maintenanceContainer = tbody.closest(".table-container");

    if (filtered.length === 0) {
      tbody.innerHTML = this._getEmptyState(
        7,
        searchTerm ? "No maintenance items found" : "No maintenance items yet",
        searchTerm
          ? "Try a different search term"
          : "Click 'Add Maintenance Item' to create one",
        "tool",
      );
      if (maintenanceContainer)
        maintenanceContainer.classList.remove("has-data");
      return;
    }

    if (maintenanceContainer) maintenanceContainer.classList.add("has-data");

    tbody.innerHTML = filtered
      .map((item) => {
        const isRequired = requiredIds.has(item.id);
        const rowClass = isRequired ? "maintenance-required" : "";
        const enabledText = item.enabled ? "Yes" : "No";
        const enabledClass = item.enabled ? "enabled-yes" : "enabled-no";

        return `
            <tr data-maintenance-id="${item.id}" class="${rowClass}">
                <td contenteditable="true" data-field="name" data-id="${item.id}">${item.name}</td>
                <td contenteditable="true" data-field="mileage_interval" data-id="${item.id}">${item.mileage_interval}</td>
                <td contenteditable="true" data-field="last_service_mileage" data-id="${item.id}">${item.last_service_mileage}</td>
                <td class="calculated">${item.remaining_mileage}</td>
                <td class="enabled-cell ${enabledClass}" role="button" tabindex="0" data-field="enabled" data-id="${item.id}">${enabledText}</td>
                <td class="notes-cell" contenteditable="true" data-field="notes" data-id="${item.id}" title="${item.notes || ""}">${item.notes || ""}</td>
                <td class="action-cell">
                    <button class="btn-delete-small" onclick="handleDeleteMaintenance('${item.id}')" title="Delete">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <polyline points="3 6 5 6 21 6"></polyline>
                            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                        </svg>
                    </button>
                </td>
            </tr>
        `;
      })
      .join("");
  },

  // ===== MODALS =====
  openModal(modalId) {
    const modal = document.getElementById(modalId);
    modal.classList.add("show");
    document.body.style.overflow = "hidden";
  },

  closeModal(modalId) {
    const modal = document.getElementById(modalId);
    modal.classList.remove("show");
    document.body.style.overflow = "";

    // Clear form fields based on modal type
    if (modalId === "endShiftModal") {
      document.getElementById("endOdo").value = "";
      document.getElementById("fare").value = "0";
      document.getElementById("tips").value = "0";
      document.getElementById("gasCost").value = "0";
      document.getElementById("notes").value = "";
    } else if (modalId === "maintenanceModal") {
      document.getElementById("maintenanceName").value = "";
      document.getElementById("maintenanceMileageInterval").value = "";
      document.getElementById("maintenanceLastService").value = "0";
      document.getElementById("maintenanceEnabled").checked = true;
      document.getElementById("maintenanceNotes").value = "";
    }
  },

  // ===== TABLE SORTING =====
  setupTableSorting(type, sortCallback) {
    const tableId = type === "shifts" ? "shiftsTable" : "maintenanceTable";
    document.querySelectorAll(`#${tableId} th[data-sort]`).forEach((th) => {
      th.addEventListener("click", () => {
        const sortField = th.dataset.sort;
        const currentSort = th.classList.contains("sort-asc")
          ? "asc"
          : th.classList.contains("sort-desc")
            ? "desc"
            : null;

        document.querySelectorAll(`#${tableId} th`).forEach((header) => {
          header.classList.remove("sort-asc", "sort-desc");
        });

        if (currentSort === "asc") {
          th.classList.add("sort-desc");
          sortCallback(sortField, "desc");
        } else {
          th.classList.add("sort-asc");
          sortCallback(sortField, "asc");
        }
      });
    });
  },

  // ===== CELL EDITING =====
  onCellEdit(type, callback) {
    const tableId = type === "shifts" ? "shiftsBody" : "maintenanceBody";

    // Attach listeners via event delegation
    const tbody = document.getElementById(tableId);

    tbody.addEventListener(
      "blur",
      (e) => {
        if (e.target.contentEditable === "true") {
          callback(e);
        }
      },
      true,
    );

    tbody.addEventListener("keydown", (e) => {
      if (
        e.target.contentEditable === "true" &&
        e.key === "Enter" &&
        !e.shiftKey
      ) {
        e.preventDefault();
        e.target.blur();
      }
    });

    // For enabled toggle cells (maintenance only)
    if (type === "maintenance") {
      tbody.addEventListener("click", (e) => {
        if (e.target.classList.contains("enabled-cell")) {
          callback(e);
        }
      });

      tbody.addEventListener("keydown", (e) => {
        if (
          e.target.classList.contains("enabled-cell") &&
          (e.key === "Enter" || e.key === " ")
        ) {
          e.preventDefault();
          callback(e);
        }
      });
    }
  },

  // ===== DEMO BANNER =====
  showDemoBanner() {
    // Check if already exists to avoid duplicates
    if (document.getElementById("demoBanner")) return;

    const banner = document.createElement("div");
    banner.id = "demoBanner";
    banner.className = "demo-banner";
    banner.innerHTML = `
      <div class="banner-content">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="0.5">
        <path d="M12,14a1.25,1.25,0,1,0,1.25,1.25A1.25,1.25,0,0,0,12,14Zm0-1.5a1,1,0,0,0,1-1v-3a1,1,0,0,0-2,0v3A1,1,0,0,0,12,12.5ZM12,2A10,10,0,1,0,22,12,10.01114,10.01114,0,0,0,12,2Zm0,18a8,8,0,1,1,8-8A8.00917,8.00917,0,0,1,12,20Z"/>
        </svg>
        <div class="banner-text">
          <strong>This is a demo environment.</strong>
          <span>Changes won’t be saved and reset each session.</span>
        </div>
      </div>
      <button class="banner-close" onclick="UI.hideDemoBanner()" aria-label="Dismiss demo notice">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    `;

    // Insert as first child of container (after header but inside .container)
    const container = document.querySelector(".container");
    const viewToggle = document.querySelector(".view-toggle-container");
    const header = document.querySelector(".header");
    document.body.insertAdjacentElement("afterbegin", banner);
    setTimeout(() => this.hideDemoBanner(), 8000); // auto-close after 8 seconds
  },

  hideDemoBanner() {
    const banner = document.getElementById("demoBanner");
    if (banner) {
      banner.classList.add("dismissing");
      setTimeout(() => banner.remove(), 300);
    }
  },

  // ===== TABLE SCROLL ENHANCEMENTS =====
  initTableScrollFade(wrapper) {
    if (!wrapper) return;
    const container = wrapper.closest(".table-container");
    if (!container) return;

    const firstTh = wrapper.querySelector("thead tr th:first-child");

    const update = () => {
      // Re-measure sticky column width dynamically
      if (firstTh) {
        container.style.setProperty(
          "--sticky-col-width",
          `${firstTh.offsetWidth}px`,
        );
      }

      const scrollLeft = wrapper.scrollLeft;
      const isScrollable = wrapper.scrollWidth > wrapper.clientWidth + 2;
      const atEnd = scrollLeft + wrapper.clientWidth >= wrapper.scrollWidth - 2;
      const atStart = scrollLeft <= 2;

      // Fades
      container.classList.toggle("can-scroll-right", isScrollable);
      container.classList.toggle("scrolled-end", atEnd);
      container.classList.toggle("scrolled-start", !atStart);

      // Drag cursor
      wrapper.classList.toggle("is-scrollable", isScrollable);
    };

    wrapper.addEventListener("scroll", update, { passive: true });

    // Observe both the wrapper (for container resize) and the sticky
    // column itself (for any reflow that changes its width independently)
    const ro = new ResizeObserver(update);
    ro.observe(wrapper);
    if (firstTh) ro.observe(firstTh);

    update();

    // ----- Drag-to-scroll (desktop) -----
    // Touch devices handle their own scrolling natively; this targets mouse only.
    // Uses a movement threshold to distinguish a drag (scroll) from a click (edit).
    const DRAG_THRESHOLD = 5;

    let isDragging = false;
    let hasCrossedThreshold = false;
    let dragStartX = 0;
    let dragStartScrollLeft = 0;

    wrapper.addEventListener("mousedown", (e) => {
      if (e.button !== 0) return;
      if (e.target.closest("button, a")) return;

      // Don't interfere if the table doesn't need scrolling
      if (!wrapper.classList.contains("is-scrollable")) return;

      // Don't interfere if the user is clicking inside an already-focused
      // editable cell — let them select text normally
      const active = document.activeElement;
      if (active && active.isContentEditable && active.contains(e.target))
        return;

      isDragging = true;
      hasCrossedThreshold = false;
      dragStartX = e.clientX;
      dragStartScrollLeft = wrapper.scrollLeft;
      // Don't preventDefault here — we need focus events to still fire
      // so editable cells can activate normally if this turns out to be a click.
    });

    window.addEventListener("mousemove", (e) => {
      if (!isDragging) return;

      const delta = dragStartX - e.clientX;

      if (!hasCrossedThreshold) {
        // Not yet committed to a drag — check if we've moved enough
        if (Math.abs(delta) < DRAG_THRESHOLD) return;

        // Threshold crossed: this is a drag, not a click
        hasCrossedThreshold = true;
        wrapper.classList.add("is-dragging");
        document.body.style.userSelect = "none";
        // Clear any selection that formed during the pre-threshold movement
        window.getSelection()?.removeAllRanges();
      }

      wrapper.scrollLeft = dragStartScrollLeft + delta;
    });

    window.addEventListener("mouseup", () => {
      if (!isDragging) return;

      if (hasCrossedThreshold) {
        // Suppress the click that follows mouseup so it doesn't
        // accidentally focus/activate whichever cell the drag ended on.
        window.addEventListener("click", (e) => e.stopPropagation(), {
          capture: true,
          once: true,
        });
        wrapper.classList.remove("is-dragging");
        document.body.style.userSelect = "";
      }

      isDragging = false;
      hasCrossedThreshold = false;
    });

    window.addEventListener("mouseleave", () => {
      if (!isDragging) return;
      isDragging = false;
      hasCrossedThreshold = false;
      wrapper.classList.remove("is-dragging");
      document.body.style.userSelect = "";
    });
  },

  initAllTableScrollFades() {
    document.querySelectorAll(".table-wrapper").forEach((wrapper) => {
      this.initTableScrollFade(wrapper);
    });
  },

  // ===== PRIVATE HELPERS =====
  _getEmptyState(colspan, title, subtitle, iconType = "calendar") {
    const icons = {
      calendar: `<rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
                 <line x1="16" y1="2" x2="16" y2="6"></line>
                 <line x1="8" y1="2" x2="8" y2="6"></line>
                 <line x1="3" y1="10" x2="21" y2="10"></line>`,
      tool: `<path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"></path>`,
    };

    return `
      <tr class="empty-state">
        <td colspan="${colspan}">
          <div class="empty-content">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              ${icons[iconType]}
            </svg>
            <p>${title}</p>
            <small>${subtitle}</small>
          </div>
        </td>
      </tr>
    `;
  },
};
