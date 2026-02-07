/*
** Golden file generator for biofabric-rs parity tests.
**
** Uses BioFabric's headless pipeline to load a network file (SIF or GW)
** with the default layout algorithm, then exports the resulting network as:
**   - output.noa  (node order attribute file)
**   - output.eda  (edge/link order attribute file)
**   - output.bif  (full session XML)
**
** Usage:
**   java -Djava.awt.headless=true -cp <classpath> GoldenGenerator \
**        <input.sif|input.gw> <output-dir>
**
** This replicates the exact same code path as BioFabric's headless mode,
** ensuring golden files match real BioFabric output bit-for-bit.
*/

import java.io.*;
import java.util.*;

import org.systemsbiology.biofabric.app.ArgParser;
import org.systemsbiology.biofabric.app.BioFabricWindow;
import org.systemsbiology.biofabric.api.io.Indenter;
import org.systemsbiology.biofabric.api.util.ExceptionHandler;
import org.systemsbiology.biofabric.cmd.CommandSet;
import org.systemsbiology.biofabric.cmd.HeadlessOracle;
import org.systemsbiology.biofabric.model.BioFabricNetwork;
import org.systemsbiology.biofabric.plugin.PlugInManager;
import org.systemsbiology.biofabric.ui.display.BioFabricPanel;
import org.systemsbiology.biofabric.util.ResourceManager;

public class GoldenGenerator {

    public static void main(String[] argv) {
        if (argv.length < 2) {
            System.err.println("Usage: GoldenGenerator <input.sif|input.gw> <output-dir>");
            System.err.println();
            System.err.println("Loads a SIF or GW file through BioFabric's default layout");
            System.err.println("pipeline and writes golden output files for parity testing.");
            System.exit(1);
        }

        String inputPath = argv[0];
        String outputDir = argv[1];

        try {
            new GoldenGenerator().run(inputPath, outputDir);
        } catch (Exception e) {
            System.err.println("ERROR: " + e.getMessage());
            e.printStackTrace(System.err);
            System.exit(1);
        }
    }

    private void run(String inputPath, String outputDir) throws Exception {

        File inputFile = new File(inputPath);
        if (!inputFile.exists()) {
            throw new FileNotFoundException("Input file not found: " + inputPath);
        }

        String lowerName = inputFile.getName().toLowerCase();
        boolean isGW = lowerName.endsWith(".gw");
        boolean isSIF = lowerName.endsWith(".sif");
        if (!isGW && !isSIF) {
            throw new IllegalArgumentException("Unsupported file type (expected .sif or .gw): " + inputFile.getName());
        }

        File outDir = new File(outputDir);
        outDir.mkdirs();

        // ---- 1. Initialize BioFabric in headless mode ----

        System.setProperty("java.awt.headless", "true");
        ResourceManager.initManager("org.systemsbiology.biofabric.props.BioFabric");

        HashMap<String, Object> argMap = new HashMap<String, Object>();
        BioFabricWindow bfw = new BioFabricWindow(argMap, null, true, true);
        ExceptionHandler.getHandler().initializeForHeadless(false);

        PlugInManager plum = new PlugInManager();
        CommandSet.initCmds("mainWindow", null, bfw, true, plum, new HeadlessOracle());
        bfw.initWindow(null);
        CommandSet cmd = CommandSet.getCmds("mainWindow");

        System.out.println("BioFabric headless mode initialized.");

        // ---- 2. Load network file (triggers default layout) ----

        System.out.println("Loading: " + inputFile.getName() + " (" + (isGW ? "GW" : "SIF") + ")");
        boolean loadOk;

        if (isSIF) {
            CommandSet.HeadlessImportSIFAction loader =
                cmd.new HeadlessImportSIFAction(inputFile);
            loadOk = loader.performOperation(null);
        } else {
            CommandSet.HeadlessImportGWAction loader =
                cmd.new HeadlessImportGWAction(inputFile);
            loadOk = loader.performOperation(null);
        }

        if (!loadOk) {
            throw new RuntimeException("BioFabric failed to load file: " + inputPath);
        }
        System.out.println("Network loaded and laid out successfully.");

        // ---- 3. Get the network from the panel ----

        BioFabricPanel bfp = cmd.getBFW().getFabricPanel();
        BioFabricNetwork bfn = (BioFabricNetwork) bfp.getNetwork();

        if (bfn == null) {
            throw new RuntimeException("Network is null after loading.");
        }

        int nodeCount = bfn.getNodeCount();
        int linkCount = bfn.getLinkCount(true);
        int linkCountNoShadow = bfn.getLinkCount(false);
        System.out.println("Network: " + nodeCount + " nodes, " +
                           linkCountNoShadow + " links (" + linkCount + " with shadows)");

        // ---- 4. Export golden files ----

        File noaFile = new File(outDir, "output.noa");
        PrintWriter noaPw = new PrintWriter(
            new BufferedWriter(new OutputStreamWriter(new FileOutputStream(noaFile), "UTF-8")));
        bfn.writeNOA(noaPw);
        noaPw.flush();
        noaPw.close();
        System.out.println("  Wrote: " + noaFile.getPath());

        File edaFile = new File(outDir, "output.eda");
        PrintWriter edaPw = new PrintWriter(
            new BufferedWriter(new OutputStreamWriter(new FileOutputStream(edaFile), "UTF-8")));
        bfn.writeEDA(edaPw);
        edaPw.flush();
        edaPw.close();
        System.out.println("  Wrote: " + edaFile.getPath());

        File bifFile = new File(outDir, "output.bif");
        PrintWriter bifPw = new PrintWriter(
            new BufferedWriter(new OutputStreamWriter(new FileOutputStream(bifFile), "UTF-8")));
        bfn.writeXML(bifPw, new Indenter(bifPw, Indenter.DEFAULT_INDENT), null, false);
        bifPw.flush();
        bifPw.close();
        System.out.println("  Wrote: " + bifFile.getPath());

        System.out.println("Done. Golden files in: " + outDir.getPath());
    }
}
