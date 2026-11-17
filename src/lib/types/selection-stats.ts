export interface SelectionStatsRoot {
  path: string;
  skip_rel_paths?: string[];
}

export interface SelectionStatsProgress {
  request_id: string;
  selected_items: number;
  files: number;
  dirs: number;
  total_bytes: number;
  incomplete: boolean;
  cancelled: boolean;
}
