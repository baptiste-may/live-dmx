export type RecentShow = {
  displayName: string;
  path: string;
  updatedAt: string | number | Date;
  /** Computed by the backend on every read; false when the file is missing. */
  available: boolean;
};
