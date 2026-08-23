import type * as React from "react";

export type Column<T> = {
  key: string;
  header: string;
  cell?: (row: T) => React.ReactNode;
  className?: string;
  sortable?: boolean;
};
