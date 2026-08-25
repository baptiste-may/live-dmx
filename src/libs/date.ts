/**
 * Parses a date value from various representations:
 * - Date instance
 * - Numeric timestamp (milliseconds since Unix epoch)
 * - Numeric string (e.g. "1740501255000")
 * - ISO 8601 or RFC 2822 date string
 */
export function parseDate(value: string | number | Date | null | undefined): Date | null {
  if (value === null || value === undefined || value === "") {
    return null;
  }
  if (value instanceof Date) {
    return isNaN(value.getTime()) ? null : value;
  }
  if (typeof value === "number") {
    const date = new Date(value);
    return isNaN(date.getTime()) ? null : date;
  }
  if (typeof value === "string") {
    const trimmed = value.trim();
    if (!trimmed) {
      return null;
    }
    if (/^\d+$/.test(trimmed)) {
      const date = new Date(Number(trimmed));
      return isNaN(date.getTime()) ? null : date;
    }
    const date = new Date(trimmed);
    return isNaN(date.getTime()) ? null : date;
  }
  return null;
}

/**
 * Formats a date value into a localized date/time string.
 * Returns an empty string if the date value cannot be parsed.
 */
export function formatDate(
  value: string | number | Date | null | undefined,
  options: Intl.DateTimeFormatOptions = {
    dateStyle: "short",
    timeStyle: "short",
  },
  locale?: string,
): string {
  const parsed = parseDate(value);
  if (!parsed) {
    return "";
  }
  return parsed.toLocaleString(locale, options);
}
