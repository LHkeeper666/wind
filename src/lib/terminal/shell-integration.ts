// Shell Integration - OSC sequence parser
// Supports OSC 7 (current directory) and OSC 133 (command tracking)

export type CommandState = 'idle' | 'prompt' | 'command' | 'finished';

export interface ShellState {
  currentDirectory: string;
  commandState: CommandState;
  lastExitCode: number | null;
  isCommandRunning: boolean;
}

export type ShellStateListener = (state: ShellState) => void;
export type DirectoryChangeListener = (directory: string) => void;

export class ShellIntegration {
  private state: ShellState = {
    currentDirectory: '',
    commandState: 'idle',
    lastExitCode: null,
    isCommandRunning: false,
  };

  private listeners: Set<ShellStateListener> = new Set();
  private directoryListeners: Set<DirectoryChangeListener> = new Set();
  private oscBuffer: string = '';
  private inOscSequence: boolean = false;

  subscribe(listener: ShellStateListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  onDirectoryChange(listener: DirectoryChangeListener): () => void {
    this.directoryListeners.add(listener);
    return () => this.directoryListeners.delete(listener);
  }

  getState(): ShellState {
    return { ...this.state };
  }

  // Process terminal output data, extracting OSC sequences
  processData(data: string): string {
    let result = '';
    let i = 0;

    while (i < data.length) {
      if (this.inOscSequence) {
        // Look for ST (String Terminator): ESC \ or BEL (0x07)
        if (data[i] === '\x07') {
          // BEL terminates OSC
          this.processOscSequence(this.oscBuffer);
          this.oscBuffer = '';
          this.inOscSequence = false;
          i++;
        } else if (data[i] === '\x1b' && i + 1 < data.length && data[i + 1] === '\\') {
          // ESC \ terminates OSC
          this.processOscSequence(this.oscBuffer);
          this.oscBuffer = '';
          this.inOscSequence = false;
          i += 2;
        } else {
          this.oscBuffer += data[i];
          i++;
        }
      } else {
        // Look for OSC start: ESC ]
        if (data[i] === '\x1b' && i + 1 < data.length && data[i + 1] === ']') {
          this.inOscSequence = true;
          this.oscBuffer = '';
          i += 2;
        } else {
          result += data[i];
          i++;
        }
      }
    }

    return result;
  }

  private processOscSequence(sequence: string) {
    // OSC 7 ; file://<path>
    if (sequence.startsWith('7;')) {
      const pathPart = sequence.substring(2);
      if (pathPart.startsWith('file://')) {
        const path = decodeURIComponent(pathPart.substring(7));
        this.updateState({ currentDirectory: path });
      }
      return;
    }

    // OSC 133 ; <command_state>
    if (sequence.startsWith('133;')) {
      const code = sequence.substring(4).trim();
      switch (code) {
        case 'A':
          // Prompt start
          this.updateState({ commandState: 'prompt', isCommandRunning: false });
          break;
        case 'B':
          // Prompt end / command input start
          this.updateState({ commandState: 'prompt' });
          break;
        case 'C':
          // Command execution start
          this.updateState({ commandState: 'command', isCommandRunning: true, lastExitCode: null });
          break;
        case 'D':
          // Command finished (no exit code)
          this.updateState({ commandState: 'finished', isCommandRunning: false });
          break;
        default:
          // D;<exit_code> - Command finished with exit code
          if (code.startsWith('D;')) {
            const exitCode = parseInt(code.substring(2), 10);
            this.updateState({
              commandState: 'finished',
              isCommandRunning: false,
              lastExitCode: isNaN(exitCode) ? null : exitCode,
            });
          }
          break;
      }
    }
  }

  private updateState(partial: Partial<ShellState>) {
    const oldDirectory = this.state.currentDirectory;
    this.state = { ...this.state, ...partial };

    // Notify directory change listeners if directory changed
    if (partial.currentDirectory && partial.currentDirectory !== oldDirectory) {
      for (const listener of this.directoryListeners) {
        listener(partial.currentDirectory);
      }
    }

    this.notifyListeners();
  }

  private notifyListeners() {
    const state = this.getState();
    for (const listener of this.listeners) {
      listener(state);
    }
  }

  reset() {
    this.state = {
      currentDirectory: '',
      commandState: 'idle',
      lastExitCode: null,
      isCommandRunning: false,
    };
    this.oscBuffer = '';
    this.inOscSequence = false;
    this.notifyListeners();
  }
}
