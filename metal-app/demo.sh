#!/bin/bash

# Demo script — simulates stream-startup-core.sh output with realistic timing.
# Use this to test the Jarvis boot-up visualization without actually starting services.

BASE="/Users/dylan/Desktop/projects/music-player"

echo "Validating YouTube token..."
sleep 0.8
echo "Token validated                              OK"
echo ""

echo "Cleaning up existing processes..."
sleep 0.5
echo "Cleanup complete                             OK"
echo ""
echo "Starting stream services..."

sleep 0.3
echo "[1/6] OBS Audio Bridge..."
sleep 1.0
echo "       OBS Audio Bridge                      ONLINE"

echo "[2/6] Great Firewall..."
sleep 1.5
echo "       Great Firewall                        ONLINE"

echo "[3/6] Chat Monitor..."
sleep 0.8
echo "       Chat Monitor                          ONLINE"

echo "[4/6] Firewall Monitor..."
sleep 0.8
echo "       Firewall Monitor                      ONLINE"

echo "[5/6] Electron Music Player..."
sleep 0.8
echo "       Electron Music Player                 ONLINE"

echo "[6/6] VibeToText..."
sleep 0.8
echo "       VibeToText                            ONLINE"

echo ""
echo "Waiting for services to initialize..."
sleep 1.5

echo "Refreshing OBS browser caches..."
sleep 0.5
echo "       OBS caches refreshed                  OK"

echo ""
echo "All systems operational."
