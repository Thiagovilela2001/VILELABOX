// Desmonta uma lista de funcoes: uma instrucao por linha, com o valor das constantes lidas da memoria.
// Uso: -postScript DisasmList.java <diretorio-de-saida> <lista-de-enderecos>
// @category VILELABOX

import java.io.BufferedWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;

import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.Instruction;
import ghidra.program.model.mem.MemoryBlock;
import ghidra.program.model.symbol.RefType;
import ghidra.program.model.symbol.Reference;

public class DisasmList extends GhidraScript {
    @Override
    protected void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length != 2) {
            throw new IllegalArgumentException("uso: DisasmList.java <diretorio-de-saida> <lista-de-enderecos>");
        }
        Path out = Paths.get(args[0]).resolve("disasm");
        Files.createDirectories(out);
        List<String> lines = Files.readAllLines(Paths.get(args[1]), StandardCharsets.UTF_8);
        for (String line : lines) {
            String text = line.trim();
            if (text.isEmpty() || text.startsWith("#")) {
                continue;
            }
            Address entry = toAddr(text);
            Function f = entry == null ? null : getFunctionAt(entry);
            Path dest = out.resolve(text.replaceAll("[^0-9A-Fa-f]", "_") + ".asm");
            try (BufferedWriter w = Files.newBufferedWriter(dest, StandardCharsets.UTF_8)) {
                if (f == null) {
                    w.write("// falha: funcao nao encontrada\n");
                    continue;
                }
                for (Instruction ins : currentProgram.getListing().getInstructions(f.getBody(), true)) {
                    w.write(ins.getAddress() + "\t" + ins + constants(ins) + "\n");
                }
            }
        }
    }

    // Para cada leitura de memoria fora do codigo, anexa o valor como f64, f32 e i32.
    private String constants(Instruction ins) throws Exception {
        StringBuilder sb = new StringBuilder();
        for (Reference ref : ins.getReferencesFrom()) {
            Address to = ref.getToAddress();
            if (ref.getReferenceType().isCall() || ref.getReferenceType().isJump()
                    || ref.getReferenceType() == RefType.WRITE) {
                if (ref.getReferenceType().isCall()) {
                    sb.append("\t; call ").append(to);
                }
                continue;
            }
            MemoryBlock block = getMemoryBlock(to);
            if (block == null || block.isExecute()) {
                continue;
            }
            long bits = getLong(to);
            sb.append("\t; [").append(to).append("] f64=").append(Double.longBitsToDouble(bits))
                .append(" f32=").append(Float.intBitsToFloat((int) bits))
                .append(" i32=").append((int) bits);
        }
        return sb.toString();
    }
}
