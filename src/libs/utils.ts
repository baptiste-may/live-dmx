/**
 * Slices a string to a maximum length, adding an ellipsis at the beginning if it exceeds that length.
 * @param str - The string to slice.
 * @param maxLength - The maximum length of the resulting string.
 * @returns The sliced string with an ellipsis if it was too long.
 */
export function sliceString(str: string, maxLength: number): string {
  if (str.length <= maxLength) {
    return str;
  }
  return "..." + str.slice(-maxLength);
}
