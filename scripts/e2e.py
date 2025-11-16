import os
import socket

import pytest
from selenium import webdriver
from selenium.common.exceptions import WebDriverException
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.support.ui import WebDriverWait

REMOTE_URL = "http://localhost:4444/wd/hub"
APP_HOST = (
    "host.docker.internal" if os.getenv("GITHUB_ACTIONS") == "true" else "127.0.0.1"
)


def port_open(host, port, timeout=1.0):
    """Check whether a TCP port is open."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.settimeout(timeout)
        return s.connect_ex((host, port)) == 0


def create_driver():
    options = Options()
    # add options if needed:
    # options.add_argument("--headless")

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
    WebDriverWait(driver, timeout).until(
        lambda d: d.execute_script("return document.readyState") == "complete"
    )


# --- Pytest fixtures ---
@pytest.fixture
def driver():
    drv = create_driver()
    yield drv
    drv.quit()


# --- Tests ---
def test_homepage_title(driver):
    driver.get(f"http://{APP_HOST}:3000")
    wait_for_page_load(driver)
    assert "LastMile" in driver.title
