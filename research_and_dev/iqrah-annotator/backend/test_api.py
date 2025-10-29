"""Test script for API endpoints."""

import json
import requests
from io import BytesIO

BASE_URL = "http://localhost:8000"


def test_health():
    """Test health endpoint."""
    print("\n=== Testing Health Endpoint ===")
    response = requests.get(f"{BASE_URL}/")
    print(f"Status: {response.status_code}")
    print(f"Response: {json.dumps(response.json(), indent=2)}")
    assert response.status_code == 200
    print("✅ Health check passed")


def test_create_recording():
    """Test creating a recording."""
    print("\n=== Testing Create Recording ===")
    data = {
        "rule": "ghunnah",
        "anti_pattern": "weak-ghunnah",
        "qpc_location": "89:27:3",
        "sample_rate": 16000,
        "duration_sec": 2.5
    }
    response = requests.post(f"{BASE_URL}/api/recordings", json=data)
    print(f"Status: {response.status_code}")
    print(f"Response: {json.dumps(response.json(), indent=2)}")
    assert response.status_code == 201
    recording = response.json()
    print(f"✅ Recording created with ID: {recording['id']}")
    return recording


def test_list_recordings():
    """Test listing recordings."""
    print("\n=== Testing List Recordings ===")
    response = requests.get(f"{BASE_URL}/api/recordings")
    print(f"Status: {response.status_code}")
    recordings = response.json()
    print(f"Found {len(recordings)} recordings")
    if recordings:
        print(f"First recording: {json.dumps(recordings[0], indent=2)}")
    assert response.status_code == 200
    print("✅ List recordings passed")
    return recordings


def test_get_recording(recording_id):
    """Test getting a specific recording."""
    print(f"\n=== Testing Get Recording {recording_id} ===")
    response = requests.get(f"{BASE_URL}/api/recordings/{recording_id}")
    print(f"Status: {response.status_code}")
    print(f"Response: {json.dumps(response.json(), indent=2)}")
    assert response.status_code == 200
    print("✅ Get recording passed")
    return response.json()


def test_upload_audio(recording_id):
    """Test uploading audio file."""
    print(f"\n=== Testing Upload Audio for Recording {recording_id} ===")

    # Create a dummy WAV file (44 bytes header + 1 second of silence at 16kHz)
    wav_data = bytearray([
        # RIFF header
        0x52, 0x49, 0x46, 0x46,  # "RIFF"
        0x24, 0x00, 0x01, 0x00,  # File size (36 + data size)
        0x57, 0x41, 0x56, 0x45,  # "WAVE"
        # fmt chunk
        0x66, 0x6D, 0x74, 0x20,  # "fmt "
        0x10, 0x00, 0x00, 0x00,  # Chunk size (16)
        0x01, 0x00,              # Audio format (1 = PCM)
        0x01, 0x00,              # Num channels (1 = mono)
        0x80, 0x3E, 0x00, 0x00,  # Sample rate (16000)
        0x00, 0x7D, 0x00, 0x00,  # Byte rate
        0x02, 0x00,              # Block align
        0x10, 0x00,              # Bits per sample (16)
        # data chunk
        0x64, 0x61, 0x74, 0x61,  # "data"
        0x00, 0x00, 0x01, 0x00,  # Data size
    ])

    # Add 1 second of silence (16000 samples * 2 bytes = 32000 bytes)
    wav_data.extend([0x00] * 32000)

    files = {"file": ("test_audio.wav", BytesIO(bytes(wav_data)), "audio/wav")}
    response = requests.post(f"{BASE_URL}/api/recordings/{recording_id}/upload", files=files)
    print(f"Status: {response.status_code}")
    print(f"Response: {json.dumps(response.json(), indent=2)}")
    assert response.status_code == 200
    print("✅ Audio upload passed")


def test_create_region(recording_id):
    """Test creating annotation regions."""
    print(f"\n=== Testing Create Region for Recording {recording_id} ===")
    data = {
        "recording_id": recording_id,
        "start_sec": 0.5,
        "end_sec": 1.2,
        "label": "weak-ghunnah-onset",
        "confidence": 0.9,
        "notes": "Clear weak nasal resonance"
    }
    response = requests.post(f"{BASE_URL}/api/regions", json=data)
    print(f"Status: {response.status_code}")
    print(f"Response: {json.dumps(response.json(), indent=2)}")
    assert response.status_code == 201
    region = response.json()
    print(f"✅ Region created with ID: {region['id']}")
    return region


def test_get_regions(recording_id):
    """Test getting regions for a recording."""
    print(f"\n=== Testing Get Regions for Recording {recording_id} ===")
    response = requests.get(f"{BASE_URL}/api/recordings/{recording_id}/regions")
    print(f"Status: {response.status_code}")
    regions = response.json()
    print(f"Found {len(regions)} regions")
    if regions:
        print(f"First region: {json.dumps(regions[0], indent=2)}")
    assert response.status_code == 200
    print("✅ Get regions passed")
    return regions


def test_update_region(region_id):
    """Test updating a region."""
    print(f"\n=== Testing Update Region {region_id} ===")
    data = {
        "confidence": 0.95,
        "notes": "Updated: Very clear weak nasal resonance"
    }
    response = requests.patch(f"{BASE_URL}/api/regions/{region_id}", json=data)
    print(f"Status: {response.status_code}")
    print(f"Response: {json.dumps(response.json(), indent=2)}")
    assert response.status_code == 200
    print("✅ Update region passed")


def test_export_json():
    """Test JSON export."""
    print("\n=== Testing JSON Export ===")
    response = requests.get(f"{BASE_URL}/api/export/json?rule=ghunnah")
    print(f"Status: {response.status_code}")
    export_data = response.json()
    print(f"Export version: {export_data['version']}")
    print(f"Recordings in export: {len(export_data['recordings'])}")
    if export_data['recordings']:
        rec = export_data['recordings'][0]
        print(f"First recording ID: {rec['id']}, Regions: {len(rec['regions'])}")
    assert response.status_code == 200
    print("✅ JSON export passed")
    return export_data


def test_delete_region(region_id):
    """Test deleting a region."""
    print(f"\n=== Testing Delete Region {region_id} ===")
    response = requests.delete(f"{BASE_URL}/api/regions/{region_id}")
    print(f"Status: {response.status_code}")
    print(f"Response: {json.dumps(response.json(), indent=2)}")
    assert response.status_code == 200
    print("✅ Delete region passed")


def test_delete_recording(recording_id):
    """Test deleting a recording."""
    print(f"\n=== Testing Delete Recording {recording_id} ===")
    response = requests.delete(f"{BASE_URL}/api/recordings/{recording_id}")
    print(f"Status: {response.status_code}")
    print(f"Response: {json.dumps(response.json(), indent=2)}")
    assert response.status_code == 200
    print("✅ Delete recording passed")


def run_all_tests():
    """Run all tests in sequence."""
    print("=" * 60)
    print("Starting API Tests")
    print("=" * 60)

    try:
        # Health check
        test_health()

        # Create a recording
        recording = test_create_recording()
        recording_id = recording['id']

        # Upload audio
        test_upload_audio(recording_id)

        # Create regions
        region1 = test_create_region(recording_id)
        region1_id = region1['id']

        # Create another region
        data2 = {
            "recording_id": recording_id,
            "start_sec": 1.5,
            "end_sec": 2.0,
            "label": "weak-ghunnah-sustain",
            "confidence": 0.85
        }
        response = requests.post(f"{BASE_URL}/api/regions", json=data2)
        region2 = response.json()

        # Get regions
        test_get_regions(recording_id)

        # Update region
        test_update_region(region1_id)

        # List recordings
        test_list_recordings()

        # Get specific recording
        test_get_recording(recording_id)

        # Export
        test_export_json()

        # Cleanup tests
        test_delete_region(region1_id)

        # Create one more recording to test filtering
        recording2_data = {
            "rule": "qalqalah",
            "anti_pattern": "weak-qalqalah",
            "sample_rate": 16000,
            "duration_sec": 1.8
        }
        response = requests.post(f"{BASE_URL}/api/recordings", json=recording2_data)
        recording2_id = response.json()['id']

        # Test filtered list
        print("\n=== Testing Filtered List (rule=qalqalah) ===")
        response = requests.get(f"{BASE_URL}/api/recordings?rule=qalqalah")
        filtered = response.json()
        print(f"Found {len(filtered)} recordings with rule=qalqalah")
        assert len(filtered) == 1
        print("✅ Filtered list passed")

        # Clean up
        test_delete_recording(recording_id)
        test_delete_recording(recording2_id)

        print("\n" + "=" * 60)
        print("✅ ALL TESTS PASSED!")
        print("=" * 60)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}")
        raise
    except Exception as e:
        print(f"\n❌ Error: {e}")
        raise


if __name__ == "__main__":
    run_all_tests()
