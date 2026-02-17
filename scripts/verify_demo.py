import requests

BASE_URL = "http://localhost:3000/api"


def test_demo_mode():
    print("🚀 Starting Demo Mode Verification")

    # 1. Start a new session (no cookie)
    print("\n1. Requesting shifts (New Session A)...")
    try:
        response_a = requests.get(f"{BASE_URL}/shifts")
    except requests.exceptions.ConnectionError:
        print("❌ Connection failed! Is the server running on port 3000?")
        return

    if response_a.status_code != 200:
        print(f"❌ Failed to get shifts: {response_a.status_code}")
        return

    cookie_a = response_a.cookies.get("lastmile_session")
    if not cookie_a:
        print("❌ No 'lastmile_session' cookie received!")
        return
    print(f"✅ Received Session A Cookie: {cookie_a}")

    shifts_a = response_a.json()
    print(f"ℹ️  Session A has {len(shifts_a)} shifts")
    if len(shifts_a) == 0:
        print("⚠️  Warning: No shifts found in new demo session. Is seeding working?")

    # 2. Create a specific shift in Session A
    print("\n2. Creating a new shift in Session A...")
    start_shift_payload = {"odometer_start": 10000}
    response_create = requests.post(
        f"{BASE_URL}/shifts/start",
        json=start_shift_payload,
        cookies={"lastmile_session": cookie_a},
    )
    if response_create.status_code != 200:
        print(
            f"❌ Failed to create shift: {response_create.status_code} {response_create.text}"
        )
        return

    shift_data = response_create.json()
    # Handle both string ID and Thing struct
    shift_id = shift_data["id"]
    if isinstance(shift_id, dict) and "id" in shift_id:
        shift_id = shift_id["id"]["String"]

    print(f"✅ Shift created in Session A with ID: {shift_id}")

    # 3. Verify Session A persistence
    print("\n3. Verifying Session A persistence...")
    response_a_2 = requests.get(
        f"{BASE_URL}/shifts", cookies={"lastmile_session": cookie_a}
    )
    shifts_a_2 = response_a_2.json()

    found = False
    for s in shifts_a_2:
        s_id = s["id"]
        if isinstance(s_id, dict) and "id" in s_id:
            s_id = s_id["id"]["String"]
        if str(s_id) == str(shift_id):
            found = True
            break

    if found:
        print("✅ Shift persisted in Session A")
    else:
        print("❌ Shift NOT found in Session A!")

    # 4. Start a SECOND session (Request without cookie)
    print("\n4. Requesting shifts (New Session B)...")
    response_b = requests.get(f"{BASE_URL}/shifts")  # No cookies

    cookie_b = response_b.cookies.get("lastmile_session")
    print(f"✅ Received Session B Cookie: {cookie_b}")

    if cookie_a == cookie_b:
        print("❌ Session IDs are identical! Isolation failed.")
        return
    else:
        print("✅ Session IDs represent different sessions")

    shifts_b = response_b.json()
    print(f"ℹ️  Session B has {len(shifts_b)} shifts")

    # Verify Session B does NOT see Session A's new shift
    found_in_b = False
    for s in shifts_b:
        s_id = s["id"]
        if isinstance(s_id, dict) and "id" in s_id:
            s_id = s_id["id"]["String"]
        if str(s_id) == str(shift_id):
            found_in_b = True
            break

    if found_in_b:
        print("❌ Session Isolation FAILED: Session B can see Session A's shift!")
    else:
        print("✅ Session Isolation VERIFIED: Session B cannot see Session A's shift")

    print("\n🎉 Demo Mode Verification Complete!")


if __name__ == "__main__":
    test_demo_mode()
