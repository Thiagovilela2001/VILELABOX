#!/usr/bin/env python3
"""Summarises a gtest XML report by VILELABOX module. usage: report.py <xml>"""
import sys
import xml.etree.ElementTree as ET

MODULES = [
    ("Compassos e grid", ("BeatsTest", "BeatGridTest", "BeatMapTest", "BpmControlTest", "BeatsTranslateTest",
                          "BarJumpLoopTest", "AutoDownbeatTest", "AnalyzerGridTest", "GridPanelTest")),
    ("Sync, Mix Point e pads", ("BarSyncTest",)),
    ("Frases e cues", ("PhrasesTest", "AnalyzerPhrasesTest")),
    ("Relacionadas e energia", ("RelatedTracksTest",)),
    ("Vocal", ("VocalAnalysisTest", "AnalyzerVocalsTest")),
    ("Tonalidade e motor", ("KeyUtilsTest", "EngineBufferTest")),
]

def module_of(suite, name):
    suite = suite.split("/")[-1]  # parameterised suites: "Prefix/GridPanelTest"
    for module, suites in MODULES:
        if suite in suites or any(s in name for s in suites):
            return module
    return "Outros"

root = ET.parse(sys.argv[1]).getroot()
stats, failures = {}, []
for case in root.iter("testcase"):
    if case.get("status") == "notrun":
        continue
    module = module_of(case.get("classname", ""), case.get("name", ""))
    total, failed = stats.get(module, (0, 0))
    failed_now = case.find("failure") is not None
    stats[module] = (total + 1, failed + failed_now)
    if failed_now:
        message = case.find("failure").get("message", "").splitlines()[0][:160]
        failures.append(f"  [{module}] {case.get('classname')}.{case.get('name')}: {message}")
for module, (total, failed) in sorted(stats.items()):
    print(f"  {module:<26} {total - failed:>4}/{total:<4} {'OK' if not failed else f'{failed} FALHA(S)'}")
if failures:
    print("Falhas:")
    print("\n".join(failures))
sys.exit(1 if failures else 0)
