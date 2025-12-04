/**
 * Minimal WASI shims for browser environment.
 * These provide stub implementations for WASI interfaces that
 * the grammar plugins require but don't actually use.
 */

// Error type for WASI I/O
class WasiError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'WasiError';
  }
}

// Minimal stream implementation
class OutputStream {
  write(_contents: Uint8Array): bigint {
    // Silently discard output
    return BigInt(0);
  }

  blockingWriteAndFlush(_contents: Uint8Array): void {
    // No-op
  }

  blockingFlush(): void {
    // No-op
  }

  checkWrite(): bigint {
    return BigInt(1024 * 1024); // Allow large writes
  }

  subscribe(): void {
    // No-op
  }
}

class InputStream {
  read(_len: bigint): Uint8Array {
    return new Uint8Array(0);
  }

  blockingRead(_len: bigint): Uint8Array {
    return new Uint8Array(0);
  }

  subscribe(): void {
    // No-op
  }
}

// Create the WASI import object expected by jco-generated modules
export function createWasiImports() {
  const stdout = new OutputStream();
  const stderr = new OutputStream();
  const stdin = new InputStream();

  return {
    'wasi:cli/environment@0.2.3': {
      getEnvironment(): Array<[string, string]> {
        return [];
      },
      getArguments(): string[] {
        return [];
      },
    },

    'wasi:cli/exit@0.2.3': {
      exit(status: { tag: string; val?: number }): void {
        if (status.tag === 'err') {
          throw new WasiError(`WASI exit with error: ${status.val}`);
        }
      },
    },

    'wasi:cli/stdin@0.2.3': {
      getStdin(): InputStream {
        return stdin;
      },
    },

    'wasi:cli/stdout@0.2.3': {
      getStdout(): OutputStream {
        return stdout;
      },
    },

    'wasi:cli/stderr@0.2.3': {
      getStderr(): OutputStream {
        return stderr;
      },
    },

    'wasi:clocks/wall-clock@0.2.3': {
      now(): { seconds: bigint; nanoseconds: number } {
        const ms = Date.now();
        return {
          seconds: BigInt(Math.floor(ms / 1000)),
          nanoseconds: (ms % 1000) * 1_000_000,
        };
      },
      resolution(): { seconds: bigint; nanoseconds: number } {
        return { seconds: BigInt(0), nanoseconds: 1_000_000 };
      },
    },

    'wasi:filesystem/types@0.2.3': {
      // Stub - grammar plugins shouldn't use filesystem
      Descriptor: class {},
      DirectoryEntryStream: class {},
    },

    'wasi:filesystem/preopens@0.2.3': {
      getDirectories(): Array<[unknown, string]> {
        return [];
      },
    },

    'wasi:io/error@0.2.3': {
      Error: WasiError,
    },

    'wasi:io/streams@0.2.3': {
      InputStream,
      OutputStream,
    },

    'wasi:random/random@0.2.3': {
      getRandomBytes(len: bigint): Uint8Array {
        const bytes = new Uint8Array(Number(len));
        crypto.getRandomValues(bytes);
        return bytes;
      },
      getRandomU64(): bigint {
        const bytes = new Uint8Array(8);
        crypto.getRandomValues(bytes);
        const view = new DataView(bytes.buffer);
        return view.getBigUint64(0, true);
      },
    },
  };
}

// Grammar types import (the plugin exports these)
export const grammarTypesImport = {
  'arborium:grammar/types@0.1.0': {
    // Types are just interfaces, nothing to export
  },
};
