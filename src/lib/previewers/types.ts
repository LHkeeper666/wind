export interface Previewer {
  match(filePath: string): boolean;
  render(content: string | ArrayBuffer, container: HTMLElement): void | Promise<void>;
  dispose(): void;
}

export interface FileInfo {
  name: string;
  path: string;
  extension: string;
  size: number;
}
