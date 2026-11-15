const WHOLE_NUMBER_FORMAT = new Intl.NumberFormat("en-US");

export function formatWholeNumber(wholeNumber: number): string {
  return WHOLE_NUMBER_FORMAT.format(wholeNumber);
}
