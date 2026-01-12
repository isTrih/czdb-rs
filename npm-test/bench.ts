/**
 * czdb-rs Node.js Benchmark
 * Run with: bun run bench.ts
 * Set CZDB_SECRET environment variable
 */

import { readFileSync, mkdirSync, writeFileSync } from 'fs';

const CZDB_SECRET = process.env.CZDB_SECRET;
if (!CZDB_SECRET) {
    console.error("Error: CZDB_SECRET environment variable not set");
    console.error("Please set it before running: export CZDB_SECRET=your_key");
    process.exit(1);
}

// Try to import native czdb library for comparison
let czdbNative: any = null;
try {
    czdbNative = await import('czdb');
    console.log("\n✓ Native czdb library found - will include comparison");
} catch (e) {
    console.log("\n⚠ Native czdb library not installed - skipping comparison");
    console.log("  Install with: npm install czdb");
}

// Dynamic import for WASM module (works with both CommonJS and ESM)
let SearcherClass: any = null;

async function getSearcherClass() {
    if (SearcherClass) return SearcherClass;

    try {
        // Use the wasm-pack generated czdb_rs.js (CommonJS entry)
        const czdbModule = await import('../pkg/czdb_rs.js');
        SearcherClass = czdbModule.CzdbSearcher;
        return SearcherClass;
    } catch (e) {
        console.error("Failed to load WASM module:", e);
        throw e;
    }
}

interface BenchmarkResult {
    name: string;
    mode: string;
    totalTime: number;
    avgTime: number;
    count: number;
    outputFile: string;
}

const results: BenchmarkResult[] = [];

// Output directory
const OUTPUT_DIR = "tests/output";
mkdirSync(OUTPUT_DIR, { recursive: true });

function loadIps(testFile: string): string[] {
    return readFileSync(testFile, 'utf-8')
        .split('\n')
        .map(l => l.trim())
        .filter(l => l.length > 0)
        .map(l => l.includes('/') ? l.split('/')[0] : l);
}

function saveResults(outputFile: string, lines: string[]) {
    writeFileSync(outputFile, lines.join("\n"), "utf-8");
    console.log(`  -> Results saved to: ${outputFile}`);
}

async function benchRust(name: string, mode: number, dbPath: string, testFile: string, outputFile: string) {
    console.log(`\nBenchmarking ${name} (mode: ${mode === 0 ? 'Memory' : 'BTree'})...`);

    try {
        const Searcher = await getSearcherClass();
        const dbData = readFileSync(dbPath);
        // mode parameter: 0 = Memory, 1 = BTree
        const searcher = new Searcher(dbData, CZDB_SECRET, mode);

        const ips = loadIps(testFile);
        const outLines: string[] = [];

        const start = performance.now();

        for (const ip of ips) {
            try {
                const res = searcher.search(ip);
                outLines.push(`${ip}\t${res}`);
            } catch (e: any) {
                outLines.push(`${ip}\tERROR:${e?.message ?? e}`);
            }
        }

        const end = performance.now();

        saveResults(outputFile, outLines);

        const totalTime = end - start;
        const avgTime = totalTime / ips.length;

        console.log(`  -> Processed ${ips.length} IPs in ${totalTime.toFixed(2)}ms. Avg: ${(avgTime * 1000).toFixed(4)}us/ip`);

        results.push({
            name,
            mode: mode === 0 ? 'Memory' : 'BTree',
            totalTime: parseFloat(totalTime.toFixed(2)),
            avgTime: parseFloat(avgTime.toFixed(4)),
            count: ips.length,
            outputFile
        });

    } catch (e) {
        console.error(`  -> Failed:`, e);
    }
}

// Benchmark native czdb library for comparison
async function benchNativeCzdb(name: string, dbPath: string, testFile: string, outputFile: string) {
    if (!czdbNative) {
        console.log(`\nSkipping ${name} - native czdb not available`);
        return;
    }

    console.log(`\nBenchmarking ${name} (native czdb)...`);

    try {
        const { default: DbSearcher, QueryType } = czdbNative;
        const searcher = new DbSearcher(dbPath, QueryType.MEMORY, CZDB_SECRET);

        const ips = loadIps(testFile);
        const outLines: string[] = [];

        const start = performance.now();

        for (const ip of ips) {
            try {
                const res = searcher.search(ip);
                outLines.push(`${ip}\t${res ?? 'ERROR:null'}`);
            } catch (e: any) {
                outLines.push(`${ip}\tERROR:${e?.message ?? e}`);
            }
        }

        const end = performance.now();

        // Close the database to release resources
        searcher.close();

        saveResults(outputFile, outLines);

        const totalTime = end - start;
        const avgTime = totalTime / ips.length;

        console.log(`  -> Processed ${ips.length} IPs in ${totalTime.toFixed(2)}ms. Avg: ${(avgTime * 1000).toFixed(4)}us/ip`);

        results.push({
            name,
            mode: 'Native',
            totalTime: parseFloat(totalTime.toFixed(2)),
            avgTime: parseFloat(avgTime.toFixed(4)),
            count: ips.length,
            outputFile
        });

    } catch (e) {
        console.error(`  -> Failed:`, e);
    }
}

function printTable() {
    console.log('\n=== Benchmark Summary ===');
    console.log('┌─────┬────────────────────────┬────────────┬────────────────┬────────────────┬───────┐');
    console.log('│ No. │ Name                   │ Mode       │ Total Time(ms) │ Avg Time(us)   │ Count │');
    console.log('├─────┼────────────────────────┼────────────┼────────────────┼────────────────┼───────┤');

    for (let i = 0; i < results.length; i++) {
        const res = results[i];
        const timeStr = res.totalTime.toFixed(2);
        const avgStr = (res.avgTime * 1000).toFixed(4);
        const modeStr = res.mode.padEnd(10);

        console.log(
            `│ ${(i + 1).toString().padStart(2)} │ ${res.name.padEnd(22)} │ ${modeStr} │ ${timeStr.padEnd(14)} │ ${avgStr.padEnd(14)} │ ${res.count.toString().padEnd(5)} │`
        );
    }

    console.log('└─────┴────────────────────────┴────────────┴────────────────┴────────────────┴───────┘');
}

// ------------------------
// Run Benchmarks
// ------------------------

console.log('=== czdb-rs Node.js Benchmark ===');
console.log(`Output directory: ${OUTPUT_DIR}`);

// czdb-rs WASM benchmarks
// IPv4 Memory
await benchRust(
    'WASM IPv4',
    0,  // Memory mode
    'czdb/cz88_public_v4.czdb',
    'tests/IPV4.txt',
    `${OUTPUT_DIR}/wasm_ipv4_memory.txt`
);

// IPv4 BTree
await benchRust(
    'WASM IPv4 BTree',
    1,  // BTree mode
    'czdb/cz88_public_v4.czdb',
    'tests/IPV4.txt',
    `${OUTPUT_DIR}/wasm_ipv4_btree.txt`
);

// IPv6 Memory
await benchRust(
    'WASM IPv6',
    0,  // Memory mode
    'czdb/cz88_public_v6.czdb',
    'tests/IPV6.txt',
    `${OUTPUT_DIR}/wasm_ipv6_memory.txt`
);

// IPv6 BTree
await benchRust(
    'WASM IPv6 BTree',
    1,  // BTree mode
    'czdb/cz88_public_v6.czdb',
    'tests/IPV6.txt',
    `${OUTPUT_DIR}/wasm_ipv6_btree.txt`
);

// Native czdb benchmarks (if available)
await benchNativeCzdb(
    'Native IPv4',
    'czdb/cz88_public_v4.czdb',
    'tests/IPV4.txt',
    `${OUTPUT_DIR}/native_ipv4.txt`
);

await benchNativeCzdb(
    'Native IPv6',
    'czdb/cz88_public_v6.czdb',
    'tests/IPV6.txt',
    `${OUTPUT_DIR}/native_ipv6.txt`
);

printTable();

console.log('\n=== Benchmark Complete ===');
