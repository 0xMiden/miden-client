import { Note, type NoteFilterTypes } from '@miden-sdk/miden-sdk';
import {
  AllowedPrivateData,
  BaseMessageSignerWalletAdapter,
  EventEmitter,
  scopePollingDetectionStrategy,
  SignKind,
  WalletConnectionError,
  WalletDisconnectionError,
  WalletName,
  WalletNotConnectedError,
  WalletNotReadyError,
  WalletReadyState,
  PrivateDataPermission,
  WalletAdapterNetwork,
  MidenSendTransaction,
  MidenTransaction,
  MidenConsumeTransaction,
  WalletTransactionError,
  Asset,
  InputNoteDetails,
  TransactionOutput,
  WalletTransactionOutput,
  b64ToU8,
} from '@miden-sdk/miden-wallet-adapter-base';

export interface MidenWalletEvents {
  connect(...args: unknown[]): unknown;
  disconnect(...args: unknown[]): unknown;
  accountChange(...args: unknown[]): unknown;
}

export interface MidenWallet extends EventEmitter<MidenWalletEvents> {
  address?: string;
  publicKey?: Uint8Array;
  requestSend(
    transaction: MidenSendTransaction
  ): Promise<{ transactionId?: string }>;
  requestConsume(
    transaction: MidenConsumeTransaction
  ): Promise<{ transactionId?: string }>;
  requestTransaction(
    transaction: MidenTransaction
  ): Promise<{ transactionId?: string }>;
  requestAssets(): Promise<{ assets: Asset[] }>;
  requestPrivateNotes(
    noteFilterType: NoteFilterTypes,
    noteIds?: string[]
  ): Promise<{
    privateNotes: InputNoteDetails[];
  }>;
  waitForTransaction(
    txId: string,
    timeout?: number
  ): Promise<WalletTransactionOutput>;
  signBytes(
    message: Uint8Array,
    kind: SignKind
  ): Promise<{ signature: Uint8Array }>;
  importPrivateNote(note: Uint8Array): Promise<{ noteId: string }>;
  requestConsumableNotes(): Promise<{ consumableNotes: InputNoteDetails[] }>;
  connect(
    privateDataPermission: PrivateDataPermission,
    network: WalletAdapterNetwork,
    allowedPrivateData?: AllowedPrivateData
  ): Promise<void>;
  disconnect(): Promise<void>;
}

export interface MidenWindow extends Window {
  midenWallet?: MidenWallet;
  miden?: MidenWallet;
}

declare const window: MidenWindow;

export interface MidenWalletAdapterConfig {
  appName?: string;
}

export const MidenWalletName = 'Miden Wallet' as WalletName<'Miden Wallet'>;

export class MidenWalletAdapter extends BaseMessageSignerWalletAdapter {
  name = MidenWalletName;
  url = 'https://miden.fi/';
  icon =
    'data:image/png;base64,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';
  readonly supportedTransactionVersions = null;

  private _connecting: boolean;
  private _wallet: MidenWallet | null;
  private _address: string | null;
  private _publicKey: Uint8Array | null;
  private _privateDataPermission: string;
  private _readyState: WalletReadyState =
    typeof window === 'undefined' || typeof document === 'undefined'
      ? WalletReadyState.Unsupported
      : WalletReadyState.NotDetected;

  constructor({ appName = 'sample' }: MidenWalletAdapterConfig = {}) {
    super();
    this._connecting = false;
    this._wallet = null;
    this._address = null;
    this._publicKey = null;
    this._privateDataPermission = PrivateDataPermission.UponRequest;

    if (this._readyState !== WalletReadyState.Unsupported) {
      scopePollingDetectionStrategy(() => {
        if (window?.midenWallet || window?.miden) {
          this._readyState = WalletReadyState.Installed;
          this.emit('readyStateChange', this._readyState);
          return true;
        }
        return false;
      });
    }
  }

  get address() {
    return this._address;
  }

  get publicKey() {
    return this._publicKey;
  }

  get privateDataPermission() {
    return this._privateDataPermission;
  }

  get connecting() {
    return this._connecting;
  }

  get readyState() {
    return this._readyState;
  }

  set readyState(readyState) {
    this._readyState = readyState;
  }

  async requestSend(transaction: MidenSendTransaction): Promise<string> {
    try {
      const wallet = this._wallet;
      if (!wallet || !this.address) throw new WalletNotConnectedError();
      try {
        const result = await wallet.requestSend(transaction);
        return result.transactionId!;
      } catch (error: any) {
        throw new WalletTransactionError(error?.message, error);
      }
    } catch (error: any) {
      this.emit('error', error);
      throw error;
    }
  }

  async requestConsume(transaction: MidenConsumeTransaction): Promise<string> {
    try {
      const wallet = this._wallet;
      if (!wallet || !this.address) throw new WalletNotConnectedError();
      try {
        const result = await wallet.requestConsume(transaction);
        return result.transactionId!;
      } catch (error: any) {
        throw new WalletTransactionError(error?.message, error);
      }
    } catch (error: any) {
      this.emit('error', error);
      throw error;
    }
  }

  async requestTransaction(transaction: MidenTransaction): Promise<string> {
    try {
      const wallet = this._wallet;
      if (!wallet || !this.address) throw new WalletNotConnectedError();
      try {
        const result = await wallet.requestTransaction(transaction);
        return result.transactionId!;
      } catch (error: any) {
        throw new WalletTransactionError(error?.message, error);
      }
    } catch (error: any) {
      this.emit('error', error);
      throw error;
    }
  }

  async requestAssets(): Promise<Asset[]> {
    try {
      const wallet = this._wallet;
      if (!wallet || !this.address) throw new WalletNotConnectedError();
      try {
        const result = await wallet.requestAssets();
        return result.assets;
      } catch (error: any) {
        throw new WalletTransactionError(error?.message, error);
      }
    } catch (error: any) {
      this.emit('error', error);
      throw error;
    }
  }

  async requestPrivateNotes(
    noteFilterType: NoteFilterTypes,
    noteIds?: string[]
  ): Promise<InputNoteDetails[]> {
    try {
      const wallet = this._wallet;
      if (!wallet || !this.address) throw new WalletNotConnectedError();
      try {
        const result = await wallet.requestPrivateNotes(
          noteFilterType,
          noteIds
        );
        return result.privateNotes;
      } catch (error: any) {
        throw new WalletTransactionError(error?.message, error);
      }
    } catch (error: any) {
      this.emit('error', error);
      throw error;
    }
  }

  async signBytes(message: Uint8Array, kind: SignKind): Promise<Uint8Array> {
    try {
      const wallet = this._wallet;
      if (!wallet || !this.address) throw new WalletNotConnectedError();
      try {
        const result = await wallet.signBytes(message, kind);
        return result.signature;
      } catch (error: any) {
        throw new WalletTransactionError(error?.message, error);
      }
    } catch (error: any) {
      this.emit('error', error);
      throw error;
    }
  }

  async importPrivateNote(note: Uint8Array): Promise<string> {
    try {
      const wallet = this._wallet;
      if (!wallet || !this.address) throw new WalletNotConnectedError();
      const result = await wallet.importPrivateNote(note);
      return result.noteId;
    } catch (error: any) {
      this.emit('error', error);
      throw error;
    }
  }

  async requestConsumableNotes(): Promise<InputNoteDetails[]> {
    try {
      const wallet = this._wallet;
      if (!wallet || !this.address) throw new WalletNotConnectedError();
      const result = await wallet.requestConsumableNotes();
      return result.consumableNotes;
    } catch (error: any) {
      this.emit('error', error);
      throw error;
    }
  }

  async waitForTransaction(
    txId: string,
    timeout?: number
  ): Promise<TransactionOutput> {
    try {
      const wallet = this._wallet;
      if (!wallet || !this.address) throw new WalletNotConnectedError();
      const result = await wallet.waitForTransaction(txId, timeout);
      if ('errorMessage' in result) {
        throw new WalletTransactionError(result.errorMessage);
      }
      const notes = result.outputNotes.map((serNote) => {
        const u8Arr = b64ToU8(serNote);
        return Note.deserialize(u8Arr);
      });
      return {
        txHash: result.txHash,
        outputNotes: notes,
      };
    } catch (error: any) {
      this.emit('error', error);
      throw error;
    }
  }

  async connect(
    privateDataPermission: PrivateDataPermission,
    network: WalletAdapterNetwork,
    allowedPrivateData?: AllowedPrivateData
  ): Promise<void> {
    try {
      if (this.connected || this.connecting) return;
      if (this._readyState !== WalletReadyState.Installed)
        throw new WalletNotReadyError();

      this._connecting = true;

      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const wallet = window.midenWallet! || window.miden!;

      try {
        await wallet.connect(
          privateDataPermission,
          network,
          allowedPrivateData
        );
        if (!wallet.address) {
          throw new WalletConnectionError();
        }
        this._address = wallet.address;
        this._publicKey = wallet.publicKey!;
      } catch (error: any) {
        throw new WalletConnectionError(error?.message, error);
      }

      this._wallet = wallet;
      this._privateDataPermission = privateDataPermission;

      this.emit('connect', this._address);
    } catch (error: any) {
      this.emit('error', error);
      throw error;
    } finally {
      this._connecting = false;
    }
  }

  async disconnect(): Promise<void> {
    const wallet = this._wallet;
    if (wallet) {
      this._wallet = null;
      this._address = null;
      this._publicKey = null;

      try {
        await wallet.disconnect();
      } catch (error: any) {
        this.emit('error', new WalletDisconnectionError(error?.message, error));
      }
    }

    this.emit('disconnect');
  }
}
