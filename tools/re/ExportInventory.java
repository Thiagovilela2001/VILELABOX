// Exporta metadados estruturais do programa aberto pelo analyzeHeadless.
// @category VILELABOX

import java.io.BufferedWriter;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.Set;

import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.mem.MemoryAccessException;
import ghidra.program.model.symbol.Symbol;
import ghidra.program.model.listing.Data;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.FunctionIterator;
import ghidra.program.model.listing.GhidraClass;
import ghidra.program.model.symbol.ExternalLocation;
import ghidra.program.model.symbol.ExternalLocationIterator;
import ghidra.program.model.symbol.ExternalManager;
import ghidra.program.model.symbol.Namespace;

public class ExportInventory extends GhidraScript {
    @Override
    protected void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length != 1) {
            throw new IllegalArgumentException("uso: ExportInventory.java <diretorio-de-saida>");
        }

        Path out = Paths.get(args[0]);
        Files.createDirectories(out);
        exportFunctions(out.resolve("functions.tsv"), out.resolve("callgraph.tsv"));
        exportClasses(out.resolve("classes.tsv"));
        exportVtables(out.resolve("vtables.tsv"));
        exportStrings(out.resolve("strings.tsv"));
        exportImports(out.resolve("imports.tsv"));
    }

    private void exportFunctions(Path functionsPath, Path callgraphPath) throws Exception {
        try (BufferedWriter functions = writer(functionsPath);
                BufferedWriter callgraph = writer(callgraphPath)) {
            functions.write("address\tname\tnamespace\tsize\tcallers\tcallees\n");
            callgraph.write("caller\tcallee\n");

            FunctionIterator iterator = currentProgram.getFunctionManager().getFunctions(true);
            while (iterator.hasNext()) {
                monitor.checkCancelled();
                Function function = iterator.next();
                Set<Function> callers = function.getCallingFunctions(monitor);
                Set<Function> callees = function.getCalledFunctions(monitor);
                functions.write(tsv(
                    function.getEntryPoint().toString(),
                    function.getName(),
                    function.getParentNamespace().getName(true),
                    Long.toString(function.getBody().getNumAddresses()),
                    Integer.toString(callers.size()),
                    Integer.toString(callees.size())));

                List<Function> ordered = new ArrayList<>(callees);
                ordered.sort(Comparator.comparing(Function::getEntryPoint));
                for (Function callee : ordered) {
                    callgraph.write(tsv(
                        function.getEntryPoint().toString(),
                        callee.getEntryPoint().toString()));
                }
            }
        }
    }

    private void exportClasses(Path path) throws Exception {
        try (BufferedWriter output = writer(path)) {
            output.write("class\tparent\tfunctions\n");
            // Chave pelo ID: GhidraClass e o Namespace devolvido pela função
            // são objetos diferentes para o mesmo namespace.
            Map<Long, Integer> functionCounts = new HashMap<>();
            FunctionIterator functions = currentProgram.getFunctionManager().getFunctions(true);
            while (functions.hasNext()) {
                Namespace namespace = functions.next().getParentNamespace();
                functionCounts.merge(namespace.getID(), 1, Integer::sum);
            }

            List<Namespace> classes = new ArrayList<>();
            Iterator<GhidraClass> classIterator = currentProgram.getSymbolTable().getClassNamespaces();
            classIterator.forEachRemaining(classes::add);
            classes.sort(Comparator.comparing(namespace -> namespace.getName(true)));
            for (Namespace namespace : classes) {
                monitor.checkCancelled();
                Namespace parent = namespace.getParentNamespace();
                output.write(tsv(
                    namespace.getName(true),
                    parent == null ? "" : parent.getName(true),
                    Integer.toString(functionCounts.getOrDefault(namespace.getID(), 0))));
            }
        }
    }

    // Métodos virtuais por classe lidos das vftables já rotuladas pela análise
    // RTTI padrão. Substitui RecoverClassesFromRTTIScript, que estoura a memória
    // neste binário. ponytail: para no primeiro slot que não é função (ou num
    // novo rótulo); slots com thunks/funções não criadas ficam de fora.
    private void exportVtables(Path path) throws Exception {
        final int maxSlots = 1024;
        int pointerSize = currentProgram.getDefaultPointerSize();
        try (BufferedWriter output = writer(path)) {
            output.write("class\tvftable\tslot\tfunction\n");
            Iterator<Symbol> symbols = currentProgram.getSymbolTable().getSymbols("vftable");
            while (symbols.hasNext()) {
                monitor.checkCancelled();
                Symbol symbol = symbols.next();
                Address table = symbol.getAddress();
                String owner = symbol.getParentNamespace().getName(true);
                for (int slot = 0; slot < maxSlots; slot++) {
                    Address entry = table.add((long) slot * pointerSize);
                    if (slot > 0 && currentProgram.getSymbolTable().hasSymbol(entry)) {
                        break;
                    }
                    long raw;
                    try {
                        raw = pointerSize == 8
                            ? currentProgram.getMemory().getLong(entry)
                            : currentProgram.getMemory().getInt(entry) & 0xffffffffL;
                    } catch (MemoryAccessException exception) {
                        break;
                    }
                    Address target = table.getNewAddress(raw);
                    Function function = currentProgram.getFunctionManager().getFunctionAt(target);
                    if (function == null) {
                        break;
                    }
                    output.write(tsv(owner, table.toString(), Integer.toString(slot),
                        function.getEntryPoint().toString()));
                }
            }
        }
    }

    private void exportStrings(Path path) throws Exception {
        try (BufferedWriter output = writer(path)) {
            output.write("address\txrefs\tvalue\n");
            for (Data data : currentProgram.getListing().getDefinedData(true)) {
                monitor.checkCancelled();
                if (!data.hasStringValue()) {
                    continue;
                }
                Object value = data.getValue();
                output.write(tsv(
                    data.getAddress().toString(),
                    Integer.toString(currentProgram.getReferenceManager().getReferenceCountTo(data.getAddress())),
                    value == null ? "" : value.toString()));
            }
        }
    }

    private void exportImports(Path path) throws Exception {
        try (BufferedWriter output = writer(path)) {
            output.write("library\tsymbol\n");
            ExternalManager manager = currentProgram.getExternalManager();
            for (String library : manager.getExternalLibraryNames()) {
                monitor.checkCancelled();
                ExternalLocationIterator locations = manager.getExternalLocations(library);
                while (locations.hasNext()) {
                    ExternalLocation location = locations.next();
                    output.write(tsv(library, location.getLabel()));
                }
            }
        }
    }

    private BufferedWriter writer(Path path) throws IOException {
        return Files.newBufferedWriter(path, StandardCharsets.UTF_8);
    }

    private String tsv(String... fields) {
        StringBuilder line = new StringBuilder();
        for (int i = 0; i < fields.length; i++) {
            if (i > 0) {
                line.append('\t');
            }
            line.append(escape(fields[i]));
        }
        return line.append('\n').toString();
    }

    private String escape(String value) {
        return value.replace("\\", "\\\\")
            .replace("\t", "\\t")
            .replace("\r", "\\r")
            .replace("\n", "\\n");
    }
}
