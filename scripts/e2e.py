import os
import socket
import time

import pytest
import requests
from selenium import webdriver
from selenium.common.exceptions import (
    WebDriverException,
)
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.common.by import By
from selenium.webdriver.common.keys import Keys
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.support.ui import WebDriverWait

REMOTE_URL = "http://localhost:4444/wd/hub"
APP_HOST = (
    "host.docker.internal" if os.getenv("GITHUB_ACTIONS") == "true" else "127.0.0.1"
)
APP_URL = f"http://{APP_HOST}:3000"

API_HOST = "localhost" if os.getenv("GITHUB_ACTIONS") == "true" else APP_HOST
API_URL = f"http://{API_HOST}:3000/api"


def port_open(host, port, timeout=1.0):
    """Check whether a TCP port is open."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.settimeout(timeout)
        return s.connect_ex((host, port)) == 0


def create_driver():
    options = Options()
    # Uncomment for headless mode:
    # options.add_argument("--headless")
    options.add_argument("--no-sandbox")
    options.add_argument("--disable-dev-shm-usage")

    # 1. Try remote Selenium (GitHub Actions)
    if port_open("localhost", 4444):
        print("➡ Using remote Selenium Grid at :4444")
        try:
            return webdriver.Remote(command_executor=REMOTE_URL, options=options)
        except WebDriverException:
            print(
                "⚠ Remote Selenium found but failed to connect, falling back to local driver."
            )

    # 2. Fallback: local Chrome / ChromeDriver
    print("➡ Using local ChromeDriver")
    return webdriver.Chrome(options=options)


def wait_for_page_load(driver, timeout=10):
    """Wait for page to fully load."""
    WebDriverWait(driver, timeout).until(
        lambda d: d.execute_script("return document.readyState") == "complete"
    )


def teardown_database():
    """Clear all data from the database via API."""
    try:
        # Use localhost in GitHub Actions instead of host.docker.internal
        api_host = "localhost" if os.getenv("GITHUB_ACTIONS") == "true" else APP_HOST
        api_url = f"http://{api_host}:3000/api"

        response = requests.post(f"{api_url}/test/teardown", timeout=5)
        response.raise_for_status()
        print(f"✓ Database teardown: {response.json()}")
    except Exception as e:
        print(f"⚠ Database teardown failed: {e}")


# --- Pytest fixtures ---
@pytest.fixture(scope="function")
def driver():
    """Create a fresh driver instance for each test."""
    drv = create_driver()
    drv.implicitly_wait(2)  # Set implicit wait for element lookups
    yield drv
    drv.quit()


@pytest.fixture(autouse=True)
def clean_database():
    """Clean database before and after each test."""
    teardown_database()
    yield
    teardown_database()


# ============================================================================
# BASIC UI TESTS
# ============================================================================


def test_homepage_loads(driver):
    """Test that the homepage loads correctly."""
    driver.get(APP_URL)
    wait_for_page_load(driver)
    assert "LastMile" in driver.title

    # Check that main elements are present
    header = driver.find_element(By.TAG_NAME, "h1")
    assert "LastMile" in header.text


def test_view_toggle_exists(driver):
    """Test that view toggle between Shifts and Maintenance exists."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    shifts_btn = driver.find_element(By.CSS_SELECTOR, '[data-view="shifts"]')
    maintenance_btn = driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]')

    assert shifts_btn.is_displayed()
    assert maintenance_btn.is_displayed()
    assert "active" in shifts_btn.get_attribute("class")


def test_switch_to_maintenance_view(driver):
    """Test switching between Shifts and Maintenance views."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Switch to maintenance view
    maintenance_btn = driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]')
    maintenance_btn.click()

    # Wait for view to switch
    WebDriverWait(driver, 5).until(
        lambda d: "active"
        in d.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').get_attribute(
            "class"
        )
    )

    # Check that maintenance view is visible
    maintenance_view = driver.find_element(By.ID, "maintenanceView")
    assert maintenance_view.is_displayed()

    # Check that shifts view is hidden
    shifts_view = driver.find_element(By.ID, "shiftsView")
    assert shifts_view.get_attribute("style") == "display: none;"


def test_theme_toggle(driver):
    """Test theme toggle between light and dark mode."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Get initial theme state
    html = driver.find_element(By.TAG_NAME, "html")
    initial_classes = html.get_attribute("class") or ""

    # Click theme toggle
    theme_btn = driver.find_element(By.ID, "themeToggle")
    theme_btn.click()

    # Wait a moment for theme to change
    time.sleep(0.3)

    # Check that theme changed
    new_classes = html.get_attribute("class") or ""
    assert initial_classes != new_classes


def test_stats_period_toggle(driver):
    """Test stats period toggle (Month/All/Custom)."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Click "All Time" button
    all_btn = driver.find_element(By.CSS_SELECTOR, '[data-period="all"]')
    all_btn.click()

    # Wait for active state to change
    WebDriverWait(driver, 5).until(
        lambda d: "active"
        in d.find_element(By.CSS_SELECTOR, '[data-period="all"]').get_attribute("class")
    )

    # Check that "This Month" is no longer active
    month_btn = driver.find_element(By.CSS_SELECTOR, '[data-period="month"]')
    assert "active" not in month_btn.get_attribute("class")


def test_custom_date_range_shows(driver):
    """Test that custom date range inputs appear when Custom Range is selected."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    custom_range = driver.find_element(By.ID, "customDateRange")
    assert "hidden" in custom_range.get_attribute("class")

    # Click “Custom Range”
    driver.find_element(By.CSS_SELECTOR, '[data-period="custom"]').click()

    # Wait for the container to become visible
    WebDriverWait(driver, 5).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "customDateRange").get_attribute("class")
    )

    # Now wait for the actual inputs to be visible
    start_date = WebDriverWait(driver, 5).until(
        EC.visibility_of_element_located((By.ID, "startDate"))
    )

    end_date = WebDriverWait(driver, 5).until(
        EC.visibility_of_element_located((By.ID, "endDate"))
    )

    # Assertions (now guaranteed stable)
    assert start_date.is_displayed()
    assert end_date.is_displayed()


def test_app_version_display(driver):
    """Test that the application version is displayed in the footer."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Find the version span
    version_span = driver.find_element(By.ID, "appVersion")

    # Wait for the text to change from the default placeholder "..."
    # The version fetching happens asynchronously on load
    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "appVersion").text != "..."
    )

    version_text = version_span.text

    # Assert it looks like a version number (basic check)
    # It should not be empty and should not be just "..."
    assert version_text
    assert version_text != "..."

    # Check that it's contained within the footer text correctly
    footer = driver.find_element(By.CLASS_NAME, "app-footer")
    assert f"LastMile v{version_text}" in footer.text


# ============================================================================
# SHIFT WORKFLOW TESTS
# ============================================================================


def test_start_shift_basic(driver):
    driver.get(APP_URL)
    wait_for_page_load(driver)

    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    # Wait for banner to be visible
    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    # Wait for correct text to appear
    WebDriverWait(driver, 10).until(
        EC.text_to_be_present_in_element((By.ID, "activeShiftBanner"), "10000")
    )

    banner = driver.find_element(By.ID, "activeShiftBanner")
    assert "10000" in banner.text


def test_start_shift_validation_empty(driver):
    """Test that starting a shift without odometer shows error."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Try to start shift without entering odometer
    start_btn = driver.find_element(By.ID, "startShiftBtn")
    start_btn.click()

    # Wait for toast notification
    toast = WebDriverWait(driver, 5).until(lambda d: d.find_element(By.ID, "toast"))

    # Toast might take a moment to populate, so wait and retry if empty
    max_retries = 5
    for i in range(max_retries):
        if toast.text and "odometer" in toast.text.lower():
            break
        time.sleep(0.2)
        toast = driver.find_element(By.ID, "toast")

    assert toast.text, "Please enter starting odometer reading"
    assert "odometer" in toast.text.lower(), (
        f"Expected 'odometer' in toast but got: {toast.text}"
    )


def test_cannot_start_two_shifts(driver):
    """Test that starting a second shift while one is active fails."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Start first shift
    odo_input = driver.find_element(By.ID, "startOdo")
    odo_input.send_keys("10000")
    start_btn = driver.find_element(By.ID, "startShiftBtn")
    start_btn.click()

    # Wait for active shift banner
    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    # The start shift section should be hidden now
    start_section = driver.find_element(By.ID, "startShiftSection")
    assert start_section.get_attribute("style") == "display: none;"


def test_end_shift_opens_modal(driver):
    """Test that clicking End Shift opens the modal."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Start a shift first
    odo_input = driver.find_element(By.ID, "startOdo")
    odo_input.send_keys("10000")
    start_btn = driver.find_element(By.ID, "startShiftBtn")
    start_btn.click()

    # Wait for active shift banner
    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    # Click End Shift button
    end_shift_btn = driver.find_element(By.ID, "endShiftBtn")
    end_shift_btn.click()

    # Wait for modal to appear
    modal = WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "endShiftModal")
    )
    assert "show" in modal.get_attribute("class")


def test_end_shift_complete_workflow(driver):
    """Test complete shift workflow: start, end with data."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Start shift
    odo_input = driver.find_element(By.ID, "startOdo")
    odo_input.send_keys("10000")
    start_btn = driver.find_element(By.ID, "startShiftBtn")
    start_btn.click()

    # Wait for active shift banner
    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    # Click End Shift button
    end_shift_btn = driver.find_element(By.ID, "endShiftBtn")
    end_shift_btn.click()

    # Wait for modal
    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    # Fill in end shift data
    driver.find_element(By.ID, "endOdo").send_keys("10100")
    driver.find_element(By.ID, "earnings").clear()
    driver.find_element(By.ID, "earnings").send_keys("120.50")
    driver.find_element(By.ID, "tips").clear()
    driver.find_element(By.ID, "tips").send_keys("25.00")
    driver.find_element(By.ID, "gasCost").clear()
    driver.find_element(By.ID, "gasCost").send_keys("15.00")
    driver.find_element(By.ID, "notes").send_keys("Good shift today")

    # Submit
    driver.find_element(By.ID, "modalSubmit").click()

    # Wait for modal to close
    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    # Wait for shift to appear in table
    time.sleep(2)  # Give time for table to update

    # Check that shift appears in table with correct data
    table = driver.find_element(By.ID, "shiftsBody")
    assert "10000" in table.text
    assert "10100" in table.text
    assert "120.50" in table.text or "120.5" in table.text


def test_end_shift_validation_odometer(driver):
    """Test that ending shift with lower odometer shows error."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Start shift with odometer 10000
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    # Open end shift modal
    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    # Try to end with lower odometer
    driver.find_element(By.ID, "endOdo").send_keys("9999")
    driver.find_element(By.ID, "modalSubmit").click()

    # Wait for error toast
    time.sleep(1)
    WebDriverWait(driver, 5).until(
        lambda d: "show" in d.find_element(By.ID, "toast").get_attribute("class")
    )


def test_modal_cancel_closes(driver):
    """Test that cancel button closes the modal."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Start shift
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    # Open modal
    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    # Click cancel
    driver.find_element(By.ID, "modalCancel").click()

    # Wait for modal to close
    WebDriverWait(driver, 5).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )


def test_search_shifts(driver):
    """Test searching for shifts."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Create a shift first
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("10100")
    driver.find_element(By.ID, "notes").send_keys("Test shift notes")
    driver.find_element(By.ID, "modalSubmit").click()

    # Wait for modal to close
    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    time.sleep(2)  # Wait for table to update

    # Search for the shift
    search_input = driver.find_element(By.ID, "searchInput")
    search_input.send_keys("Test shift")

    time.sleep(1)  # Wait for debounce

    # Check that shift appears in results
    table = driver.find_element(By.ID, "shiftsBody")
    assert "Test shift" in table.text


# ============================================================================
# TABLE INTERACTION TESTS
# ============================================================================


def test_table_sorting(driver):
    """Test that clicking table headers sorts the table."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Create two shifts with different odometers
    for odo in ["10000", "20000"]:
        driver.find_element(By.ID, "startOdo").send_keys(odo)
        driver.find_element(By.ID, "startShiftBtn").click()

        WebDriverWait(driver, 10).until(
            lambda d: "hidden"
            not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
        )

        driver.find_element(By.ID, "endShiftBtn").click()

        WebDriverWait(driver, 5).until(
            lambda d: "show"
            in d.find_element(By.ID, "endShiftModal").get_attribute("class")
        )

        driver.find_element(By.ID, "endOdo").send_keys(str(int(odo) + 100))
        driver.find_element(By.ID, "modalSubmit").click()

        WebDriverWait(driver, 10).until(
            lambda d: "show"
            not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
        )

        time.sleep(2)

    # Click on a sortable header
    start_time_header = driver.find_element(By.CSS_SELECTOR, '[data-sort="start_time"]')
    start_time_header.click()

    # Check that sort class is applied
    time.sleep(0.5)
    assert "sort-" in start_time_header.get_attribute("class")


def test_delete_shift(driver):
    """Test deleting a shift from the table."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Create a shift
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("10100")
    driver.find_element(By.ID, "modalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    time.sleep(2)

    # Find and click delete button
    delete_btn = driver.find_element(By.CSS_SELECTOR, ".btn-delete-small")
    delete_btn.click()

    # Accept confirmation dialog
    alert = WebDriverWait(driver, 5).until(EC.alert_is_present())
    alert.accept()

    # Wait for shift to be removed
    time.sleep(2)

    # Check that table shows empty state
    table = driver.find_element(By.ID, "shiftsBody")
    assert "No shifts" in table.text or "Loading" in table.text


# ============================================================================
# MAINTENANCE TESTS
# ============================================================================


def test_add_maintenance_item(driver):
    """Test adding a maintenance item."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Switch to maintenance view
    driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').click()

    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceView").is_displayed()
    )

    # Click Add Maintenance Item
    driver.find_element(By.ID, "addMaintenanceBtn").click()

    # Wait for modal
    modal = WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceModal")
    )
    assert "show" in modal.get_attribute("class")

    # Fill in form
    driver.find_element(By.ID, "maintenanceName").send_keys("Oil Change")
    driver.find_element(By.ID, "maintenanceMileageInterval").send_keys("3000")
    driver.find_element(By.ID, "maintenanceLastService").clear()
    driver.find_element(By.ID, "maintenanceLastService").send_keys("10000")
    driver.find_element(By.ID, "maintenanceNotes").send_keys("Full synthetic oil")

    # Submit
    driver.find_element(By.ID, "maintenanceModalSubmit").click()

    # Wait for modal to close
    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    # Wait for item to appear in table
    time.sleep(2)

    # Check that item appears
    table = driver.find_element(By.ID, "maintenanceBody")
    assert "Oil Change" in table.text
    assert "3000" in table.text
    # Check remaining mileage (should be 3000 since last service was 10000 and current is assumed 0/less)
    # Note: If no shifts, latest mileage is 0. 0 < 10000, so remaining = interval = 3000
    assert "3000" in table.text


def test_maintenance_search(driver):
    """Test searching maintenance items."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Switch to maintenance view
    driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').click()

    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceView").is_displayed()
    )

    # Add a maintenance item first
    driver.find_element(By.ID, "addMaintenanceBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    driver.find_element(By.ID, "maintenanceName").send_keys("Tire Rotation")
    driver.find_element(By.ID, "maintenanceMileageInterval").send_keys("5000")
    driver.find_element(By.ID, "maintenanceModalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    time.sleep(2)

    # Search for the item
    search_input = driver.find_element(By.ID, "maintenanceSearchInput")
    search_input.send_keys("Tire")

    time.sleep(1)

    # Check that item appears
    table = driver.find_element(By.ID, "maintenanceBody")
    assert "Tire Rotation" in table.text


def test_maintenance_delete(driver):
    """Test deleting a maintenance item."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Switch to maintenance view
    driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').click()

    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceView").is_displayed()
    )

    # Add item
    driver.find_element(By.ID, "addMaintenanceBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    driver.find_element(By.ID, "maintenanceName").send_keys("Test Item")
    driver.find_element(By.ID, "maintenanceMileageInterval").send_keys("1000")
    driver.find_element(By.ID, "maintenanceModalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    time.sleep(2)

    # Delete item
    delete_btn = driver.find_element(
        By.CSS_SELECTOR, "#maintenanceBody .btn-delete-small"
    )
    delete_btn.click()

    # Accept confirmation
    alert = WebDriverWait(driver, 5).until(EC.alert_is_present())
    alert.accept()

    time.sleep(2)

    # Check that table shows empty state
    table = driver.find_element(By.ID, "maintenanceBody")
    assert "No maintenance" in table.text or "Loading" in table.text


def test_maintenance_required_badge(driver):
    """Test that maintenance badge shows when maintenance is required."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Add maintenance item
    driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').click()

    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceView").is_displayed()
    )

    driver.find_element(By.ID, "addMaintenanceBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    # Create maintenance item with last service at 10000, interval 100
    driver.find_element(By.ID, "maintenanceName").send_keys("Due Soon")
    driver.find_element(By.ID, "maintenanceMileageInterval").send_keys("100")
    driver.find_element(By.ID, "maintenanceLastService").clear()
    driver.find_element(By.ID, "maintenanceLastService").send_keys("10000")
    driver.find_element(By.ID, "maintenanceModalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    # Switch back to shifts
    driver.find_element(By.CSS_SELECTOR, '[data-view="shifts"]').click()

    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "shiftsView").is_displayed()
    )

    # Create shift with odometer that triggers maintenance
    driver.find_element(By.ID, "startOdo").send_keys("10100")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("10200")
    driver.find_element(By.ID, "modalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    time.sleep(3)  # Wait for maintenance calculation

    # Check that maintenance badge appears
    badge = driver.find_element(By.ID, "maintenanceBadge")
    assert "hidden" not in badge.get_attribute("class")
    assert "1" in badge.text

    # Verify remaining mileage is 0
    driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').click()
    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceView").is_displayed()
    )
    table = driver.find_element(By.ID, "maintenanceBody")
    # Item "Due Soon", Interval 100, Last Service 10000, Current 10200
    # Remaining: 100 - (10200 - 10000) = -100 -> clamped to 0
    row = table.find_element(By.XPATH, "//tr[contains(., 'Due Soon')]")
    assert "0" in row.text


def test_maintenance_toggle_enabled(driver):
    """Test toggling maintenance item enabled status."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Switch to maintenance view
    driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').click()

    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceView").is_displayed()
    )

    # Add item
    driver.find_element(By.ID, "addMaintenanceBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    driver.find_element(By.ID, "maintenanceName").send_keys("Toggle Test")
    driver.find_element(By.ID, "maintenanceMileageInterval").send_keys("1000")
    driver.find_element(By.ID, "maintenanceModalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    time.sleep(2)

    # Find enabled cell and click it
    enabled_cell = driver.find_element(By.CSS_SELECTOR, "#maintenanceBody .enabled-yes")
    initial_text = enabled_cell.text
    enabled_cell.click()

    time.sleep(2)

    # Check that status changed
    disabled_cell = driver.find_element(By.CSS_SELECTOR, "#maintenanceBody .enabled-no")
    new_text = disabled_cell.text
    assert initial_text != new_text


# ============================================================================
# STATS TESTS
# ============================================================================


def test_stats_update_after_shift(driver):
    """Test that stats update after completing a shift."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Get initial stats
    initial_earnings = driver.find_element(By.ID, "statTotalEarnings").text

    # Create a shift
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("10100")
    driver.find_element(By.ID, "earnings").clear()
    driver.find_element(By.ID, "earnings").send_keys("100.00")
    driver.find_element(By.ID, "tips").clear()
    driver.find_element(By.ID, "tips").send_keys("20.00")
    driver.find_element(By.ID, "modalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    time.sleep(2)

    # Check that stats updated
    new_earnings = driver.find_element(By.ID, "statTotalEarnings").text
    assert new_earnings != initial_earnings
    assert "120" in new_earnings  # 100 + 20


def test_stats_all_time_vs_month(driver):
    """Test that switching between All Time and This Month updates stats."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Create a shift
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("10100")
    driver.find_element(By.ID, "earnings").clear()
    driver.find_element(By.ID, "earnings").send_keys("100.00")
    driver.find_element(By.ID, "modalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    time.sleep(2)

    # Get month stats
    month_earnings = driver.find_element(By.ID, "statTotalEarnings").text

    # Switch to All Time
    driver.find_element(By.CSS_SELECTOR, '[data-period="all"]').click()

    time.sleep(2)

    # Get all time stats (should be same since we only have one shift)
    all_earnings = driver.find_element(By.ID, "statTotalEarnings").text
    assert month_earnings == all_earnings


# ============================================================================
# CSV EXPORT TEST
# ============================================================================


def test_csv_export_button_exists(driver):
    """Test that CSV export button exists and is clickable."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    export_btn = driver.find_element(By.ID, "exportBtn")
    assert export_btn.is_displayed()
    assert export_btn.is_enabled()


def test_csv_export_all_time(driver):
    """Test CSV export with 'All Time' filter - verify API call and response."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Create a test shift first (following existing pattern)
    odo_input = driver.find_element(By.ID, "startOdo")
    odo_input.send_keys("10000")
    start_btn = driver.find_element(By.ID, "startShiftBtn")
    start_btn.click()

    # Wait for active shift banner
    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    # Click End Shift button
    end_shift_btn = driver.find_element(By.ID, "endShiftBtn")
    end_shift_btn.click()

    # Wait for modal
    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    # Fill in end shift data
    driver.find_element(By.ID, "endOdo").send_keys("10100")
    driver.find_element(By.ID, "earnings").clear()
    driver.find_element(By.ID, "earnings").send_keys("100")
    driver.find_element(By.ID, "tips").clear()
    driver.find_element(By.ID, "tips").send_keys("20")
    driver.find_element(By.ID, "gasCost").clear()
    driver.find_element(By.ID, "gasCost").send_keys("15")
    driver.find_element(By.ID, "notes").send_keys("Test shift for CSV")

    # Submit
    driver.find_element(By.ID, "modalSubmit").click()

    # Wait for modal to close
    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )
    time.sleep(1)

    # Make API call to export CSV (simulating button click)
    response = requests.get(f"{API_URL}/shifts/export")
    assert response.status_code == 200
    assert response.headers["Content-Type"] == "text/csv"
    assert "attachment" in response.headers.get("Content-Disposition", "")

    # Verify CSV content
    csv_content = response.text
    lines = csv_content.strip().split("\n")

    # Should have header + at least 1 shift
    assert len(lines) >= 2

    # Verify header
    assert "ID,Start Time,End Time" in lines[0]
    assert "Odometer Start,Odometer End" in lines[0]
    assert "Earnings,Tips,Gas Cost" in lines[0]

    # Verify our test shift is in the CSV
    assert "Test shift for CSV" in csv_content
    assert "10000" in csv_content  # odometer start
    assert "10100" in csv_content  # odometer end


def test_csv_export_month_view(driver):
    """Test CSV export with 'This Month' filter - verify date range filtering."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Create first shift (will be kept in current month)
    odo_input = driver.find_element(By.ID, "startOdo")
    odo_input.send_keys("11000")
    start_btn = driver.find_element(By.ID, "startShiftBtn")
    start_btn.click()

    # Wait for active shift banner
    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    # Click End Shift button
    end_shift_btn = driver.find_element(By.ID, "endShiftBtn")
    end_shift_btn.click()

    # Wait for modal
    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    # Fill in end shift data
    driver.find_element(By.ID, "endOdo").send_keys("11100")
    driver.find_element(By.ID, "earnings").clear()
    driver.find_element(By.ID, "earnings").send_keys("50")
    driver.find_element(By.ID, "notes").send_keys("In-range shift")

    # Submit
    driver.find_element(By.ID, "modalSubmit").click()

    # Wait for modal to close
    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )
    time.sleep(1)

    # Create second shift (will be edited to be outside current month)
    odo_input = driver.find_element(By.ID, "startOdo")
    odo_input.send_keys("11100")
    start_btn = driver.find_element(By.ID, "startShiftBtn")
    start_btn.click()

    # Wait for active shift banner
    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    # Click End Shift button
    end_shift_btn = driver.find_element(By.ID, "endShiftBtn")
    end_shift_btn.click()

    # Wait for modal
    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    # Fill in end shift data
    driver.find_element(By.ID, "endOdo").send_keys("11200")
    driver.find_element(By.ID, "earnings").clear()
    driver.find_element(By.ID, "earnings").send_keys("75")
    driver.find_element(By.ID, "notes").send_keys("Out-of-range shift")

    # Submit
    driver.find_element(By.ID, "modalSubmit").click()

    # Wait for modal to close
    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )
    time.sleep(2)

    # Now edit the second shift's start time to be 2 months ago
    # Find start time cell for the specific shift
    start_time_cell = driver.find_element(
        By.CSS_SELECTOR, 'td.datetime-cell[data-field="start_time"]'
    )
    start_time_cell.click()

    # Wait for modal
    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "editDatetimeModal").get_attribute("class")
    )

    # Calculate a date 2 months ago
    from datetime import datetime, timedelta

    two_months_ago = datetime.now() - timedelta(days=60)
    datetime_str = two_months_ago.strftime("%Y-%m-%dT%H:%M:%S")

    # Update the datetime input using JS (more robust)
    datetime_input = driver.find_element(By.ID, "datetimeInput")
    driver.execute_script(
        "arguments[0].value = arguments[1]; arguments[0].dispatchEvent(new Event('input', { bubbles: true }));",
        datetime_input,
        datetime_str,
    )

    # Submit the datetime change
    driver.find_element(By.ID, "datetimeModalSubmit").click()

    # Wait for modal to close
    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "editDatetimeModal").get_attribute("class")
    )
    time.sleep(1)

    # Test API with month date range
    from datetime import timezone

    now = datetime.now(timezone.utc)
    start_of_month = now.replace(day=1, hour=0, minute=0, second=0, microsecond=0)

    # Make API call with date range
    params = {
        "start": start_of_month.isoformat().replace("+00:00", "Z"),
        "end": now.isoformat().replace("+00:00", "Z"),
    }
    response = requests.get(f"{API_URL}/shifts/export", params=params)
    assert response.status_code == 200

    # Verify CSV contains ONLY the in-range shift
    csv_content = response.text
    lines = csv_content.strip().split("\n")

    # Should have header + 1 shift (only the in-range one)
    assert len(lines) == 2, f"Expected 2 lines (header + 1 shift), got {len(lines)}"

    # Verify the in-range shift is present
    assert "In-range shift" in csv_content

    # Verify the out-of-range shift is NOT present
    assert "Out-of-range shift" not in csv_content


def test_csv_export_custom_range(driver):
    """Test CSV export with custom date range - verify filtering excludes out-of-range shifts."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Create first shift (will be kept in range)
    odo_input = driver.find_element(By.ID, "startOdo")
    odo_input.send_keys("12000")
    start_btn = driver.find_element(By.ID, "startShiftBtn")
    start_btn.click()

    # Wait for active shift banner
    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    # Click End Shift button
    end_shift_btn = driver.find_element(By.ID, "endShiftBtn")
    end_shift_btn.click()

    # Wait for modal
    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    # Fill in end shift data
    driver.find_element(By.ID, "endOdo").send_keys("12100")
    driver.find_element(By.ID, "earnings").clear()
    driver.find_element(By.ID, "earnings").send_keys("75")
    driver.find_element(By.ID, "notes").send_keys("In 7-day range")

    # Submit
    driver.find_element(By.ID, "modalSubmit").click()

    # Wait for modal to close
    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )
    time.sleep(1)

    # Create second shift (will be edited to be outside 7-day range)
    odo_input = driver.find_element(By.ID, "startOdo")
    odo_input.send_keys("12100")
    start_btn = driver.find_element(By.ID, "startShiftBtn")
    start_btn.click()

    # Wait for active shift banner
    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    # Click End Shift button
    end_shift_btn = driver.find_element(By.ID, "endShiftBtn")
    end_shift_btn.click()

    # Wait for modal
    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    # Fill in end shift data
    driver.find_element(By.ID, "endOdo").send_keys("12200")
    driver.find_element(By.ID, "earnings").clear()
    driver.find_element(By.ID, "earnings").send_keys("90")
    driver.find_element(By.ID, "notes").send_keys("Outside 7-day range")

    # Submit
    driver.find_element(By.ID, "modalSubmit").click()

    # Wait for modal to close
    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )
    time.sleep(2)

    # Now edit the second shift's start time to be 30 days ago (outside 7-day range)
    # Find start time cell for the specific shift
    start_time_cell = driver.find_element(
        By.CSS_SELECTOR, 'td.datetime-cell[data-field="start_time"]'
    )
    start_time_cell.click()

    # Wait for modal
    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "editDatetimeModal").get_attribute("class")
    )

    # Calculate a date 30 days ago (outside the 7-day range we'll test)
    from datetime import datetime, timedelta

    thirty_days_ago = datetime.now() - timedelta(days=30)
    datetime_str = thirty_days_ago.strftime("%Y-%m-%dT%H:%M:%S")

    # Update the datetime input using JS (more robust)
    datetime_input = driver.find_element(By.ID, "datetimeInput")
    driver.execute_script(
        "arguments[0].value = arguments[1]; arguments[0].dispatchEvent(new Event('input', { bubbles: true }));",
        datetime_input,
        datetime_str,
    )

    # Submit the datetime change
    driver.find_element(By.ID, "datetimeModalSubmit").click()

    # Wait for modal to close
    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "editDatetimeModal").get_attribute("class")
    )
    time.sleep(1)

    # Test API with custom 7-day date range
    from datetime import timezone

    now = datetime.now(timezone.utc)
    start = (now - timedelta(days=7)).isoformat().replace("+00:00", "Z")
    end = now.isoformat().replace("+00:00", "Z")

    params = {"start": start, "end": end}
    response = requests.get(f"{API_URL}/shifts/export", params=params)
    assert response.status_code == 200

    # Verify CSV contains ONLY the in-range shift
    csv_content = response.text
    lines = csv_content.strip().split("\n")

    # Should have header + 1 shift (only the one in 7-day range)
    assert len(lines) == 2, f"Expected 2 lines (header + 1 shift), got {len(lines)}"

    # Verify the in-range shift is present
    assert "In 7-day range" in csv_content

    # Verify the out-of-range shift is NOT present
    assert "Outside 7-day range" not in csv_content


def test_csv_export_empty_database(driver):
    """Test CSV export with no shifts - should return header only."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Export with empty database
    response = requests.get(f"{API_URL}/shifts/export")
    assert response.status_code == 200

    csv_content = response.text
    lines = csv_content.strip().split("\n")

    # Should only have header
    assert len(lines) == 1
    assert "ID,Start Time,End Time" in lines[0]


def test_csv_export_invalid_date_range(driver):
    """Test CSV export with invalid date format - should return error."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Try to export with invalid date format
    params = {"start": "invalid-date", "end": "2025-12-31T23:59:59Z"}
    response = requests.get(f"{API_URL}/shifts/export", params=params)

    # Should return error (400 Bad Request)
    assert response.status_code == 400


# ============================================================================
# EDGE CASES AND VALIDATION
# ============================================================================


def test_empty_states(driver):
    """Test that empty states display correctly."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Check shifts empty state
    shifts_body = driver.find_element(By.ID, "shiftsBody")
    assert "No shifts" in shifts_body.text or "Loading" in shifts_body.text

    # Check maintenance empty state
    driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').click()

    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceView").is_displayed()
    )

    maintenance_body = driver.find_element(By.ID, "maintenanceBody")
    assert (
        "No maintenance" in maintenance_body.text or "Loading" in maintenance_body.text
    )


def test_search_no_results(driver):
    """Test that searching with no results shows appropriate message."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Create a shift
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("10100")
    driver.find_element(By.ID, "modalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    time.sleep(2)

    # Search for something that doesn't exist
    search_input = driver.find_element(By.ID, "searchInput")
    search_input.send_keys("nonexistent search term xyz")

    time.sleep(1)

    # Check for no results message
    table = driver.find_element(By.ID, "shiftsBody")
    assert "No shifts found" in table.text


def test_maintenance_validation_positive_interval(driver):
    """Test that maintenance interval must be positive."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Switch to maintenance
    driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').click()

    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceView").is_displayed()
    )

    # Try to create with zero or negative interval
    driver.find_element(By.ID, "addMaintenanceBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    driver.find_element(By.ID, "maintenanceName").send_keys("Invalid Item")
    driver.find_element(By.ID, "maintenanceMileageInterval").send_keys("0")
    driver.find_element(By.ID, "maintenanceModalSubmit").click()

    # Wait for error toast
    time.sleep(1)
    WebDriverWait(driver, 5).until(
        lambda d: "show" in d.find_element(By.ID, "toast").get_attribute("class")
    )


def test_notes_field_optional(driver):
    """Test that notes field is optional when ending shift."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Start and end shift without notes
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("10100")
    # Don't fill in notes
    driver.find_element(By.ID, "modalSubmit").click()

    # Should succeed
    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )


def test_enter_key_starts_shift(driver):
    driver.get(APP_URL)
    wait_for_page_load(driver)

    odo_input = driver.find_element(By.ID, "startOdo")
    odo_input.send_keys("10000")
    odo_input.send_keys(Keys.RETURN)

    # Wait for banner to become visible
    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    # Wait for text to update
    WebDriverWait(driver, 10).until(
        EC.text_to_be_present_in_element((By.ID, "activeShiftBanner"), "10000")
    )

    banner = driver.find_element(By.ID, "activeShiftBanner")
    assert "10000" in banner.text


# ============================================================================
# MULTIPLE SHIFTS TEST
# ============================================================================


def test_multiple_shifts_display(driver):
    """Test that multiple shifts display correctly in the table."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Create 3 shifts
    for i, odo in enumerate(["10000", "20000", "30000"]):
        driver.find_element(By.ID, "startOdo").send_keys(odo)
        driver.find_element(By.ID, "startShiftBtn").click()

        WebDriverWait(driver, 10).until(
            lambda d: "hidden"
            not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
        )

        driver.find_element(By.ID, "endShiftBtn").click()

        WebDriverWait(driver, 5).until(
            lambda d: "show"
            in d.find_element(By.ID, "endShiftModal").get_attribute("class")
        )

        driver.find_element(By.ID, "endOdo").send_keys(str(int(odo) + 100))
        driver.find_element(By.ID, "earnings").clear()
        driver.find_element(By.ID, "earnings").send_keys(str((i + 1) * 50))
        driver.find_element(By.ID, "modalSubmit").click()

        WebDriverWait(driver, 10).until(
            lambda d: "show"
            not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
        )

        time.sleep(2)

    # Check that all shifts appear in table
    table = driver.find_element(By.ID, "shiftsBody")
    rows = table.find_elements(By.TAG_NAME, "tr")

    # Should have 3 shifts (filter out empty state row if present)
    actual_rows = [r for r in rows if "empty-state" not in r.get_attribute("class")]
    assert len(actual_rows) >= 3


# ============================================================================
# RESPONSIVE/UI TESTS
# ============================================================================


def test_modal_close_button_works(driver):
    """Test that modal X button closes the modal."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Start shift
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    # Open modal
    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    # Click X button
    driver.find_element(By.ID, "modalClose").click()

    # Modal should close
    WebDriverWait(driver, 5).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )


def test_maintenance_modal_close(driver):
    """Test that maintenance modal closes properly."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Switch to maintenance
    driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').click()

    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceView").is_displayed()
    )

    # Open modal
    driver.find_element(By.ID, "addMaintenanceBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    # Close with X button
    driver.find_element(By.ID, "maintenanceModalClose").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        not in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )


# ============================================================================
# INTEGRATION TESTS
# ============================================================================


def test_complete_workflow_with_maintenance(driver):
    """Test complete workflow: create maintenance, complete shift, check required."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # 1. Create maintenance item
    driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').click()

    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceView").is_displayed()
    )

    driver.find_element(By.ID, "addMaintenanceBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    driver.find_element(By.ID, "maintenanceName").send_keys("5000mi Service")
    driver.find_element(By.ID, "maintenanceMileageInterval").send_keys("5000")
    driver.find_element(By.ID, "maintenanceLastService").clear()
    driver.find_element(By.ID, "maintenanceLastService").send_keys("0")
    driver.find_element(By.ID, "maintenanceModalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    time.sleep(2)

    # 2. Switch to shifts and create one that triggers maintenance
    driver.find_element(By.CSS_SELECTOR, '[data-view="shifts"]').click()

    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "shiftsView").is_displayed()
    )

    driver.find_element(By.ID, "startOdo").send_keys("5000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("5100")
    driver.find_element(By.ID, "modalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    time.sleep(3)

    # 3. Check that maintenance badge shows
    badge = driver.find_element(By.ID, "maintenanceBadge")
    assert "hidden" not in badge.get_attribute("class")

    # 4. Switch to maintenance and verify item is highlighted
    driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').click()

    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceView").is_displayed()
    )

    time.sleep(1)

    # Check for maintenance-required class
    WebDriverWait(driver, 10).until(
        lambda d: "maintenance-required"
        in d.find_element(By.CSS_SELECTOR, "#maintenanceBody tr").get_attribute("class")
    )


def test_remaining_mileage_updates_dynamically(driver):
    """Test that remaining mileage updates dynamically without page refresh."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # 1. Create maintenance item
    driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').click()
    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceView").is_displayed()
    )

    driver.find_element(By.ID, "addMaintenanceBtn").click()
    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    driver.find_element(By.ID, "maintenanceName").send_keys("Dynamic Test")
    driver.find_element(By.ID, "maintenanceMileageInterval").send_keys("5000")
    driver.find_element(By.ID, "maintenanceLastService").clear()
    driver.find_element(By.ID, "maintenanceLastService").send_keys("10000")
    driver.find_element(By.ID, "maintenanceModalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "maintenanceModal").get_attribute("class")
    )

    # Initial state: No shifts, so remaining should be 5000 (interval)
    time.sleep(1)
    row = driver.find_element(By.XPATH, "//tr[contains(., 'Dynamic Test')]")
    assert "5000" in row.text

    # 2. Create a shift
    driver.find_element(By.CSS_SELECTOR, '[data-view="shifts"]').click()
    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "shiftsView").is_displayed()
    )

    driver.find_element(By.ID, "startOdo").send_keys("12000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()
    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("13000")
    driver.find_element(By.ID, "modalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    # 3. Check maintenance view again - should be updated
    driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').click()
    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceView").is_displayed()
    )

    # Calculation: 5000 - (13000 - 10000) = 2000
    time.sleep(1)
    row = driver.find_element(By.XPATH, "//tr[contains(., 'Dynamic Test')]")
    assert "2000" in row.text

    # 4. Update the shift odometer
    driver.find_element(By.CSS_SELECTOR, '[data-view="shifts"]').click()
    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "shiftsView").is_displayed()
    )

    # Find the shift row and edit end odometer
    # The shift ends at 13000. We'll change it to 14000.
    end_odo_cell = driver.find_element(By.XPATH, "//td[contains(text(), '13000')]")
    end_odo_cell.click()

    end_odo_cell.send_keys(Keys.CONTROL + "a")
    end_odo_cell.send_keys(Keys.DELETE)
    end_odo_cell.send_keys("14000")
    end_odo_cell.send_keys(Keys.RETURN)

    time.sleep(1)  # Wait for update

    # 5. Check maintenance view again
    driver.find_element(By.CSS_SELECTOR, '[data-view="maintenance"]').click()
    WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "maintenanceView").is_displayed()
    )

    # New Calculation: 5000 - (14000 - 10000) = 1000
    time.sleep(1)
    row = driver.find_element(By.XPATH, "//tr[contains(., 'Dynamic Test')]")
    assert "1000" in row.text


def test_stats_calculation_accuracy(driver):
    """Test that stats calculations are accurate."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Create shift with known values
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    # Wait a bit to have measurable hours
    time.sleep(3)

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    # Earnings: 100, Tips: 20, Gas: 15 = Total: 105
    driver.find_element(By.ID, "endOdo").send_keys("10050")  # 50 miles
    driver.find_element(By.ID, "earnings").clear()
    driver.find_element(By.ID, "earnings").send_keys("100.00")
    driver.find_element(By.ID, "tips").clear()
    driver.find_element(By.ID, "tips").send_keys("20.00")
    driver.find_element(By.ID, "gasCost").clear()
    driver.find_element(By.ID, "gasCost").send_keys("15.00")
    driver.find_element(By.ID, "modalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    time.sleep(2)

    # Check stats
    total_earnings = driver.find_element(By.ID, "statTotalEarnings").text
    total_miles = driver.find_element(By.ID, "statTotalMiles").text

    assert "105" in total_earnings
    assert "50" in total_miles


# ============================================================================
# DATETIME EDITING TESTS
# ============================================================================


def test_edit_shift_start_time_happy_path(driver):
    """Test successfully editing a shift's start time."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Start and end a shift
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("10100")
    driver.find_element(By.ID, "earnings").clear()
    driver.find_element(By.ID, "earnings").send_keys("100")
    driver.find_element(By.ID, "tips").clear()
    driver.find_element(By.ID, "tips").send_keys("20")
    driver.find_element(By.ID, "gasCost").clear()
    driver.find_element(By.ID, "gasCost").send_keys("10")
    driver.find_element(By.ID, "modalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    time.sleep(2)  # Wait for table to update

    # Find and click the start_time cell
    start_time_cell = driver.find_element(
        By.CSS_SELECTOR, 'td.datetime-cell[data-field="start_time"]'
    )
    original_start_time = start_time_cell.text
    start_time_cell.click()

    # Wait for datetime modal to open
    datetime_modal = WebDriverWait(driver, 5).until(
        lambda d: d.find_element(By.ID, "editDatetimeModal")
    )
    assert "show" in datetime_modal.get_attribute("class")

    # Verify modal title
    modal_title = driver.find_element(By.ID, "datetimeModalTitle").text
    assert "Start Time" in modal_title

    # Get the current value and modify it (subtract 1 hour)
    datetime_input = driver.find_element(By.ID, "datetimeInput")
    current_value = datetime_input.get_attribute("value")

    # Parse and modify the datetime (subtract 1 hour)
    from datetime import datetime, timedelta

    dt = datetime.fromisoformat(current_value)
    new_dt = dt - timedelta(hours=1)
    new_value = new_dt.strftime("%Y-%m-%dT%H:%M:%S")

    # Use JavaScript to set the value to ensure proper format
    driver.execute_script(
        "arguments[0].value = arguments[1]; arguments[0].dispatchEvent(new Event('input', { bubbles: true }));",
        datetime_input,
        new_value,
    )

    # Submit the change
    driver.find_element(By.ID, "datetimeModalSubmit").click()

    # Wait for modal to close
    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "editDatetimeModal").get_attribute("class")
    )

    time.sleep(2)  # Wait for table to update

    # Verify the start time changed
    updated_start_time_cell = driver.find_element(
        By.CSS_SELECTOR, 'td.datetime-cell[data-field="start_time"]'
    )
    updated_start_time = updated_start_time_cell.text
    assert updated_start_time != original_start_time


def test_edit_shift_end_time_happy_path(driver):
    """Test successfully editing a shift's end time."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Start and end a shift
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("10100")
    driver.find_element(By.ID, "earnings").clear()
    driver.find_element(By.ID, "earnings").send_keys("80")
    driver.find_element(By.ID, "modalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    time.sleep(2)

    # Click the end_time cell
    end_time_cell = driver.find_element(
        By.CSS_SELECTOR, 'td.datetime-cell[data-field="end_time"]'
    )
    original_end_time = end_time_cell.text
    end_time_cell.click()

    # Wait for modal
    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "editDatetimeModal").get_attribute("class")
    )

    # Verify modal title
    modal_title = driver.find_element(By.ID, "datetimeModalTitle").text
    assert "End Time" in modal_title

    # Modify end time (add 2 hours)
    datetime_input = driver.find_element(By.ID, "datetimeInput")
    current_value = datetime_input.get_attribute("value")

    from datetime import datetime, timedelta

    dt = datetime.fromisoformat(current_value)
    new_dt = dt + timedelta(hours=2)
    new_value = new_dt.strftime("%Y-%m-%dT%H:%M:%S")

    # Use JavaScript to set the value
    driver.execute_script(
        "arguments[0].value = arguments[1]; arguments[0].dispatchEvent(new Event('input', { bubbles: true }));",
        datetime_input,
        new_value,
    )
    driver.find_element(By.ID, "datetimeModalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "editDatetimeModal").get_attribute("class")
    )

    time.sleep(2)

    # Verify end time changed
    updated_end_time_cell = driver.find_element(
        By.CSS_SELECTOR, 'td.datetime-cell[data-field="end_time"]'
    )
    updated_end_time = updated_end_time_cell.text
    assert updated_end_time != original_end_time


def test_edit_both_times_and_verify_recalculation(driver):
    """Test editing both start and end times and verify hours_worked recalculates."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Create a shift
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("10100")
    driver.find_element(By.ID, "earnings").clear()
    driver.find_element(By.ID, "earnings").send_keys("120")
    driver.find_element(By.ID, "tips").clear()
    driver.find_element(By.ID, "tips").send_keys("30")
    driver.find_element(By.ID, "gasCost").clear()
    driver.find_element(By.ID, "gasCost").send_keys("10")
    driver.find_element(By.ID, "modalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    time.sleep(2)

    # Get original hours worked
    table = driver.find_element(By.ID, "shiftsBody")
    original_table_text = table.text

    # Edit start time (1 hour earlier)
    start_time_cell = driver.find_element(
        By.CSS_SELECTOR, 'td.datetime-cell[data-field="start_time"]'
    )
    start_time_cell.click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "editDatetimeModal").get_attribute("class")
    )

    datetime_input = driver.find_element(By.ID, "datetimeInput")
    current_value = datetime_input.get_attribute("value")

    from datetime import datetime, timedelta

    dt = datetime.fromisoformat(current_value)
    new_dt = dt - timedelta(hours=1)
    new_value = new_dt.strftime("%Y-%m-%dT%H:%M:%S")

    # Use JavaScript to set the value
    driver.execute_script(
        "arguments[0].value = arguments[1]; arguments[0].dispatchEvent(new Event('input', { bubbles: true }));",
        datetime_input,
        new_value,
    )
    driver.find_element(By.ID, "datetimeModalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "editDatetimeModal").get_attribute("class")
    )

    time.sleep(2)

    # Verify table updated (hours worked should have changed)
    table = driver.find_element(By.ID, "shiftsBody")
    updated_table_text = table.text
    # The hours worked and hourly pay should be different now
    assert updated_table_text != original_table_text


def test_edit_end_time_before_start_time_validation(driver):
    """Test that setting end time before start time shows validation error."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Create a shift
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("10100")
    driver.find_element(By.ID, "modalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    time.sleep(2)

    # Click end_time cell
    end_time_cell = driver.find_element(
        By.CSS_SELECTOR, 'td.datetime-cell[data-field="end_time"]'
    )
    end_time_cell.click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "editDatetimeModal").get_attribute("class")
    )

    # Try to set end time to 2 hours BEFORE current time (should fail validation)
    datetime_input = driver.find_element(By.ID, "datetimeInput")
    current_value = datetime_input.get_attribute("value")

    from datetime import datetime, timedelta

    dt = datetime.fromisoformat(current_value)
    # Set to way before start time
    new_dt = dt - timedelta(hours=5)
    new_value = new_dt.strftime("%Y-%m-%dT%H:%M:%S")

    # Use JavaScript to set the value
    driver.execute_script(
        "arguments[0].value = arguments[1]; arguments[0].dispatchEvent(new Event('input', { bubbles: true }));",
        datetime_input,
        new_value,
    )
    driver.find_element(By.ID, "datetimeModalSubmit").click()

    # Wait for error toast
    toast = WebDriverWait(driver, 5).until(lambda d: d.find_element(By.ID, "toast"))

    # Wait for toast to populate
    max_retries = 10
    for i in range(max_retries):
        if toast.text and (
            "after" in toast.text.lower() or "before" in toast.text.lower()
        ):
            break
        time.sleep(0.2)
        toast = driver.find_element(By.ID, "toast")

    assert toast.text, "End time must be after start time"
    assert "after" in toast.text.lower() or "before" in toast.text.lower()


def test_edit_start_time_after_end_time_validation(driver):
    """Test that setting start time after end time shows validation error."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Create a shift
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("10100")
    driver.find_element(By.ID, "modalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    time.sleep(2)

    # Click start_time cell
    start_time_cell = driver.find_element(
        By.CSS_SELECTOR, 'td.datetime-cell[data-field="start_time"]'
    )
    start_time_cell.click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "editDatetimeModal").get_attribute("class")
    )

    # Try to set start time to way in the future (after end time)
    datetime_input = driver.find_element(By.ID, "datetimeInput")
    current_value = datetime_input.get_attribute("value")

    from datetime import datetime, timedelta

    dt = datetime.fromisoformat(current_value)
    new_dt = dt + timedelta(hours=10)  # Way after end time
    new_value = new_dt.strftime("%Y-%m-%dT%H:%M:%S")

    # Use JavaScript to set the value
    driver.execute_script(
        "arguments[0].value = arguments[1]; arguments[0].dispatchEvent(new Event('input', { bubbles: true }));",
        datetime_input,
        new_value,
    )
    driver.find_element(By.ID, "datetimeModalSubmit").click()

    # Wait for error toast
    toast = WebDriverWait(driver, 5).until(lambda d: d.find_element(By.ID, "toast"))

    max_retries = 10
    for i in range(max_retries):
        if toast.text and (
            "before" in toast.text.lower() or "after" in toast.text.lower()
        ):
            break
        time.sleep(0.2)
        toast = driver.find_element(By.ID, "toast")

    assert toast.text, "Start time must be before end time"
    assert "before" in toast.text.lower() or "after" in toast.text.lower()


def test_datetime_modal_cancel_closes(driver):
    """Test that canceling the datetime edit modal closes it without changes."""
    driver.get(APP_URL)
    wait_for_page_load(driver)

    # Create a shift
    driver.find_element(By.ID, "startOdo").send_keys("10000")
    driver.find_element(By.ID, "startShiftBtn").click()

    WebDriverWait(driver, 10).until(
        lambda d: "hidden"
        not in d.find_element(By.ID, "activeShiftBanner").get_attribute("class")
    )

    driver.find_element(By.ID, "endShiftBtn").click()

    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    driver.find_element(By.ID, "endOdo").send_keys("10100")
    driver.find_element(By.ID, "modalSubmit").click()

    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "endShiftModal").get_attribute("class")
    )

    time.sleep(2)

    # Get original start time
    start_time_cell = driver.find_element(
        By.CSS_SELECTOR, 'td.datetime-cell[data-field="start_time"]'
    )
    original_start_time = start_time_cell.text
    start_time_cell.click()

    # Wait for modal
    WebDriverWait(driver, 5).until(
        lambda d: "show"
        in d.find_element(By.ID, "editDatetimeModal").get_attribute("class")
    )

    # Modify the value but cancel
    datetime_input = driver.find_element(By.ID, "datetimeInput")
    current_value = datetime_input.get_attribute("value")

    from datetime import datetime, timedelta

    dt = datetime.fromisoformat(current_value)
    new_dt = dt - timedelta(hours=3)
    new_value = new_dt.strftime("%Y-%m-%dT%H:%M:%S")

    # Use JavaScript to set the value
    driver.execute_script(
        "arguments[0].value = arguments[1]; arguments[0].dispatchEvent(new Event('input', { bubbles: true }));",
        datetime_input,
        new_value,
    )

    # Click cancel instead of submit
    driver.find_element(By.ID, "datetimeModalCancel").click()

    # Wait for modal to close
    WebDriverWait(driver, 10).until(
        lambda d: "show"
        not in d.find_element(By.ID, "editDatetimeModal").get_attribute("class")
    )

    time.sleep(1)

    # Verify start time didn't change
    start_time_cell = driver.find_element(
        By.CSS_SELECTOR, 'td.datetime-cell[data-field="start_time"]'
    )
    current_start_time = start_time_cell.text
    assert current_start_time == original_start_time


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
