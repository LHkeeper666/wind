export interface TocHeading {
  level: number;
  text: string;
  line: number;
  children: TocHeading[];
  expanded: boolean;
}

export interface Previewer {
  match(filePath: string): boolean;
  render(content: string | ArrayBuffer, container: HTMLElement): void | Promise<void>;
  dispose(): void;
  onHeadings?: (headings: TocHeading[]) => void;
}

export interface FileInfo {
  name: string;
  path: string;
  extension: string;
  size: number;
}
