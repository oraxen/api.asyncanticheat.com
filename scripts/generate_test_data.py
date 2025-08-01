#!/usr/bin/env python3
"""
Generate labeled test data for NCP module check validation.

Usage:
    python scripts/generate_test_data.py

This creates NDJSON.gz files in ./test_data/ that can be sent to the module
to verify each check triggers correctly.
"""

import gzip
import json
import os
import uuid
from datetime import datetime

OUTPUT_DIR = "./test_data"

def ensure_dir():
    os.makedirs(OUTPUT_DIR, exist_ok=True)

def write_ndjson_gz(filename: str, meta: dict, events: list):
    """Write meta + events as NDJSON.gz"""
    filepath = os.path.join(OUTPUT_DIR, filename)
    with gzip.open(filepath, "wt", encoding="utf-8") as f:
        f.write(json.dumps(meta) + "\n")
        for ev in events:
            f.write(json.dumps(ev) + "\n")
    print(f"  Created: {filepath} ({len(events)} events)")

def generate_combat_wrongturn():
    """FIGHT_WRONGTURN: pitch > 90 degrees"""
    player_uuid = str(uuid.uuid4())
    events = []
    base_ts = 1700000000000
    
    # Normal attacks first
    for i in range(5):
        events.append({
            "ts": base_ts + i * 100,
            "uuid": player_uuid,
            "entity_id": 12345,
            "player_pitch": 45.0,  # Normal
            "player_yaw": 90.0,
        })
    
    # Invalid pitch attacks (should trigger)
    for i in range(5):
        events.append({
            "ts": base_ts + 500 + i * 100,
            "uuid": player_uuid,
            "entity_id": 12345,
            "player_pitch": 95.0 + i * 5,  # Invalid: > 90
            "player_yaw": 90.0,
        })
    
    write_ndjson_gz("combat_wrongturn.ndjson.gz", 
                    {"transform": "combat_events_v1_ndjson_gz", "label": "wrongturn"},
                    events)
    return player_uuid

def generate_combat_speed():
    """FIGHT_SPEED: attacks per second > 8"""
    player_uuid = str(uuid.uuid4())
    events = []
    base_ts = 1700000000000
    
    # Very fast attacks: 20 attacks in 1 second = 20 APS (should trigger)
    for i in range(20):
        events.append({
            "ts": base_ts + i * 50,  # 50ms apart = 20 APS
            "uuid": player_uuid,
            "entity_id": 12345,
            "player_pitch": 0.0,
            "player_yaw": 90.0,
        })
    
    write_ndjson_gz("combat_speed.ndjson.gz",
                    {"transform": "combat_events_v1_ndjson_gz", "label": "speed"},
                    events)
    return player_uuid

def generate_combat_reach():
    """FIGHT_REACH: reach distance > 4.4 blocks"""
    player_uuid = str(uuid.uuid4())
    events = []
    base_ts = 1700000000000
    
    # Normal reach attacks
    for i in range(3):
        events.append({
            "ts": base_ts + i * 200,
            "uuid": player_uuid,
            "entity_id": 12345,
            "reach_distance": 3.5,  # Normal
        })
    
    # Long reach attacks (should trigger)
    for i in range(5):
        events.append({
            "ts": base_ts + 600 + i * 200,
            "uuid": player_uuid,
            "entity_id": 12345,
            "reach_distance": 5.0 + i * 0.5,  # > 4.4
        })
    
    write_ndjson_gz("combat_reach.ndjson.gz",
                    {"transform": "combat_events_v1_ndjson_gz", "label": "reach"},
                    events)
    return player_uuid

def generate_combat_direction():
    """FIGHT_DIRECTION: aim_off > 0.1"""
    player_uuid = str(uuid.uuid4())
    events = []
    base_ts = 1700000000000
    
    # Normal aim
    for i in range(3):
        events.append({
            "ts": base_ts + i * 200,
            "uuid": player_uuid,
            "entity_id": 12345,
            "aim_off": 0.05,  # Normal
        })
    
    # Bad aim (should trigger)
    for i in range(5):
        events.append({
            "ts": base_ts + 600 + i * 200,
            "uuid": player_uuid,
            "entity_id": 12345,
            "aim_off": 0.5 + i * 0.1,  # > 0.1
        })
    
    write_ndjson_gz("combat_direction.ndjson.gz",
                    {"transform": "combat_events_v1_ndjson_gz", "label": "direction"},
                    events)
    return player_uuid

def generate_combat_noswing():
    """FIGHT_NOSWING: attacks without arm swing"""
    player_uuid = str(uuid.uuid4())
    events = []
    base_ts = 1700000000000
    
    # Attacks with swing (had_swing=true) - normal
    for i in range(3):
        events.append({
            "ts": base_ts + i * 200,
            "uuid": player_uuid,
            "entity_id": 12345,
            "had_swing": True,
        })
    
    # Attacks without swing (should trigger after threshold)
    for i in range(10):
        events.append({
            "ts": base_ts + 600 + i * 100,
            "uuid": player_uuid,
            "entity_id": 12345,
            "had_swing": False,
        })
    
    write_ndjson_gz("combat_noswing.ndjson.gz",
                    {"transform": "combat_events_v1_ndjson_gz", "label": "noswing"},
                    events)
    return player_uuid

def generate_combat_angle():
    """FIGHT_ANGLE: rapid target switching with large yaw changes"""
    player_uuid = str(uuid.uuid4())
    events = []
    base_ts = 1700000000000
    
    # Rapid target switching with large yaw changes (aimbot pattern)
    targets = [1001, 1002, 1003, 1004, 1005]
    yaws = [0.0, 90.0, 180.0, 270.0, 45.0, 135.0, 225.0, 315.0]
    
    for i in range(15):
        events.append({
            "ts": base_ts + i * 50,  # Very fast: 50ms apart
            "uuid": player_uuid,
            "entity_id": targets[i % len(targets)],  # Rotating targets
            "player_x": 100.0,
            "player_y": 64.0,
            "player_z": 100.0,
            "player_yaw": yaws[i % len(yaws)],  # Large yaw changes
            "player_pitch": 0.0,
        })
    
    write_ndjson_gz("combat_angle.ndjson.gz",
                    {"transform": "combat_events_v1_ndjson_gz", "label": "angle"},
                    events)
    return player_uuid

def generate_movement_speed():
    """MOVING_SPEED: speed_bps > 15"""
    player_uuid = str(uuid.uuid4())
    events = []
    base_ts = 1700000000000
    
    # Normal speed
    for i in range(5):
        events.append({
            "ts": base_ts + i * 50,
            "uuid": player_uuid,
            "x": 100.0 + i * 0.5,
            "y": 64.0,
            "z": 100.0,
            "speed_bps": 8.0,  # Normal walking/sprinting
            "dt_ms": 50.0,
        })
    
    # High speed (should trigger)
    for i in range(10):
        events.append({
            "ts": base_ts + 250 + i * 50,
            "uuid": player_uuid,
            "x": 102.5 + i * 2.0,  # Moving fast
            "y": 64.0,
            "z": 100.0,
            "speed_bps": 25.0 + i * 2.0,  # > 15 bps
            "dt_ms": 50.0,
        })
    
    write_ndjson_gz("movement_speed.ndjson.gz",
                    {"transform": "movement_events_v1_ndjson_gz", "label": "speed"},
                    events)
    return player_uuid

def generate_movement_morepackets():
    """MOVING_MOREPACKETS: dt_ms < 5"""
    player_uuid = str(uuid.uuid4())
    events = []
    base_ts = 1700000000000
    
    # Normal packet rate
    for i in range(5):
        events.append({
            "ts": base_ts + i * 50,
            "uuid": player_uuid,
            "x": 100.0 + i * 0.1,
            "y": 64.0,
            "z": 100.0,
            "dt_ms": 50.0,  # Normal
        })
    
    # Too frequent packets (should trigger)
    for i in range(20):
        events.append({
            "ts": base_ts + 250 + i * 2,  # 2ms apart
            "uuid": player_uuid,
            "x": 100.5 + i * 0.01,
            "y": 64.0,
            "z": 100.0,
            "dt_ms": 2.0,  # < 5ms threshold
        })
    
    write_ndjson_gz("movement_morepackets.ndjson.gz",
                    {"transform": "movement_events_v1_ndjson_gz", "label": "morepackets"},
                    events)
    return player_uuid

def generate_movement_nofall():
    """MOVING_NOFALL: dy < -3 with on_ground=true"""
    player_uuid = str(uuid.uuid4())
    events = []
    base_ts = 1700000000000
    
    # Normal falling (on_ground=false)
    for i in range(5):
        events.append({
            "ts": base_ts + i * 50,
            "uuid": player_uuid,
            "x": 100.0,
            "y": 70.0 - i * 0.8,
            "z": 100.0,
            "dy": -0.8,
            "on_ground": False,  # Correct
        })
    
    # NoFall cheat: large fall with on_ground=true (should trigger)
    for i in range(5):
        events.append({
            "ts": base_ts + 250 + i * 50,
            "uuid": player_uuid,
            "x": 100.0,
            "y": 66.0 - i * 4.0,  # Falling fast
            "z": 100.0,
            "dy": -4.0 - i,  # Large negative dy
            "on_ground": True,  # Spoofed!
        })
    
    write_ndjson_gz("movement_nofall.ndjson.gz",
                    {"transform": "movement_events_v1_ndjson_gz", "label": "nofall"},
                    events)
    return player_uuid

def generate_movement_timer():
    """MOVING_TIMER: packet rate > expected * tolerance"""
    player_uuid = str(uuid.uuid4())
    events = []
    base_ts = 1700000000000
    
    # Timer hack: 30 packets in 1 second (should be ~20)
    for i in range(30):
        events.append({
            "ts": base_ts + i * 33,  # ~30 packets/sec
            "uuid": player_uuid,
            "x": 100.0 + i * 0.1,
            "y": 64.0,
            "z": 100.0,
            "dt_ms": 33.0,
        })
    
    write_ndjson_gz("movement_timer.ndjson.gz",
                    {"transform": "movement_events_v1_ndjson_gz", "label": "timer"},
                    events)
    return player_uuid

def main():
    print("Generating labeled test data for NCP module checks...\n")
    ensure_dir()
    
    print("Combat checks:")
    generate_combat_wrongturn()
    generate_combat_speed()
    generate_combat_reach()
    generate_combat_direction()
    generate_combat_noswing()
    generate_combat_angle()
    
    print("\nMovement checks:")
    generate_movement_speed()
    generate_movement_morepackets()
    generate_movement_nofall()
    generate_movement_timer()
    
    print(f"\n✓ Test data generated in {OUTPUT_DIR}/")
    print("\nTo test locally, run the NCP module and send data:")
    print("""
# Start the module locally:
cd modules/ncp_module && cargo run

# In another terminal, send test data:
for f in test_data/*.ndjson.gz; do
    label=$(basename "$f" .ndjson.gz)
    echo "Testing: $label"
    curl -X POST http://localhost:4020/ingest \\
        -H "x-server-id: test-server" \\
        -H "x-transform: $(head -1 <(zcat "$f") | jq -r .transform)" \\
        -H "Content-Type: application/octet-stream" \\
        --data-binary @"$f"
    echo
done
""")

if __name__ == "__main__":
    main()

