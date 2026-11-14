// Prescript do analyzeHeadless: desliga o analyzer "Create Address Tables".
// No rekordbox.exe ele estoura o heap (OutOfMemoryError em AddressTableAnalyzer).
// @category VILELABOX

import java.util.Map;

import ghidra.app.script.GhidraScript;

public class DisableAddressTables extends GhidraScript {
    private static final String ANALYZER = "Create Address Tables";

    @Override
    public void run() throws Exception {
        Map<String, String> options = getCurrentAnalysisOptionsAndValues(currentProgram);
        if (!options.containsKey(ANALYZER)) {
            throw new IllegalStateException("Analyzer nao encontrado: " + ANALYZER);
        }
        setAnalysisOption(currentProgram, ANALYZER, "false");
        println(ANALYZER + " desligado");
    }
}
