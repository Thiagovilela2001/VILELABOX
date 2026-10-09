#!/usr/bin/env bash
set -euo pipefail
rbx_source_dir="$(cd -- "$(dirname -- "$0")" && pwd)"
cmake -S "$rbx_source_dir" -B "$rbx_source_dir/build" -DQML=OFF -DENGINEPRIME=OFF -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTING=ON
cmake --build "$rbx_source_dir/build" --parallel 4
QT_QPA_PLATFORM=offscreen "$rbx_source_dir/build/mixxx-test" --gtest_filter='BeatsTest.*:BeatGridTest.*:BeatMapTest.*:BpmControlTest.*:BeatsTranslateTest.*:BarSyncTest.*:BarJumpLoopTest.*:*GridPanelTest*:AutoDownbeatTest.*'
