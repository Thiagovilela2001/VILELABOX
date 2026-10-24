// Descompila uma lista de enderecos sem interromper o lote em caso de falha.
// @category VILELABOX

import java.io.BufferedWriter;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.StandardOpenOption;
import java.util.List;

import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;

public class DecompileList extends GhidraScript {
    @Override
    protected void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length != 2) {
            throw new IllegalArgumentException(
                "uso: DecompileList.java <diretorio-de-saida> <lista-de-enderecos>");
        }

        Path out = Paths.get(args[0]);
        Path decomp = out.resolve("decomp");
        Files.createDirectories(decomp);
        List<String> addresses = Files.readAllLines(Paths.get(args[1]), StandardCharsets.UTF_8);

        DecompInterface decompiler = new DecompInterface();
        try (BufferedWriter status = Files.newBufferedWriter(
                decomp.resolve("_status.tsv"), StandardCharsets.UTF_8)) {
            status.write("address\tstatus\n");
            if (!decompiler.openProgram(currentProgram)) {
                throw new IOException("nao foi possivel abrir o programa no descompilador");
            }

            for (String raw : addresses) {
                monitor.checkCancelled();
                String addressText = raw.trim();
                if (addressText.isEmpty() || addressText.startsWith("#")) {
                    continue;
                }
                processAddress(addressText, decomp, status, decompiler);
            }
        } finally {
            decompiler.dispose();
        }
    }

    private void processAddress(String addressText, Path decomp, BufferedWriter status,
            DecompInterface decompiler) throws IOException {
        String safeName = addressText.replaceAll("[^0-9A-Fa-f]", "_");
        Path destination = decomp.resolve(safeName + ".c");

        Address address;
        try {
            address = currentProgram.getAddressFactory().getAddress(addressText);
        } catch (RuntimeException exception) {
            address = null;
        }
        Function function = address == null
            ? null
            : currentProgram.getFunctionManager().getFunctionAt(address);
        if (function == null) {
            writeFailure(destination, "funcao nao encontrada");
            status.write(addressText + "\tnao-encontrada\n");
            return;
        }

        try {
            DecompileResults result = decompiler.decompileFunction(function, 60, monitor);
            if (!result.decompileCompleted()) {
                String message = cleanReason(result.getErrorMessage());
                String state = message.toLowerCase().contains("timeout") ? "timeout" : "erro";
                writeFailure(destination, message);
                status.write(addressText + "\t" + state + "\n");
                return;
            }

            String source = result.getDecompiledFunction().getC();
            Files.writeString(destination, source, StandardCharsets.UTF_8,
                StandardOpenOption.CREATE, StandardOpenOption.TRUNCATE_EXISTING);
            status.write(addressText + "\tok\n");
        } catch (Exception exception) {
            writeFailure(destination, cleanReason(exception.getMessage()));
            status.write(addressText + "\terro\n");
        }
    }

    private void writeFailure(Path destination, String reason) throws IOException {
        Files.writeString(destination, "// falha: " + cleanReason(reason) + "\n",
            StandardCharsets.UTF_8, StandardOpenOption.CREATE, StandardOpenOption.TRUNCATE_EXISTING);
    }

    private String cleanReason(String reason) {
        if (reason == null || reason.isBlank()) {
            return "motivo nao informado";
        }
        return reason.replace('\t', ' ').replace('\r', ' ').replace('\n', ' ').trim();
    }
}
