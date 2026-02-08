#!/usr/bin/env python3
"""Take real screenshots of ParkHub using Selenium + Firefox."""
import time
import os
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.firefox.options import Options
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

os.makedirs("docs/screenshots", exist_ok=True)

opts = Options()
opts.add_argument("--headless")
opts.add_argument("--width=1280")
opts.add_argument("--height=800")
opts.set_preference("ui.systemUsesDarkTheme", 1)

driver = webdriver.Firefox(options=opts)
driver.set_window_size(1280, 800)
BASE = "http://localhost:7878"

def shot(name, delay=2):
    time.sleep(delay)
    path = f"docs/screenshots/{name}.png"
    driver.save_screenshot(path)
    print(f"✓ {path}")

# Welcome (unauthenticated)
driver.get(f"{BASE}/welcome")
shot("welcome")

# Login page
driver.get(f"{BASE}/login")
shot("login")

# Perform login
try:
    inputs = driver.find_elements(By.CSS_SELECTOR, "input")
    if len(inputs) >= 2:
        inputs[0].clear()
        inputs[0].send_keys("admin")
        inputs[1].clear()
        inputs[1].send_keys("Test1234!")
    submit = driver.find_element(By.CSS_SELECTOR, "button[type='submit']")
    submit.click()
    time.sleep(3)
except Exception as e:
    print(f"Login attempt: {e}")

# Dashboard
shot("dashboard")

# Booking
driver.get(f"{BASE}/book")
shot("booking")

# Admin
driver.get(f"{BASE}/admin")
shot("admin")

# Settings/themes
driver.get(f"{BASE}/settings")
shot("themes")

# Mobile view
driver.set_window_size(375, 812)
driver.get(f"{BASE}/")
shot("mobile")

# Dark mode booking mobile
driver.get(f"{BASE}/book")
shot("dark-mode")

driver.quit()
print("\nAll screenshots captured!")
