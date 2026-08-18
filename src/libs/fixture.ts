import type { NamedChannel } from "@/types/fixtures";

/**
 * Type guard to check if a DMX channel reference is a NamedChannel object.
 */
export function isNamedChannel(ch: unknown): ch is NamedChannel {
  return (
    typeof ch === "object" &&
    ch !== null &&
    "name" in ch &&
    typeof (ch as NamedChannel).name === "string"
  );
}

export function formatChannelName(ch: unknown): string {
  if (typeof ch === "string" && ch.trim() !== "") {
    return ch;
  }
  if (isNamedChannel(ch)) {
    return ch.name;
  }
  if (ch === null) {
    return "No Function / Reserved";
  }
  return "";
}
