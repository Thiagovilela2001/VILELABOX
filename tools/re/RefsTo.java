// Lista quem referencia cada endereco (string, constante, funcao): uma linha por referencia.
// Uso: -postScript RefsTo.java <saida.tsv> <lista-de-enderecos>
// @category VILELABOX

import java.io.BufferedWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.List;

import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;
import ghidra.program.model.symbol.Reference;

public class RefsTo extends GhidraScript {
    @Override
    protected void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length != 2) {
            throw new IllegalArgumentException("uso: RefsTo.java <saida.tsv> <lista-de-enderecos>");
        }
        List<String> lines = Files.readAllLines(Paths.get(args[1]), StandardCharsets.UTF_8);
        try (BufferedWriter out = Files.newBufferedWriter(Paths.get(args[0]), StandardCharsets.UTF_8)) {
            out.write("target\tfrom\tfunction\n");
            for (String line : lines) {
                String text = line.trim();
                if (text.isEmpty()) {
                    continue;
                }
                Address target = toAddr(text);
                if (target == null) {
                    out.write(text + "\tinvalid\t\n");
                    continue;
                }
                for (Reference ref : getReferencesTo(target)) {
                    Function f = getFunctionContaining(ref.getFromAddress());
                    out.write(target + "\t" + ref.getFromAddress() + "\t"
                        + (f == null ? "" : f.getEntryPoint().toString()) + "\n");
                }
            }
        }
    }
}
