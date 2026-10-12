#!/usr/bin/env bash
# Verificação dos módulos próprios do VILELABOX.
#
#   tools/verify-mod.sh [--no-build] [--full]
#
#   --no-build  usa o build/ existente (padrão: compila antes)
#   --full      roda também a suíte inteira do Mixxx; as falhas conhecidas da
#               base (tools/verify/known-failures.txt) rodam à parte e não
#               reprovam, mas qualquer outra falha reprova
#
# Variáveis: BUILD_DIR (padrão build), JOBS, VILELABOX_VOCAL_PYTHON,
# VILELABOX_RB_REFERENCE (instalador extraído para a checagem de arquivos).
# Retorna 0 só se todas as verificações obrigatórias passarem.
set -u
ROOT=$(cd "$(dirname "$0")/.." && pwd)
BUILD_DIR=${BUILD_DIR:-$ROOT/build}
JOBS=${JOBS:-$(nproc 2>/dev/null || echo 4)}
BUILD=1; FULL=0
for arg in "$@"; do
    case $arg in
        --no-build) BUILD=0 ;;
        --full) FULL=1 ;;
        *) echo "opção desconhecida: $arg" >&2; exit 2 ;;
    esac
done

RESULTS=(); FAILED=0
record() { RESULTS+=("$(printf '%-6s %s' "$1" "$2")"); [[ $1 == FALHA ]] && FAILED=1; return 0; }
step() { echo; echo "=== $1"; }

MOD_TESTS='BeatsTest.*:BeatGridTest.*:BeatMapTest.*:BpmControlTest.*:BeatsTranslateTest.*:BarSyncTest.*:BarJumpLoopTest.*:*GridPanelTest*:AutoDownbeatTest.*:AnalyzerGridTest.*:PhrasesTest.*:AnalyzerPhrasesTest.*:RelatedTracksTest.*:VocalAnalysisTest.*:AnalyzerVocalsTest.*:KeyUtilsTest.*:EngineBufferTest.*'
TEST_ENV=(env QT_QPA_PLATFORM=offscreen)

if (( BUILD )); then
    step "Compilação ($BUILD_DIR, $JOBS jobs)"
    if cmake --build "$BUILD_DIR" --parallel "$JOBS"; then record OK "compilação"; else record FALHA "compilação"; fi
fi

step "Testes C++ dos módulos"
XML=$BUILD_DIR/vilelabox-tests.xml
rm -f "$XML"
if [[ ! -x $BUILD_DIR/mixxx-test ]]; then
    record FALHA "testes C++: $BUILD_DIR/mixxx-test não existe"
else
    "${TEST_ENV[@]}" "$BUILD_DIR/mixxx-test" --resource-path "$ROOT/res" \
        --gtest_filter="$MOD_TESTS" --gtest_output="xml:$XML" > "$BUILD_DIR/vilelabox-tests.log" 2>&1
    status=$?
    if [[ -s $XML ]] && python3 "$ROOT/tools/verify/report.py" "$XML" && (( status == 0 )); then
        record OK "testes C++ dos módulos (relatório: $XML)"
    else
        [[ -s $XML ]] || tail -20 "$BUILD_DIR/vilelabox-tests.log"
        record FALHA "testes C++ dos módulos (log: $BUILD_DIR/vilelabox-tests.log)"
    fi
fi

step "Ponte XML do rekordbox (Python)"
if python3 "$ROOT/tools/rekordbox/test_sync.py"; then record OK "ponte XML"; else record FALHA "ponte XML"; fi

step "Detector de vocal (Python)"
VPY=${VILELABOX_VOCAL_PYTHON:-$HOME/.venv/bin/python}
if [[ -x $VPY ]] && "$VPY" -c 'import numpy, scipy' 2>/dev/null; then
    if "$VPY" "$ROOT/tools/vocals/test_detector.py"; then record OK "detector de vocal"; else record FALHA "detector de vocal"; fi
else
    record PULADO "detector de vocal: Python com NumPy e SciPy não encontrado ($VPY)"
fi

step "Skins"
for checker in "$ROOT"/res/skins/*/tools/check_refs.py "$ROOT"/res/skins/*/tools/check_sizes.py; do
    [[ -f $checker ]] || continue
    name=${checker#$ROOT/res/skins/}
    if python3 "$checker" > /tmp/vilelabox-skin-check.$$ 2>&1; then
        record OK "skin: $name"
    else
        cat /tmp/vilelabox-skin-check.$$
        record FALHA "skin: $name"
    fi
done
rm -f /tmp/vilelabox-skin-check.$$

step "Arquivos de terceiros"
REF=${VILELABOX_RB_REFERENCE:-$HOME/Downloads/rekordbox_extracted}
if [[ -d $REF ]]; then
    if (cd "$ROOT" && tools/check-third-party-assets.sh "$REF" 2.5.6..HEAD); then
        record OK "nenhum arquivo do instalador no histórico do mod"
    else
        record FALHA "arquivos do instalador encontrados"
    fi
else
    record PULADO "arquivos de terceiros: referência $REF não encontrada"
fi

if (( FULL )); then
    step "Suíte completa do Mixxx"
    KNOWN=$(grep -v '^#' "$ROOT/tools/verify/known-failures.txt" | sed 's/  #.*//' | grep -v '^\s*$' | paste -sd: -)
    if "${TEST_ENV[@]}" timeout 3000 "$BUILD_DIR/mixxx-test" --resource-path "$ROOT/res" \
            --gtest_filter="-$KNOWN" > "$BUILD_DIR/vilelabox-full.log" 2>&1; then
        record OK "suíte completa, sem as falhas conhecidas ($(grep -oE '[0-9]+ tests? from' "$BUILD_DIR/vilelabox-full.log" | tail -1 | cut -d' ' -f1) testes)"
    else
        grep -E '^\[  FAILED  \]' "$BUILD_DIR/vilelabox-full.log" | sort -u
        record FALHA "suíte completa (log: $BUILD_DIR/vilelabox-full.log)"
    fi
    "${TEST_ENV[@]}" timeout 60 "$BUILD_DIR/mixxx-test" --gtest_filter="$KNOWN" > "$BUILD_DIR/vilelabox-known.log" 2>&1
    record AVISO "falhas conhecidas da base rodadas à parte, não reprovam (log: $BUILD_DIR/vilelabox-known.log)"
fi

echo; echo "=== Resumo"
printf '%s\n' "${RESULTS[@]}"
if (( FAILED )); then echo "RESULTADO: FALHOU"; exit 1; fi
echo "RESULTADO: OK"
