#!/usr/bin/env python3
"""Take comprehensive screenshots of ParkHub for documentation."""
import time, os
from selenium import webdriver
from selenium.webdriver.firefox.options import Options
from selenium.webdriver.firefox.service import Service
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

BASE = "http://localhost:7878"
OUT = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "docs", "screenshots")
os.makedirs(OUT, exist_ok=True)

def make_driver(width=1280, height=800):
    opts = Options()
    opts.add_argument('--headless')
    opts.add_argument(f'--width={width}')
    opts.add_argument(f'--height={height}')
    svc = Service(log_output=os.devnull)
    d = webdriver.Firefox(options=opts, service=svc)
    d.set_window_size(width, height)
    return d

def wait_load(d, sec=3):
    time.sleep(sec)

def login(d, user="admin", pw="Test1234!"):
    d.get(f"{BASE}/login")
    wait_load(d, 2)
    try:
        u = d.find_element(By.ID, "username")
        p = d.find_element(By.ID, "password")
        u.clear(); u.send_keys(user)
        p.clear(); p.send_keys(pw)
        d.find_element(By.CSS_SELECTOR, "button[type='submit']").click()
        wait_load(d, 3)
        print(f"  Logged in as {user}, URL: {d.current_url}")
    except Exception as e:
        print(f"  Login failed: {e}")

def shot(d, name):
    path = os.path.join(OUT, name)
    d.save_screenshot(path)
    print(f"  ✓ {name}")

def take_desktop():
    print("=== Desktop Screenshots (1280x800) ===")
    d = make_driver(1280, 800)
    try:
        # Login page
        d.get(f"{BASE}/login")
        wait_load(d, 2)
        shot(d, "login-desktop.png")

        # Login
        login(d)

        # Dashboard
        d.get(f"{BASE}/")
        wait_load(d, 2)
        shot(d, "dashboard-desktop.png")

        # Booking page
        d.get(f"{BASE}/book")
        wait_load(d, 2)
        shot(d, "booking-desktop.png")

        # Try to show license plate input
        try:
            inputs = d.find_elements(By.CSS_SELECTOR, "input[placeholder]")
            for inp in inputs:
                ph = inp.get_attribute("placeholder") or ""
                if "kennzeichen" in ph.lower() or "plate" in ph.lower() or "license" in ph.lower():
                    inp.click()
                    inp.send_keys("B-")
                    wait_load(d, 1)
                    shot(d, "booking-plate.png")
                    break
        except:
            pass

        # Bookings list
        d.get(f"{BASE}/bookings")
        wait_load(d, 2)
        shot(d, "bookings-list.png")

        # Admin overview
        d.get(f"{BASE}/admin")
        wait_load(d, 2)
        shot(d, "admin-overview.png")

        # Admin - try clicking user tab
        try:
            tabs = d.find_elements(By.CSS_SELECTOR, "button, a")
            for tab in tabs:
                txt = tab.text.lower()
                if "benutzer" in txt or "user" in txt:
                    tab.click()
                    wait_load(d, 1)
                    shot(d, "admin-users.png")
                    break
        except:
            pass

        # Admin - lots tab
        try:
            tabs = d.find_elements(By.CSS_SELECTOR, "button, a")
            for tab in tabs:
                txt = tab.text.lower()
                if "parkplä" in txt or "lots" in txt or "parking" in txt:
                    tab.click()
                    wait_load(d, 1)
                    shot(d, "admin-lots.png")
                    break
        except:
            pass

        # Admin branding
        d.get(f"{BASE}/admin/branding")
        wait_load(d, 2)
        shot(d, "admin-branding.png")

        # Admin - system tab
        try:
            d.get(f"{BASE}/admin")
            wait_load(d, 2)
            tabs = d.find_elements(By.CSS_SELECTOR, "button, a")
            for tab in tabs:
                txt = tab.text.lower()
                if "system" in txt:
                    tab.click()
                    wait_load(d, 1)
                    shot(d, "admin-system.png")
                    break
        except:
            pass

        # Privacy page
        d.get(f"{BASE}/privacy")
        wait_load(d, 2)
        shot(d, "privacy-page.png")

        # About page
        d.get(f"{BASE}/about")
        wait_load(d, 2)
        shot(d, "about-page.png")

        # Themes - light (default)
        d.get(f"{BASE}/")
        wait_load(d, 1)
        shot(d, "themes-light.png")

        # Dark mode - click theme toggle
        try:
            btns = d.find_elements(By.CSS_SELECTOR, "button[aria-label]")
            for btn in btns:
                al = btn.get_attribute("aria-label") or ""
                if "theme" in al.lower() or "dark" in al.lower() or "toggle" in al.lower():
                    btn.click()
                    wait_load(d, 1)
                    shot(d, "themes-dark.png")
                    break
        except:
            pass

        # Try palette switching via profile/settings page
        for palette_name, palette_file in [("solarized", "palette-solarized.png"), ("dracula", "palette-dracula.png"), ("nord", "palette-nord.png")]:
            try:
                d.execute_script(f"""
                    localStorage.setItem('parkhub-palette', '{palette_name}');
                    window.dispatchEvent(new Event('storage'));
                """)
                d.get(f"{BASE}/")
                wait_load(d, 2)
                shot(d, palette_file)
            except:
                pass

        # Reset to default
        d.execute_script("localStorage.removeItem('parkhub-palette')")

    finally:
        d.quit()

def take_mobile():
    print("=== Mobile Screenshots (375x812) ===")
    d = make_driver(375, 812)
    try:
        # Mobile login
        d.get(f"{BASE}/login")
        wait_load(d, 2)
        shot(d, "mobile-login.png")

        login(d)

        # Mobile dashboard
        d.get(f"{BASE}/")
        wait_load(d, 2)
        shot(d, "mobile-dashboard.png")

        # Mobile booking
        d.get(f"{BASE}/book")
        wait_load(d, 2)
        shot(d, "mobile-booking.png")

        # Mobile admin
        d.get(f"{BASE}/admin")
        wait_load(d, 2)
        shot(d, "mobile-admin.png")

        # Mobile dark mode
        try:
            btns = d.find_elements(By.CSS_SELECTOR, "button[aria-label]")
            for btn in btns:
                al = btn.get_attribute("aria-label") or ""
                if "theme" in al.lower() or "toggle" in al.lower():
                    btn.click()
                    wait_load(d, 1)
                    shot(d, "mobile-dark.png")
                    break
        except:
            pass

    finally:
        d.quit()

def take_tablet():
    print("=== Tablet Screenshots (768x1024) ===")
    d = make_driver(768, 1024)
    try:
        d.get(f"{BASE}/login")
        wait_load(d, 2)
        login(d)
        d.get(f"{BASE}/")
        wait_load(d, 2)
        shot(d, "tablet-dashboard.png")
    finally:
        d.quit()

if __name__ == "__main__":
    # First check if app is running
    import urllib.request
    try:
        urllib.request.urlopen(BASE, timeout=5)
        print(f"ParkHub is running at {BASE}")
    except Exception as e:
        print(f"ERROR: ParkHub not reachable at {BASE}: {e}")
        exit(1)

    take_desktop()
    take_mobile()
    take_tablet()
    print(f"\nDone! Screenshots saved to {OUT}")
    print(f"Total: {len(os.listdir(OUT))} files")
