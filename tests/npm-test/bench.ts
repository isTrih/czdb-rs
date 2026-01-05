import { readFileSync, mkdirSync, writeFileSync } from 'fs';
import { CzdbSearcher } from '../../pkg/czdb_rs.js';
import DbSearcher, { QueryType } from "czdb";

const secret = process.env.CZDB_SECRET;
if (!secret) {
    console.error("CZDB_SECRET not set");
    process.exit(1);
}

interface BenchResult {
    name: string;
    totalTime: number;
    avgTime: number;
    count: number;
    outputFile: string;
}

const results: BenchResult[] = [];

// 输出目录
const OUTPUT_DIR = "tests/output";
mkdirSync(OUTPUT_DIR, { recursive: true });

function loadIps(testFile: string): string[] {
    return readFileSync(testFile, 'utf-8')
        .split('\n')
        .map(l => l.trim())
        .filter(l => l.length > 0);
}

function saveResults(outputFile: string, lines: string[]) {
    writeFileSync(outputFile, lines.join("\n"), "utf-8");
    console.log(`✅ Results saved to: ${outputFile}`);
}

function benchRust(name: string, dbPath: string, testFile: string, outputFile: string) {
    console.log(`Benchmarking ${name}...`);
    try {
        const dbData = readFileSync(dbPath);
        const uint8Array = new Uint8Array(dbData);
        const searcher = new CzdbSearcher(uint8Array, secret);

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

        console.log(`${name}: Processed ${ips.length} IPs in ${totalTime.toFixed(2)}ms. Avg: ${avgTime.toFixed(4)}ms/ip`);

        results.push({
            name,
            totalTime: parseFloat(totalTime.toFixed(2)),
            avgTime: parseFloat(avgTime.toFixed(4)),
            count: ips.length,
            outputFile
        });

    } catch (e) {
        console.error(`Failed to run ${name}:`, e);
    }
}

function benchCzdb(name: string, dbPath: string, testFile: string, queryType: QueryType, outputFile: string) {
    console.log(`Benchmarking ${name}...`);
    try {
        const searcher = new DbSearcher(dbPath, queryType, secret);

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

        searcher.close();
        saveResults(outputFile, outLines);

        const totalTime = end - start;
        const avgTime = totalTime / ips.length;

        console.log(`${name}: Processed ${ips.length} IPs in ${totalTime.toFixed(2)}ms. Avg: ${avgTime.toFixed(4)}ms/ip`);

        results.push({
            name,
            totalTime: parseFloat(totalTime.toFixed(2)),
            avgTime: parseFloat(avgTime.toFixed(4)),
            count: ips.length,
            outputFile
        });
    } catch (e) {
        console.error(`Failed to run ${name}:`, e);
    }
}

// ------------------------
// Run Rust WASM Bench
// ------------------------
benchRust(
    "Rust WASM IPv4",
    "czdb/cz88_public_v4.czdb",
    "tests/IPV4.txt",
    `${OUTPUT_DIR}/js_wasm_ipv4.txt`
);

benchRust(
    "Rust WASM IPv6",
    "czdb/cz88_public_v6.czdb",
    "tests/IPV6.txt",
    `${OUTPUT_DIR}/js_wasm_ipv6.txt`
);

// ------------------------
// Run czdb Bench
// ------------------------
benchCzdb(
    "czdb (MEMORY) IPv4",
    "czdb/cz88_public_v4.czdb",
    "tests/IPV4.txt",
    QueryType.MEMORY,
    `${OUTPUT_DIR}/czdb-node_memory_ipv4.txt`
);

benchCzdb(
    "czdb (BTREE) IPv4",
    "czdb/cz88_public_v4.czdb",
    "tests/IPV4.txt",
    QueryType.BTREE,
    `${OUTPUT_DIR}/czdb-node_btree_ipv4.txt`
);

benchCzdb(
    "czdb (MEMORY) IPv6",
    "czdb/cz88_public_v6.czdb",
    "tests/IPV6.txt",
    QueryType.MEMORY,
    `${OUTPUT_DIR}/czdb-node_memory_ipv6.txt`
);

benchCzdb(
    "czdb (BTREE) IPv6",
    "czdb/cz88_public_v6.czdb",
    "tests/IPV6.txt",
    QueryType.BTREE,
    `${OUTPUT_DIR}/czdb-node_btree_ipv6.txt`
);

console.log("\n=== Benchmark Summary ===");
console.table(results);
