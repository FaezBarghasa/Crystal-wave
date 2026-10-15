"""
Crystal-Wave Universal Plugin SDK for Python.
Enables creating audio effects, synthesizers, and MIDI processors using ctypes.
Directly compatible with Linux shared objects (.so).
"""
import ctypes
from typing import Optional, List

CRYSTAL_PLUGIN_API_VERSION = 1

class CrystalPluginDescriptorC(ctypes.Structure):
    _fields_ = [
        ("api_version", ctypes.c_uint32),
        ("unique_id", ctypes.c_uint32),
        ("name", ctypes.c_char_p),
        ("vendor", ctypes.c_char_p),
        ("version", ctypes.c_char_p),
        ("category", ctypes.c_uint32),
        ("num_inputs", ctypes.c_uint32),
        ("num_outputs", ctypes.c_uint32),
        ("supports_sidechain", ctypes.c_bool),
        ("latency_samples", ctypes.c_uint32),
    ]

class CrystalAudioBufferC(ctypes.Structure):
    _fields_ = [
        ("left_channel", ctypes.POINTER(ctypes.c_float)),
        ("right_channel", ctypes.POINTER(ctypes.c_float)),
        ("sidechain_channel", ctypes.POINTER(ctypes.c_float)),
        ("num_samples", ctypes.c_uint32),
    ]

# Omid Packet (8 bytes)
class OmidPacket(ctypes.Structure):
    _fields_ = [("bytes", ctypes.c_uint8 * 8)]

class BaseCrystalPlugin:
    def __init__(self, sample_rate: float):
        self.sample_rate = sample_rate

    def process(self, left: List[float], right: List[float], num_samples: int):
        pass

    def on_omid_packet(self, packet_bytes: bytes):
        pass
