@echo off
powershell -Command "Start-Process -Verb RunAs powershell -ArgumentList '-Command "cd C:\Users\eurydice\Desktop\hashnet_rust\hashnet_compute_shader; cargo.exe +nightly flamegraph --release ;pause"'"
