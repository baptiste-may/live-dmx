/* eslint-disable no-console */
import fs from "node:fs";
import path from "node:path";
import zlib from "node:zlib";

const FIXTURES_URL = "https://open-fixture-library.org/download.ofl";
const FIXTURES_DIR = "fixtures";

function extractZip(buffer: Buffer, targetDir: string): void {
  let offset = 0;
  while (offset < buffer.length - 4) {
    const signature = buffer.readUInt32LE(offset);
    if (signature !== 0x04034b50) {
      break;
    }
    const compressionMethod = buffer.readUInt16LE(offset + 8);
    const compressedSize = buffer.readUInt32LE(offset + 18);
    const fileNameLength = buffer.readUInt16LE(offset + 26);
    const extraFieldLength = buffer.readUInt16LE(offset + 28);
    const fileName = buffer.toString("utf8", offset + 30, offset + 30 + fileNameLength);
    const dataOffset = offset + 30 + fileNameLength + extraFieldLength;
    const compressedData = buffer.subarray(dataOffset, dataOffset + compressedSize);

    const resolvedTarget = path.resolve(targetDir);
    const destPath = path.resolve(targetDir, fileName);
    if (!destPath.startsWith(resolvedTarget + path.sep) && destPath !== resolvedTarget) {
      throw new Error(`Zip entry outside target directory: ${fileName}`);
    }
    if (fileName.endsWith("/")) {
      fs.mkdirSync(destPath, { recursive: true });
    } else {
      fs.mkdirSync(path.dirname(destPath), { recursive: true });
      let fileData: Buffer | Uint8Array;
      if (compressionMethod === 8) {
        fileData = zlib.inflateRawSync(compressedData);
      } else if (compressionMethod === 0) {
        fileData = compressedData;
      } else {
        throw new Error(`Unsupported compression method: ${compressionMethod}`);
      }
      fs.writeFileSync(destPath, fileData);
    }
    offset = dataOffset + compressedSize;
  }
}

async function main(): Promise<void> {
  console.log("Downloading fixtures from Open Fixture Library...");

  if (!fs.existsSync(FIXTURES_DIR)) {
    fs.mkdirSync(FIXTURES_DIR, { recursive: true });
  }

  try {
    const response = await fetch(FIXTURES_URL);
    if (!response.ok) {
      throw new Error(`Failed to fetch fixtures: ${response.status} ${response.statusText}`);
    }

    const arrayBuffer = await response.arrayBuffer();
    const buffer = Buffer.from(arrayBuffer);

    console.log("Extracting fixtures...");
    extractZip(buffer, FIXTURES_DIR);

    console.log("Fixtures updated successfully.");
  } catch (error: unknown) {
    const err = error as Error;
    console.error("Failed to update fixtures:", err.message);
    process.exit(1);
  }
}

void main();
